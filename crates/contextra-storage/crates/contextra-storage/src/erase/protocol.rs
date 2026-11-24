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
const DESTROY_KEY_BATCH: usize = 256;

pub struct EraseRequest {
    pub ticket: Ticket,
    pub scope: EraseScope,
    pub protection: Protection,
    pub level: EraseLevel,
    pub clusters: Vec<ClusterId>,
    pub purge_deadline_utc: u64,
}

struct TicketMeta {
    clusters: Vec<ClusterId>,
    t_requested: u64,
    t_logical: Option<u64>,
    t_crypto: Option<u64>,
    t_physical: Option<u64>,
    stages: Vec<ReceiptStageEntry>,
    scope_digest: [u8; 32],
    restrictions_encountered: bool,
}

pub struct EraseCoordinator {
    tickets: HashMap<Ticket, EraseTicket>,
    meta: HashMap<Ticket, TicketMeta>,
    max_gate_clusters: usize,
}

impl Default for EraseCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

fn compute_scope_digest(
    scope: EraseScope,
    protection: Protection,
    level: EraseLevel,
    clusters: &[ClusterId],
) -> [u8; 32] {
    use crate::codec::checksums::crc32c;
    let mut buf = Vec::with_capacity(4 + clusters.len() * 16);
    buf.push(scope as u8);
    buf.push(protection as u8);
    buf.push(level as u8);
    for c in clusters {
        buf.extend_from_slice(c.as_bytes());
    }
    let mut out = [0u8; 32];
    for i in 0..8 {
        buf.push(i as u8);
        let c = crc32c(&buf);
        out[i * 4..(i + 1) * 4].copy_from_slice(&c.to_be_bytes());
        buf.pop();
    }
    out
}

impl EraseCoordinator {
    pub fn new() -> Self {
        Self {
            tickets: HashMap::new(),
            meta: HashMap::new(),
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

        // If replaying a KeyDestroyIntent or PurgeProgress on a ticket created
        // with default Logical/Plain during replay, upgrade its target shape
        // before applying the stage transition.
        if record.event.kind == LedgerEventKind::KeyDestroyIntent {
            if let Some(existing) = self.tickets.get(&ticket_id) {
                if existing.level() != EraseLevel::Cryptographic
                    && existing.stage() == TicketStage::Gated
                {
                    let mut upgraded = EraseTicket::new(
                        ticket_id,
                        existing.scope(),
                        Protection::Sealed,
                        EraseLevel::Cryptographic,
                    )?;
                    if let Some(stamp) = existing.gate_stamp() {
                        let base_seq = existing.last_sequence().max(1);
                        upgraded.apply(base_seq, TicketEvidence::GateApplied { stamp })?;
                    }
                    self.tickets.insert(ticket_id, upgraded);
                }
            }
        } else if record.event.kind == LedgerEventKind::PurgeProgress {
            if let Some(existing) = self.tickets.get(&ticket_id) {
                if existing.level() == EraseLevel::Logical && existing.stage() == TicketStage::Gated
                {
                    let mut upgraded = EraseTicket::new(
                        ticket_id,
                        existing.scope(),
                        Protection::Plain,
                        EraseLevel::Physical,
                    )?;
                    if let Some(stamp) = existing.gate_stamp() {
                        let base_seq = existing.last_sequence().max(1);
                        upgraded.apply(base_seq, TicketEvidence::GateApplied { stamp })?;
                    }
                    self.tickets.insert(ticket_id, upgraded);
                }
            }
        }

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

        let stage_name = match record.event.kind {
            LedgerEventKind::EraseIntent => {
                ticket.apply(record.sequence, TicketEvidence::IntentRecorded)?;
                "EraseIntent"
            }
            LedgerEventKind::GateApplied => {
                ticket.apply(
                    record.sequence,
                    TicketEvidence::GateApplied {
                        stamp: record.clock_sample.max(1),
                    },
                )?;
                "GateApplied"
            }
            LedgerEventKind::KeyDestroyIntent => {
                ticket.apply(record.sequence, TicketEvidence::KeyDestroyIntent)?;
                "KeyDestroyIntent"
            }
            LedgerEventKind::KeysDestroyed => {
                ticket.apply(record.sequence, TicketEvidence::KeysDestroyed)?;
                "KeysDestroyed"
            }
            LedgerEventKind::PurgeProgress => {
                ticket.apply(record.sequence, TicketEvidence::PurgeStarted)?;
                "PurgeProgress"
            }
            LedgerEventKind::PurgeVerified => {
                ticket.apply(record.sequence, TicketEvidence::Purged)?;
                "PurgeVerified"
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
                "Completed"
            }
            _ => return Ok(()),
        };

        let entry = self.meta.entry(ticket_id).or_insert_with(|| TicketMeta {
            clusters: Vec::new(),
            t_requested: record.clock_sample,
            t_logical: None,
            t_crypto: None,
            t_physical: None,
            stages: Vec::new(),
            scope_digest: [0u8; 32],
            restrictions_encountered: false,
        });
        entry.stages.push(ReceiptStageEntry {
            stage_name,
            ledger_seq: record.sequence,
            evidence_digest: record.event.evidence_digest,
        });
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
            let _entry = clusters.get(*id).ok_or(Error::UnknownCluster)?;
            if let Some(res) = restrictions.get(id) {
                res.ensure_erase_allowed(monotone_now)?;
            }
        }

        let mut ticket = EraseTicket::new(req.ticket, req.scope, req.protection, req.level)?;
        let sorted_clusters: Vec<ClusterId> = unique.iter().copied().collect();
        let scope_digest =
            compute_scope_digest(req.scope, req.protection, req.level, &sorted_clusters);
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
            evidence_digest: scope_digest,
            scope_ciphertext: Vec::new(),
        };

