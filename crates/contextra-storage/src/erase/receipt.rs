//! Canonical ErasureReceipt encoding and offline verification (SPEC §9.7, §13.2.1, INV-VER-1).
#![forbid(unsafe_code)]

use super::ledger::LedgerRecord;
use super::tickets::{EraseLevel, Residual, TicketStatus};
use crate::codec::varint::varint_put;
use contextra_storage_ports::traits::{AnchorStrength, AnchorValue};
use contextra_storage_ports::{Error, Stamp, Ticket};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasureStatus {
    Complete,
    NotMeasured,
    NotImplemented,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasureKind {
    NoLiveRunBlobOrRedo,
    MemtablesClean,
    KeyProbeAbsent,
    LedgerChainValid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureReport {
    pub kind: MeasureKind,
    pub status: MeasureStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptStageEntry {
    pub stage_name: &'static str,
    pub ledger_seq: u64,
    pub evidence_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAttestation {
    pub destroyed: bool,
    pub keys_in_backups: Option<u64>,
    pub weak_key_assurance: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorEvidence {
    pub strength: AnchorStrength,
    pub value: AnchorValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasureReceipt {
    pub instance_id: [u8; 16],
    pub ticket: Ticket,
    pub scope_digest: [u8; 32],
    pub level: EraseLevel,
    pub t_requested: u64,
    pub t_logical: u64,
    pub erase_epoch: Stamp,
    pub t_crypto: Option<u64>,
    pub t_physical: Option<u64>,
    pub t_verified: Option<u64>,
    pub stages: Vec<ReceiptStageEntry>,
    pub provider: ProviderAttestation,
    pub measures: Vec<MeasureReport>,
    pub verifier_report_digest: [u8; 32],
    pub ledger_head: [u8; 32],
    pub anchor: AnchorEvidence,
    pub restrictions_encountered: bool,
    pub status: TicketStatus,
    pub residual: Option<Residual>,
    pub canonical_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptVerdict {
    Valid { status: TicketStatus },
    Invalid { reason: &'static str },
}

impl ErasureReceipt {
    /// Computes status caps according to SPEC §9.7, §9.9 and §13.2.1.
    pub fn compute_status_caps(
        base_status: TicketStatus,
        level: EraseLevel,
        provider: &ProviderAttestation,
        measures: &[MeasureReport],
        host_device_exposure: bool,
        current_utc_millis: u64,
    ) -> (TicketStatus, Option<Residual>) {
        if base_status != TicketStatus::Complete {
            return (base_status, None);
        }

        // INV-VER-1: A mandatory measurement with NotMeasured or NotImplemented
        // caps Complete to Incomplete.
        for measure in measures {
            if measure.status != MeasureStatus::Complete {
                return (TicketStatus::Incomplete, None);
            }
        }

        // Cryptographic requires 4 measurements
        if level == EraseLevel::Cryptographic && measures.len() < 4 {
            return (TicketStatus::Incomplete, None);
        }

        // Statuscaps: BoundedBackupKeys, WeakKeyAssurance, LogicalOnly, DeclaredHostOrDeviceExposure
        if level == EraseLevel::Logical {
            return (
                TicketStatus::CompleteWithResidual(Residual::LogicalOnly),
                Some(Residual::LogicalOnly),
            );
        }

        if let Some(until_utc) = provider.keys_in_backups {
            if until_utc > current_utc_millis {
                let res = Residual::BoundedBackupKeys { until_utc };
                return (TicketStatus::CompleteWithResidual(res), Some(res));
            }
        }

        if provider.weak_key_assurance {
            let res = Residual::WeakKeyAssurance;
            return (TicketStatus::CompleteWithResidual(res), Some(res));
        }

        if host_device_exposure {
            let res = Residual::DeclaredHostOrDeviceExposure;
            return (TicketStatus::CompleteWithResidual(res), Some(res));
        }

        (TicketStatus::Complete, None)
    }

    /// Serializes canonical receipt bytes.
    pub fn build_canonical_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(512);
        buf.extend_from_slice(b"CTXSERC1"); // Magic + version 1
        buf.extend_from_slice(&self.instance_id);
        buf.extend_from_slice(self.ticket.as_bytes());
        buf.extend_from_slice(&self.scope_digest);
        buf.push(self.level as u8);
        buf.extend_from_slice(&self.t_requested.to_be_bytes());
        buf.extend_from_slice(&self.t_logical.to_be_bytes());
        buf.extend_from_slice(&self.erase_epoch.to_be_bytes());
        buf.extend_from_slice(&self.t_crypto.unwrap_or(0).to_be_bytes());
        buf.extend_from_slice(&self.t_physical.unwrap_or(0).to_be_bytes());
        buf.extend_from_slice(&self.t_verified.unwrap_or(0).to_be_bytes());
        varint_put(&mut buf, self.stages.len() as u64);
        for stage in &self.stages {
            varint_put(&mut buf, stage.stage_name.len() as u64);
            buf.extend_from_slice(stage.stage_name.as_bytes());
            buf.extend_from_slice(&stage.ledger_seq.to_be_bytes());
            buf.extend_from_slice(&stage.evidence_digest);
        }
        buf.push(if self.provider.destroyed { 1 } else { 0 });
        buf.extend_from_slice(&self.provider.keys_in_backups.unwrap_or(0).to_be_bytes());
        buf.push(if self.provider.weak_key_assurance { 1 } else { 0 });
        buf.extend_from_slice(&self.verifier_report_digest);
        buf.extend_from_slice(&self.ledger_head);
        buf.push(match self.anchor.strength {
            AnchorStrength::Weak => 0,
            AnchorStrength::Strong => 1,
        });
        buf.extend_from_slice(&self.anchor.value.erase_epoch.to_be_bytes());
        buf.extend_from_slice(&self.anchor.value.ledger_head.to_be_bytes());
        buf.extend_from_slice(&self.anchor.value.catalog_epoch.to_be_bytes());
        buf.push(if self.restrictions_encountered { 1 } else { 0 });
        buf.push(match self.status {
            TicketStatus::Pending => 0,
            TicketStatus::Blocked => 1,
            TicketStatus::Incomplete => 2,
            TicketStatus::Complete => 3,
            TicketStatus::CompleteWithResidual(_) => 4,
            TicketStatus::Failed => 5,
        });
        match self.residual {
            None => buf.push(0),
            Some(Residual::LogicalOnly) => buf.push(1),
            Some(Residual::BoundedBackupKeys { until_utc }) => {
                buf.push(2);
                buf.extend_from_slice(&until_utc.to_be_bytes());
            }
            Some(Residual::WeakKeyAssurance) => buf.push(3),
            Some(Residual::DeclaredHostOrDeviceExposure) => buf.push(4),
        }
        varint_put(&mut buf, self.measures.len() as u64);
        for m in &self.measures {
            buf.push(match m.kind {
                MeasureKind::NoLiveRunBlobOrRedo => 1,
                MeasureKind::MemtablesClean => 2,
                MeasureKind::KeyProbeAbsent => 3,
                MeasureKind::LedgerChainValid => 4,
            });
            buf.push(match m.status {
                MeasureStatus::Complete => 1,
                MeasureStatus::NotMeasured => 2,
                MeasureStatus::NotImplemented => 3,
            });
        }
        buf
    }
}

/// Offline verification of an ErasureReceipt against a slice of ledger records
/// and durable anchor evidence.
pub fn verify_receipt(
    receipt: &ErasureReceipt,
    ledger_slice: &[LedgerRecord],
    anchor: &AnchorEvidence,
) -> Result<ReceiptVerdict, Error> {
    // 1. Verify canonical bytes match
    let expected_bytes = receipt.build_canonical_bytes();
    if receipt.canonical_bytes != expected_bytes {
        return Ok(ReceiptVerdict::Invalid {
            reason: "canonical bytes mismatch",
        });
    }

    // 2. Verify timestamp monotonicity
    if receipt.t_logical < receipt.t_requested {
        return Ok(ReceiptVerdict::Invalid {
            reason: "logical time preceded requested time",
        });
    }
    if let Some(tc) = receipt.t_crypto {
        if tc < receipt.t_logical {
            return Ok(ReceiptVerdict::Invalid {
                reason: "crypto time preceded logical time",
            });
        }
    }
    if let Some(tp) = receipt.t_physical {
        if tp < receipt.t_logical {
            return Ok(ReceiptVerdict::Invalid {
                reason: "physical time preceded logical time",
            });
        }
        if let Some(tc) = receipt.t_crypto {
            if tp < tc {
                return Ok(ReceiptVerdict::Invalid {
                    reason: "physical time preceded crypto time",
                });
            }
        }
    }
    if let Some(tv) = receipt.t_verified {
        if tv < receipt.t_logical {
            return Ok(ReceiptVerdict::Invalid {
                reason: "verification time preceded logical time",
            });
        }
    }

    // 2b. INV-VER-1: Complete / CompleteWithResidual requires complete mandatory measures
    if matches!(
        receipt.status,
        TicketStatus::Complete | TicketStatus::CompleteWithResidual(_)
    ) {
        for m in &receipt.measures {
            if m.status != MeasureStatus::Complete {
                return Ok(ReceiptVerdict::Invalid {
                    reason: "incomplete mandatory measurement for completed receipt",
                });
            }
        }
        if receipt.level == EraseLevel::Cryptographic {
            let required = [
                MeasureKind::NoLiveRunBlobOrRedo,
                MeasureKind::MemtablesClean,
                MeasureKind::KeyProbeAbsent,
                MeasureKind::LedgerChainValid,
            ];
            for req in required {
                if !receipt
                    .measures
                    .iter()
                    .any(|m| m.kind == req && m.status == MeasureStatus::Complete)
                {
                    return Ok(ReceiptVerdict::Invalid {
                        reason: "missing mandatory measurement for cryptographic receipt",
                    });
                }
            }
        }
    }

    // 3. Verify ledger slice chain if non-empty
    if !ledger_slice.is_empty() {
        let mut prev_head = ledger_slice[0].previous_hash;
        for (i, rec) in ledger_slice.iter().enumerate() {
            if i > 0 && rec.previous_hash != prev_head {
                return Ok(ReceiptVerdict::Invalid {
                    reason: "ledger chain broken",
                });
            }
            prev_head = rec.mac;
        }
        if let Some(last) = ledger_slice.last() {
            if last.mac != receipt.ledger_head {
                return Ok(ReceiptVerdict::Invalid {
                    reason: "receipt ledger_head does not match slice head",
                });
            }
        }
    }

    // 4. Verify anchor consistency
    if receipt.anchor.value.ledger_head < receipt.stages.last().map(|s| s.ledger_seq).unwrap_or(0) {
        return Ok(ReceiptVerdict::Invalid {
            reason: "anchor ledger_head behind last receipt stage sequence",
        });
    }
    if receipt.anchor.value != anchor.value || receipt.anchor.strength != anchor.strength {
        return Ok(ReceiptVerdict::Invalid {
            reason: "anchor mismatch",
        });
    }

    // 5. Verify stage sequence monotonicity
    let mut last_seq = 0u64;
    for stage in &receipt.stages {
        if stage.ledger_seq < last_seq {
            return Ok(ReceiptVerdict::Invalid {
                reason: "stage ledger sequence not monotonic",
            });
        }
        last_seq = stage.ledger_seq;
    }

    Ok(ReceiptVerdict::Valid {
        status: receipt.status,
    })
}
