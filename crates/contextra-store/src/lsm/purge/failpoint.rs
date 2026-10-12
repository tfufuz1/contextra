// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Failpoint-Injektion für Purge-Orchestrator (C5.1 / C5.6 / Anhang B).
// INVARIANTEN: Laufzeit-Failpoints ohne Env-Variablen via OrxFailpointSet. Zero-Cost ohne feature/test.

use crate::compaction::purge_filter::PurgeBlocked;
use std::collections::HashSet;
use std::sync::Arc;

/// Known failpoint names from Specification Appendix B.
pub const ALL_PURGE_FAILPOINTS: [&str; 14] = [
    "after_intent",
    "after_fence",
    "after_flush",
    "mid_compaction_output",
    "after_compaction_before_manifest",
    "after_manifest_before_unlink",
    "after_journal_record",
    "mid_unlink",
    "after_unlink_before_wal",
    "after_wal_retire",
    "after_proof_mint",
    "after_witness",
    "after_finalize",
    "after_chain_before_ack",
];

/// Store-relevant failpoint names (handled in contextra-store).
pub const STORE_PURGE_FAILPOINTS: [&str; 8] = [
    "after_intent",
    "after_fence",
    "after_flush",
    "mid_compaction_output",
    "after_compaction_before_manifest",
    "after_manifest_before_unlink",
    "after_journal_record",
    "mid_unlink",
];

/// Engine-relevant failpoint names (handled in contextra-engine / B05).
pub const ENGINE_PURGE_FAILPOINTS: [&str; 6] = [
    "after_unlink_before_wal",
    "after_wal_retire",
    "after_proof_mint",
    "after_witness",
    "after_finalize",
    "after_chain_before_ack",
];

/// Error returned when an operation is interrupted by a failpoint or fails during purge.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OrxPurgeError {
    #[error("Purge operation interrupted by failpoint: {point}")]
    Interrupted { point: &'static str },
    #[error("I/O error during purge execution: {0}")]
    Io(String),
    #[error("Purge operation blocked: {0:?}")]
    Blocked(PurgeBlocked),
}

/// Thread-safe explicit failpoint set passed into `PurgeOrchestrator` or tests.
#[derive(Debug, Clone, Default)]
pub struct OrxFailpointSet {
    active_points: Arc<parking_lot::Mutex<HashSet<String>>>,
}

impl OrxFailpointSet {
    /// Creates a new empty failpoint set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables a failpoint by name (e.g. "after_intent").
    pub fn enable(&self, point: &str) {
        self.active_points.lock().insert(point.to_string());
    }

    /// Disables a failpoint by name.
    pub fn disable(&self, point: &str) {
        self.active_points.lock().remove(point);
    }

    /// Returns true if the failpoint `point` is enabled.
    pub fn is_enabled(&self, point: &str) -> bool {
        self.active_points.lock().contains(point)
    }

    /// Triggers a failpoint if enabled, returning `Err(OrxPurgeError::Interrupted { point })`.
    pub fn check(&self, point: &'static str) -> Result<(), OrxPurgeError> {
        if self.is_enabled(point) {
            Err(OrxPurgeError::Interrupted { point })
        } else {
            Ok(())
        }
    }
}

/// Macro to check runtime failpoint.
#[cfg(any(test, feature = "fault-injection"))]
#[macro_export]
macro_rules! purge_failpoint {
    ($failpoints:expr, $name:expr) => {
        if let Some(fps) = $failpoints {
            fps.check($name)?;
        }
    };
}

#[cfg(not(any(test, feature = "fault-injection")))]
#[macro_export]
macro_rules! purge_failpoint {
    ($failpoints:expr, $name:expr) => {};
}
