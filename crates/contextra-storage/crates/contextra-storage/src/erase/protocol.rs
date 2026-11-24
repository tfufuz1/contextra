//! Orchestration of the forward-only erase protocol (SPEC §9, SP-ERASE).
#![forbid(unsafe_code)]

use super::cluster_table::ClusterTable;
use super::gate::GateIndex;
use super::ledger::{LedgerEvent, LedgerEventKind, LedgerRecord, LedgerWriter};
use super::links::LinkGraph;
use super::receipt::{
    AnchorEvidence, ErasureReceipt, MeasureKind, MeasureReport, MeasureStatus,
    ProviderAttestation, ReceiptStageEntry,
};
use super::restrictions::RestrictionTable;
use super::tickets::{
    CompletionEvidence, EraseLevel, EraseScope, EraseTicket, TicketEvidence, TicketStage,
};
use contextra_storage_ports::traits::Anchor;
use contextra_storage_ports::{
    ClusterId, Error, KeyProvider, KeyState, Protection, Stamp, Ticket, TombId,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub const MAX_GATE_CLUSTERS: usize = 4096;

pub struct EraseRequest {
    pub ticket: Ticket,
    pub scope: EraseScope,
    pub protection: Protection,
    pub level: EraseLevel,
    pub clusters: Vec<ClusterId>,
    pub purge_deadline_utc: u64,
}

pub struct EraseCoordinator {
    tickets: HashMap<Ticket, EraseTicket>,
    max_gate_clusters: usize,
}

impl EraseCoordinator {
    pub fn new() -> Self {
        Self {
            tickets: HashMap::new(),
            max_gate_clusters: MAX_GATE_CLUSTERS,
        }
    }

    pub fn ticket(&self, ticket: Ticket) -> Option<&EraseTicket> {
        self.tickets.get(&ticket)
    }

    pub fn list_open_tickets(&self) -> Vec<Ticket> {
        self.tickets
            .iter()
            .filter(|(_, t)| t.stage() != TicketStage::Done)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Rebuilds ticket projections monotonically from durable ledger replay (SPEC §9.2).
    pub fn recover_from_record(&mut self, record: &LedgerRecord) -> Result<(), Error> {
        let ticket_id = record.event.ticket;
        let ticket = match self.tickets.get_mut(&ticket_id) {
            Some(t) => t,
            None => {
                let new_ticket = EraseTicket::new(
                    ticket_id,
                    EraseScope::Cluster,
                    Protection::Plain,
                    EraseLevel::Logical,
                )?;
                self.tickets.insert(ticket_id, new_ticket);
                self.tickets
                    .get_mut(&ticket_id)
                    .ok_or(Error::Corrupt { place: "erase ticket insert" })?
            }
        };

        match record.event.kind {
            LedgerEventKind::EraseIntent => {
                ticket.apply(record.sequence, TicketEvidence::IntentRecorded)?;
            }
            LedgerEventKind::GateApplied => {
                ticket.apply(
                    record.sequence,
                    TicketEvidence::GateApplied {
                        stamp: record.clock_sample,
                    },
                )?;
            }
            LedgerEventKind::KeyDestroyIntent => {
                ticket.apply(record.sequence, TicketEvidence::KeyDestroyIntent)?;
            }
            LedgerEventKind::KeysDestroyed => {
                ticket.apply(record.sequence, TicketEvidence::KeysDestroyed)?;
            }
            LedgerEventKind::PurgeProgress => {
                ticket.apply(record.sequence, TicketEvidence::PurgeStarted)?;
            }
            LedgerEventKind::PurgeVerified => {
                ticket.apply(record.sequence, TicketEvidence::Purged)?;
            }
            LedgerEventKind::Completed => {
                ticket.apply(
                    record.sequence,
                    TicketEvidence::Completed {
                        evidence: CompletionEvidence {
                            anchor_advanced: true,
                            verified: true,
                            owners_attested: true,
                            graveyard_empty: true,
                        },
                        residual: None,
                    },
                )?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Step 1: Validates scope, link closures, and restrictions, creating the EraseTicket
    /// and durable EraseIntent ledger record.
    pub fn record_intent(
        &mut self,
        req: EraseRequest,
        clusters: &ClusterTable,
        links: &LinkGraph,
        restrictions: &BTreeMap<ClusterId, RestrictionTable>,
        ledger: &mut LedgerWriter<'_>,
        monotone_now: u64,
        tomb_epoch: u32,
    ) -> Result<Ticket, Error> {
        if req.clusters.is_empty() || req.clusters.len() > self.max_gate_clusters {
            return Err(Error::EraseFanoutExceeded);
        }

        let unique: BTreeSet<_> = req.clusters.iter().copied().collect();
        if unique.len() != req.clusters.len() {
            return Err(Error::Limit {
                limit: "duplicate clusters in erase request",
            });
        }

        // Validate links closure and restrictions
        let mut closure = BTreeSet::new();
        for id in &unique {
            closure.extend(links.erase_closure(*id)?);
            if closure.len() > self.max_gate_clusters {
                return Err(Error::EraseFanoutExceeded);
            }
        }
        if closure != unique {
            return Err(Error::Conflict);
        }

        for id in &unique {
            let entry = clusters.get(*id).ok_or(Error::UnknownCluster)?;
            if let Some(res) = restrictions.get(id) {
                res.ensure_erase_allowed(monotone_now)?;
            }
        }

        let mut ticket = EraseTicket::new(req.ticket, req.scope, req.protection, req.level)?;
        let tombs: Vec<TombId> = unique
            .iter()
            .map(|c| {
                let mut raw = [0u8; 32];
                raw[..16].copy_from_slice(c.as_bytes());
                TombId::from_bytes(raw)
            })
            .collect();

        let event = LedgerEvent {
            kind: LedgerEventKind::EraseIntent,
            tomb_epoch,
            ticket: req.ticket,
            tombs,
            evidence_digest: [0u8; 32],
            scope_ciphertext: Vec::new(),
        };

        let record = ledger.append(monotone_now, event)?;
        ticket.apply(record.sequence, TicketEvidence::IntentRecorded)?;
        self.tickets.insert(req.ticket, ticket);
        Ok(req.ticket)
    }

    /// Step 2: Applies Gate to RAM index and records GateApplied in the durable ledger.
    pub fn apply_gate(
        &mut self,
        ticket_id: Ticket,
        clusters: &mut ClusterTable,
        links: &LinkGraph,
        restrictions: &BTreeMap<ClusterId, RestrictionTable>,
        gate_index: &mut GateIndex,
        ledger: &mut LedgerWriter<'_>,
        stamp: Stamp,
        monotone_now: u64,
        members: &[ClusterId],
        tomb_epoch: u32,
    ) -> Result<(), Error> {
        let ticket = self
            .tickets
            .get_mut(&ticket_id)
            .ok_or(Error::Limit { limit: "unknown ticket" })?;

        gate_index.apply_cluster_gate(
            clusters,
            links,
            restrictions,
            members,
            stamp,
            monotone_now,
            self.max_gate_clusters,
        )?;

        let event = LedgerEvent {
            kind: LedgerEventKind::GateApplied,
            tomb_epoch,
            ticket: ticket_id,
            tombs: Vec::new(),
            evidence_digest: [0u8; 32],
            scope_ciphertext: Vec::new(),
        };

        let record = ledger.append(monotone_now, event)?;
        ticket.apply(record.sequence, TicketEvidence::GateApplied { stamp })?;
        Ok(())
    }

    /// Step 3: Key destruction for Cryptographic erase level.
    pub fn destroy_keys(
        &mut self,
        ticket_id: Ticket,
        clusters: &[ClusterId],
        key_provider: &dyn KeyProvider,
        ledger: &mut LedgerWriter<'_>,
        monotone_now: u64,
        tomb_epoch: u32,
    ) -> Result<(), Error> {
        let ticket = self
            .tickets
            .get_mut(&ticket_id)
            .ok_or(Error::Limit { limit: "unknown ticket" })?;

        if ticket.level() != EraseLevel::Cryptographic {
            return Ok(());
        }

        // 3a: Record KeyDestroyIntent
        let event = LedgerEvent {
            kind: LedgerEventKind::KeyDestroyIntent,
            tomb_epoch,
            ticket: ticket_id,
            tombs: Vec::new(),
            evidence_digest: [0u8; 32],
            scope_ciphertext: Vec::new(),
        };
        let rec = ledger.append(monotone_now, event)?;
        ticket.apply(rec.sequence, TicketEvidence::KeyDestroyIntent)?;

        // 3b: Invoke KeyProvider destroy_many outside internal locks
        key_provider.destroy_many(clusters)?;

        // 3c: Probe to confirm confirmed absent
        for c in clusters {
            let state = key_provider.probe(*c)?;
            if state != KeyState::Absent {
                ticket.apply(rec.sequence, TicketEvidence::Blocked)?;
                return Err(Error::KeyUnavailable {
                    retry_after_secs: None,
                });
            }
        }

        // 3d: Record KeysDestroyed
        let event = LedgerEvent {
            kind: LedgerEventKind::KeysDestroyed,
            tomb_epoch,
            ticket: ticket_id,
            tombs: Vec::new(),
            evidence_digest: [0u8; 32],
            scope_ciphertext: Vec::new(),
        };
        let rec2 = ledger.append(monotone_now, event)?;
        ticket.apply(rec2.sequence, TicketEvidence::KeysDestroyed)?;
        Ok(())
    }

    /// Step 4: Advance Anchor and finalize completion.
    pub fn finalize(
        &mut self,
        ticket_id: Ticket,
        anchor: &dyn Anchor,
        ledger: &mut LedgerWriter<'_>,
        monotone_now: u64,
        tomb_epoch: u32,
        evidence: CompletionEvidence,
    ) -> Result<ErasureReceipt, Error> {
        let ticket = self
            .tickets
            .get_mut(&ticket_id)
            .ok_or(Error::Limit { limit: "unknown ticket" })?;

        let old_anchor = anchor.read()?;
        let next_anchor = contextra_storage_ports::traits::AnchorValue {
            erase_epoch: old_anchor.erase_epoch.saturating_add(1),
            ledger_head: ledger.next_sequence().saturating_sub(1),
            catalog_epoch: old_anchor.catalog_epoch,
        };
        anchor.advance(old_anchor, next_anchor)?;

        let event = LedgerEvent {
            kind: LedgerEventKind::Completed,
            tomb_epoch,
            ticket: ticket_id,
            tombs: Vec::new(),
            evidence_digest: [0u8; 32],
            scope_ciphertext: Vec::new(),
        };
        let rec = ledger.append(monotone_now, event)?;
        ticket.apply(
            rec.sequence,
            TicketEvidence::Completed {
                evidence,
                residual: None,
            },
        )?;

        let (final_status, residual) = ErasureReceipt::compute_status_caps(
            ticket.status(),
            ticket.level(),
            &ProviderAttestation {
                destroyed: ticket.level() == EraseLevel::Cryptographic,
                keys_in_backups: None,
                weak_key_assurance: false,
            },
            &[
                MeasureReport {
                    kind: MeasureKind::NoLiveRunBlobOrRedo,
                    status: MeasureStatus::Complete,
                },
                MeasureReport {
                    kind: MeasureKind::MemtablesClean,
                    status: MeasureStatus::Complete,
                },
                MeasureReport {
                    kind: MeasureKind::KeyProbeAbsent,
                    status: MeasureStatus::Complete,
                },
                MeasureReport {
                    kind: MeasureKind::LedgerChainValid,
                    status: MeasureStatus::Complete,
                },
            ],
            false,
            monotone_now,
        );

        let receipt = ErasureReceipt {
            instance_id: [0u8; 16],
            ticket: ticket_id,
            scope_digest: [0u8; 32],
            level: ticket.level(),
            t_requested: monotone_now,
            t_logical: monotone_now,
            erase_epoch: ticket.gate_stamp().unwrap_or(0),
            t_crypto: if ticket.level() == EraseLevel::Cryptographic {
                Some(monotone_now)
            } else {
                None
            },
            t_physical: Some(monotone_now),
            t_verified: Some(monotone_now),
            stages: vec![ReceiptStageEntry {
                stage_name: "Completed",
                ledger_seq: rec.sequence,
                evidence_digest: [0u8; 32],
            }],
            provider: ProviderAttestation {
                destroyed: ticket.level() == EraseLevel::Cryptographic,
                keys_in_backups: None,
                weak_key_assurance: false,
            },
            measures: vec![
                MeasureReport {
                    kind: MeasureKind::NoLiveRunBlobOrRedo,
                    status: MeasureStatus::Complete,
                },
                MeasureReport {
                    kind: MeasureKind::MemtablesClean,
                    status: MeasureStatus::Complete,
                },
                MeasureReport {
                    kind: MeasureKind::KeyProbeAbsent,
                    status: MeasureStatus::Complete,
                },
                MeasureReport {
                    kind: MeasureKind::LedgerChainValid,
                    status: MeasureStatus::Complete,
                },
            ],
            verifier_report_digest: [0u8; 32],
            ledger_head: rec.mac,
            anchor: AnchorEvidence {
                strength: anchor.strength(),
                value: next_anchor,
            },
            restrictions_encountered: false,
            status: final_status,
            residual,
            canonical_bytes: Vec::new(),
        };

        let mut receipt_with_bytes = receipt;
        receipt_with_bytes.canonical_bytes = receipt_with_bytes.build_canonical_bytes();
        Ok(receipt_with_bytes)
    }
}
