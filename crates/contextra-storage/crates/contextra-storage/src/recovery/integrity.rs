//! Open-time integrity decisions, including the E-57 key-loss distinction.

use contextra_storage_ports::{Error, KeyState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRecoveryState { Available, Shredded }

/// A provider error must be propagated as `KeyUnavailable` by its adapter.
/// Absence alone is never evidence of deletion: only a durable erase record
/// permits recovery to classify it as Shredded.
pub fn reconcile_cluster_key(probe: KeyState, erase_proven: bool) -> Result<KeyRecoveryState, Error> {
    match (probe, erase_proven) {
        (KeyState::Present, _) => Ok(KeyRecoveryState::Available),
        (KeyState::Absent, true) => Ok(KeyRecoveryState::Shredded),
        (KeyState::Absent, false) => Err(Error::KeyLost),
        (KeyState::Disabled, _) => Err(Error::KeyUnavailable { retry_after_secs: None }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegrityDecision {
    pub writable: bool,
    pub rollback_detected: bool,
}

/// A monotone anchor ahead of either durable metadata source indicates a
/// rollback. The caller may expose diagnostics read-only, but cannot write.
pub fn check_anchor(
    anchor_catalog_epoch: u64,
    catalog_epoch: u64,
    anchor_ledger_epoch: u64,
    ledger_epoch: u64,
    read_only_requested: bool,
) -> Result<IntegrityDecision, Error> {
    let rollback_detected = anchor_catalog_epoch > catalog_epoch || anchor_ledger_epoch > ledger_epoch;
    if rollback_detected && !read_only_requested {
        return Err(Error::Rollback);
    }
    Ok(IntegrityDecision { writable: !rollback_detected, rollback_detected })
}
