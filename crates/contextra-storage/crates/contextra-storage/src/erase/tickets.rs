//! Monotone projection of durable erase evidence (SPEC §9.2 and §9.5).
//!
//! A ticket is deliberately not a source of truth: recovery rebuilds it from
//! the ledger, catalog, anchor, and owner attestations. This module makes an
//! invalid forward edge impossible to represent in the in-memory projection.

use contextra_storage_ports::{Error, Protection, Stamp, Ticket};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EraseLevel { Logical, Physical, Scrubbed, Cryptographic }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EraseScope { Cluster, Range, Branch, Space }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TicketStage {
    /// A poisoned-handle intent is durable but has not become a gate.
    IntentRecorded,
    Gated,
    KeyDestroying,
    KeysDestroyed,
    Purging,
    Purged,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Residual {
    LogicalOnly,
    BoundedBackupKeys { until_utc: u64 },
    WeakKeyAssurance,
    DeclaredHostOrDeviceExposure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketStatus {
    Pending,
    Blocked,
    Incomplete,
    Complete,
    CompleteWithResidual(Residual),
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompletionEvidence {
    pub anchor_advanced: bool,
    pub verified: bool,
    pub owners_attested: bool,
    pub graveyard_empty: bool,
}

/// Events have already been made durable. `seq` is the ledger sequence and
/// gives replay an unambiguous, monotone ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketEvidence {
    IntentRecorded,
    GateApplied { stamp: Stamp },
    KeyDestroyIntent,
    KeysDestroyed,
    PurgeStarted,
    Purged,
    /// Final ledger event after structural verification and anchor advance.
    Completed { evidence: CompletionEvidence, residual: Option<Residual> },
    Blocked,
    Unblocked,
    Incomplete,
    Failed,
}

pub struct EraseTicket {
    ticket: Ticket,
    scope: EraseScope,
    protection: Protection,
    level: EraseLevel,
    stage: TicketStage,
    status: TicketStatus,
    gate_stamp: Option<Stamp>,
    last_seq: u64,
    last_evidence: Option<TicketEvidence>,
}

impl EraseTicket {
    pub fn new(ticket: Ticket, scope: EraseScope, protection: Protection, level: EraseLevel) -> Result<Self, Error> {
        validate_target(scope, protection, level)?;
        Ok(Self {
            ticket, scope, protection, level, stage: TicketStage::IntentRecorded,
            status: TicketStatus::Pending, gate_stamp: None, last_seq: 0, last_evidence: None,
        })
    }

    pub fn ticket(&self) -> Ticket { self.ticket }
    pub fn scope(&self) -> EraseScope { self.scope }
    pub fn level(&self) -> EraseLevel { self.level }
    pub fn stage(&self) -> TicketStage { self.stage }
    pub fn status(&self) -> TicketStatus { self.status }
    pub fn gate_stamp(&self) -> Option<Stamp> { self.gate_stamp }
    pub fn last_sequence(&self) -> u64 { self.last_seq }

    /// Raises a non-terminal target level before it reaches `Done`.
    pub fn escalate(&mut self, level: EraseLevel) -> Result<(), Error> {
        if self.stage == TicketStage::Done || level < self.level {
            return Err(Error::Limit { limit: "erase level must only escalate before completion" });
        }
        validate_target(self.scope, self.protection, level)?;
        self.level = level;
        Ok(())
    }

    /// Folds a single durable ledger event. Replaying the same sequence/event
    /// is idempotent; a different event at that sequence signals corruption.
    pub fn apply(&mut self, seq: u64, event: TicketEvidence) -> Result<(), Error> {
        if seq == 0 { return Err(Error::Corrupt { place: "zero ledger sequence" }); }
        if seq < self.last_seq { return Err(Error::Corrupt { place: "ticket ledger sequence regressed" }); }
        if seq == self.last_seq {
            return if self.last_evidence == Some(event) { Ok(()) }
                else { Err(Error::Corrupt { place: "ticket ledger sequence changed" }) };
        }
        if matches!(self.status, TicketStatus::Complete | TicketStatus::CompleteWithResidual(_) | TicketStatus::Failed) {
            return Err(Error::Corrupt { place: "evidence after terminal ticket state" });
        }
        if self.status == TicketStatus::Blocked
            && !matches!(event, TicketEvidence::Unblocked | TicketEvidence::Failed)
        {
            return Err(Error::Corrupt { place: "evidence advanced while ticket is blocked" });
        }

        match event {
            TicketEvidence::IntentRecorded => require_stage(self.stage, TicketStage::IntentRecorded)?,
            TicketEvidence::GateApplied { stamp } => {
                if stamp == 0 || self.stage != TicketStage::IntentRecorded {
                    return Err(Error::Corrupt { place: "gate evidence ordering" });
                }
                self.stage = TicketStage::Gated;
                self.gate_stamp = Some(stamp);
            }
            TicketEvidence::KeyDestroyIntent => {
                require_stage(self.stage, TicketStage::Gated)?;
                if self.level != EraseLevel::Cryptographic {
                    return Err(Error::Corrupt { place: "key destroy for non-cryptographic ticket" });
                }
                self.stage = TicketStage::KeyDestroying;
            }
            TicketEvidence::KeysDestroyed => {
                require_stage(self.stage, TicketStage::KeyDestroying)?;
                self.stage = TicketStage::KeysDestroyed;
            }
            TicketEvidence::PurgeStarted => {
                let valid = match self.level {
                    EraseLevel::Cryptographic => self.stage == TicketStage::KeysDestroyed,
                    EraseLevel::Physical | EraseLevel::Scrubbed => self.stage == TicketStage::Gated,
                    EraseLevel::Logical => false,
                };
                if !valid { return Err(Error::Corrupt { place: "purge before required erase stage" }); }
                self.stage = TicketStage::Purging;
            }
            TicketEvidence::Purged => {
                require_stage(self.stage, TicketStage::Purging)?;
                self.stage = TicketStage::Purged;
            }
            TicketEvidence::Completed { evidence, residual } => self.complete(evidence, residual)?,
            TicketEvidence::Blocked => self.status = TicketStatus::Blocked,
            TicketEvidence::Unblocked => {
                if self.status != TicketStatus::Blocked { return Err(Error::Corrupt { place: "unblock without blocked ticket" }); }
                self.status = TicketStatus::Pending;
            }
            TicketEvidence::Incomplete => self.status = TicketStatus::Incomplete,
            TicketEvidence::Failed => self.status = TicketStatus::Failed,
        }
        self.last_seq = seq;
        self.last_evidence = Some(event);
        Ok(())
    }

    fn complete(&mut self, evidence: CompletionEvidence, residual: Option<Residual>) -> Result<(), Error> {
        if !evidence.anchor_advanced || !evidence.verified || !evidence.owners_attested || !evidence.graveyard_empty {
            self.status = TicketStatus::Incomplete;
            return Ok(());
        }
        let ready = match self.level {
            EraseLevel::Logical => self.stage == TicketStage::Gated,
            EraseLevel::Physical | EraseLevel::Scrubbed => self.stage == TicketStage::Purged,
            EraseLevel::Cryptographic => self.stage == TicketStage::Purged,
        };
        if !ready { return Err(Error::Corrupt { place: "ticket completed before required stage" }); }
        self.stage = TicketStage::Done;
        self.status = match residual {
            Some(residual) => TicketStatus::CompleteWithResidual(residual),
            None if self.scope == EraseScope::Cluster && self.level == EraseLevel::Logical => {
                TicketStatus::CompleteWithResidual(Residual::LogicalOnly)
            }
            None => TicketStatus::Complete,
        };
        Ok(())
    }
}

fn require_stage(actual: TicketStage, expected: TicketStage) -> Result<(), Error> {
    if actual == expected { Ok(()) } else { Err(Error::Corrupt { place: "ticket stage ordering" }) }
}

fn validate_target(scope: EraseScope, protection: Protection, level: EraseLevel) -> Result<(), Error> {
    match (scope, protection, level) {
        (EraseScope::Cluster, Protection::Sealed, EraseLevel::Physical | EraseLevel::Scrubbed)
        | (EraseScope::Cluster, Protection::Plain, EraseLevel::Cryptographic)
        | (EraseScope::Range | EraseScope::Branch, Protection::Sealed, EraseLevel::Physical | EraseLevel::Scrubbed | EraseLevel::Cryptographic)
        | (EraseScope::Space, _, EraseLevel::Cryptographic) => Err(Error::Unsupported { what: "erase level is incompatible with scope/protection" }),
        _ => Ok(()),
    }
}