        let record = ledger.append(monotone_now, event)?;
        ticket.apply(record.sequence, TicketEvidence::IntentRecorded)?;
        self.tickets.insert(req.ticket, ticket);
        self.meta.insert(
            req.ticket,
            TicketMeta {
                clusters: sorted_clusters,
                t_requested: monotone_now,
                t_logical: None,
                t_crypto: None,
                t_physical: None,
                stages: vec![ReceiptStageEntry {
                    stage_name: "EraseIntent",
                    ledger_seq: record.sequence,
                    evidence_digest: scope_digest,
                }],
                scope_digest,
                restrictions_encountered: false,
            },
        );
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
        if ticket.stage() != TicketStage::IntentRecorded {
            return Err(Error::Corrupt {
                place: "gate evidence ordering",
            });
        }
        if let Some(meta) = self.meta.get(&ticket_id) {
            let member_set: BTreeSet<_> = members.iter().copied().collect();
            let expected_set: BTreeSet<_> = meta.clusters.iter().copied().collect();
            if !expected_set.is_empty() && member_set != expected_set {
                return Err(Error::Conflict);
            }
        }

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
        if let Some(meta) = self.meta.get_mut(&ticket_id) {
            meta.t_logical = Some(monotone_now.max(meta.t_requested));
            meta.stages.push(ReceiptStageEntry {
                stage_name: "GateApplied",
                ledger_seq: record.sequence,
                evidence_digest: [0u8; 32],
            });
        }
        Ok(())
    }

    /// Step 3: Key destruction for Cryptographic erase level (batched <= 256 per R9.1).
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
        if let Some(meta) = self.meta.get_mut(&ticket_id) {
            meta.stages.push(ReceiptStageEntry {
                stage_name: "KeyDestroyIntent",
                ledger_seq: rec.sequence,
                evidence_digest: [0u8; 32],
            });
        }

        // 3b: Invoke KeyProvider destroy_many outside internal locks in batches of <= 256
        for chunk in clusters.chunks(DESTROY_KEY_BATCH) {
            key_provider.destroy_many(chunk)?;
        }

        // 3c: Probe to confirm confirmed Absent (never Disabled or Unavailable)
        for c in clusters {
            let state = key_provider.probe(*c)?;
            if state != KeyState::Absent {
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
        if let Some(meta) = self.meta.get_mut(&ticket_id) {
            let logical = meta.t_logical.unwrap_or(meta.t_requested);
            meta.t_crypto = Some(monotone_now.max(logical));
            meta.stages.push(ReceiptStageEntry {
                stage_name: "KeysDestroyed",
                ledger_seq: rec2.sequence,
                evidence_digest: [0u8; 32],
            });
        }
        Ok(())
    }

    /// Step 4a: Start physical purge (for Physical, Scrubbed, or Cryptographic tickets).
    pub fn start_purge(
        &mut self,
        ticket_id: Ticket,
        ledger: &mut LedgerWriter<'_>,
        monotone_now: u64,
        tomb_epoch: u32,
    ) -> Result<(), Error> {
        let ticket = self
            .tickets
            .get_mut(&ticket_id)
            .ok_or(Error::Limit { limit: "unknown ticket" })?;
        if ticket.level() == EraseLevel::Logical {
            return Ok(());
        }
        let event = LedgerEvent {
            kind: LedgerEventKind::PurgeProgress,
            tomb_epoch,
            ticket: ticket_id,
            tombs: Vec::new(),
            evidence_digest: [0u8; 32],
            scope_ciphertext: Vec::new(),
        };
        let rec = ledger.append(monotone_now, event)?;
        ticket.apply(rec.sequence, TicketEvidence::PurgeStarted)?;
        if let Some(meta) = self.meta.get_mut(&ticket_id) {
            meta.stages.push(ReceiptStageEntry {
                stage_name: "PurgeProgress",
                ledger_seq: rec.sequence,
                evidence_digest: [0u8; 32],
            });
        }
        Ok(())
    }

    /// Step 4b: Complete physical purge and record PurgeVerified.
    pub fn complete_purge(
        &mut self,
        ticket_id: Ticket,
        clusters: &mut ClusterTable,
        targets: &[ClusterId],
        evidence_digest: [u8; 32],
        ledger: &mut LedgerWriter<'_>,
        monotone_now: u64,
        tomb_epoch: u32,
    ) -> Result<(), Error> {
        let ticket = self
            .tickets
            .get_mut(&ticket_id)
            .ok_or(Error::Limit { limit: "unknown ticket" })?;
        if ticket.level() == EraseLevel::Logical {
            return Ok(());
        }
        let event = LedgerEvent {
            kind: LedgerEventKind::PurgeVerified,
            tomb_epoch,
            ticket: ticket_id,
            tombs: Vec::new(),
            evidence_digest,
            scope_ciphertext: Vec::new(),
        };
        let rec = ledger.append(monotone_now, event)?;
        ticket.apply(rec.sequence, TicketEvidence::Purged)?;
        for c in targets {
            clusters.mark_purged(*c)?;
        }
        if let Some(meta) = self.meta.get_mut(&ticket_id) {
            let min_t = meta
                .t_crypto
                .or(meta.t_logical)
                .unwrap_or(meta.t_requested);
            meta.t_physical = Some(monotone_now.max(min_t));
            meta.stages.push(ReceiptStageEntry {
                stage_name: "PurgeVerified",
                ledger_seq: rec.sequence,
                evidence_digest,
            });
        }
        Ok(())
    }

    /// Step 5: Advance Anchor and finalize completion, producing a verifiable ErasureReceipt.
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

        let event = LedgerEvent {
            kind: LedgerEventKind::Completed,
            tomb_epoch,
            ticket: ticket_id,
            tombs: Vec::new(),
            evidence_digest: [0u8; 32],
            scope_ciphertext: Vec::new(),
        };
        let rec = ledger.append(monotone_now, event)?;

        let old_anchor = anchor.read()?;
        let next_anchor = contextra_storage_ports::traits::AnchorValue {
            erase_epoch: old_anchor.erase_epoch.saturating_add(1),
            ledger_head: rec.sequence.max(old_anchor.ledger_head),
            catalog_epoch: old_anchor.catalog_epoch,
        };
        anchor.advance(old_anchor, next_anchor)?;

        let measures = vec![
            MeasureReport {
                kind: MeasureKind::NoLiveRunBlobOrRedo,
                status: if evidence.verified {
                    MeasureStatus::Complete
                } else {
                    MeasureStatus::NotMeasured
                },
            },
            MeasureReport {
                kind: MeasureKind::MemtablesClean,
                status: if evidence.verified && evidence.graveyard_empty {
                    MeasureStatus::Complete
                } else {
                    MeasureStatus::NotMeasured
                },
            },
            MeasureReport {
                kind: MeasureKind::KeyProbeAbsent,
                status: if ticket.level() != EraseLevel::Cryptographic
                    || ticket.stage() >= TicketStage::KeysDestroyed
                {
                    MeasureStatus::Complete
                } else {
                    MeasureStatus::NotMeasured
                },
            },
            MeasureReport {
                kind: MeasureKind::LedgerChainValid,
                status: if evidence.anchor_advanced && evidence.owners_attested {
                    MeasureStatus::Complete
                } else {
                    MeasureStatus::NotMeasured
                },
            },
        ];

        let provider = ProviderAttestation {
            destroyed: ticket.level() == EraseLevel::Cryptographic
                && ticket.stage() >= TicketStage::KeysDestroyed,
            keys_in_backups: None,
            weak_key_assurance: false,
        };

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
            &provider,
            &measures,
            false,
            monotone_now,
        );

        let meta = self.meta.get_mut(&ticket_id);
        let (t_requested, t_logical, t_crypto, t_physical, scope_digest, restrictions_encountered, stages) =
            match meta {
                Some(m) => {
                    m.stages.push(ReceiptStageEntry {
                        stage_name: "Completed",
                        ledger_seq: rec.sequence,
                        evidence_digest: [0u8; 32],
                    });
                    let tl = m.t_logical.unwrap_or(monotone_now.max(m.t_requested));
                    let tc = m.t_crypto.map(|t| t.max(tl));
                    let tp = m
                        .t_physical
                        .map(|t| t.max(tc.unwrap_or(tl)))
                        .or_else(|| Some(monotone_now.max(tc.unwrap_or(tl))));
                    (
                        m.t_requested,
                        tl,
                        tc,
                        tp,
                        m.scope_digest,
                        m.restrictions_encountered,
                        m.stages.clone(),
                    )
                }
                None => (
                    monotone_now,
                    monotone_now,
                    if ticket.level() == EraseLevel::Cryptographic {
                        Some(monotone_now)
                    } else {
                        None
                    },
                    Some(monotone_now),
                    [0u8; 32],
                    false,
                    vec![ReceiptStageEntry {
                        stage_name: "Completed",
                        ledger_seq: rec.sequence,
                        evidence_digest: [0u8; 32],
                    }],
                ),
            };
        let t_verified = Some(
            monotone_now
                .max(t_logical)
                .max(t_physical.unwrap_or(t_logical)),
        );

        let mut receipt = ErasureReceipt {
            instance_id: [0u8; 16],
            ticket: ticket_id,
            scope_digest,
            level: ticket.level(),
            t_requested,
            t_logical,
            erase_epoch: ticket.gate_stamp().unwrap_or(0),
            t_crypto,
            t_physical,
            t_verified,
            stages,
            provider,
            measures,
            verifier_report_digest: [0u8; 32],
            ledger_head: rec.mac,
            anchor: AnchorEvidence {
                strength: anchor.strength(),
                value: next_anchor,
            },
            restrictions_encountered,
            status: final_status,
            residual,
            canonical_bytes: Vec::new(),
        };
        receipt.canonical_bytes = receipt.build_canonical_bytes();
        Ok(receipt)
    }
}
