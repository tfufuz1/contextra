// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: Failpoint-Makro für Purge-Tests.

use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::Arc;

/// Errors emitted by the Purge Orchestrator.
#[derive(Debug)]
pub enum OrxPurgeError {
    /// Purge execution was interrupted at a specified failpoint to simulate a process crash.
    Interrupted {
        /// Name of the failpoint where execution was interrupted.
        point: &'static str,
    },
    /// Underlying I/O failure.
    Io(std::io::Error),
    /// Purge blocked by pre-flight check or system state.
    Blocked(crate::compaction::purge_filter::PurgeBlocked),
}

impl std::fmt::Display for OrxPurgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Interrupted { point } => write!(f, "Purge interrupted at failpoint: {}", point),
            Self::Io(err) => write!(f, "I/O error during purge: {}", err),
            Self::Blocked(blocked) => write!(f, "Purge blocked: {:?}", blocked),
        }
    }
}

impl std::error::Error for OrxPurgeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for OrxPurgeError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<crate::compaction::purge_filter::PurgeBlocked> for OrxPurgeError {
    fn from(blocked: crate::compaction::purge_filter::PurgeBlocked) -> Self {
        Self::Blocked(blocked)
    }
}

/// Canonical set of all 14 purge failpoint names (Specification Appendix B).
pub const CANONICAL_FAILPOINTS: [&str; 14] = [
    "after_intent",
    "after_fence",
    "after_flush",
    "before_compaction",
    "mid_compaction_output",
    "after_compaction_before_manifest",
    "after_manifest_before_unlink",
    "after_dispose",
    "after_wal_retire",
    "after_witness",
    "after_finalize",
    "after_chain_before_ack",
    "after_release",
    "after_intent_cleanup",
];

/// Thread-safe set of active failpoints passed explicitly to the orchestrator.
#[derive(Debug, Clone, Default)]
pub struct OrxFailpointSet {
    active: Arc<Mutex<HashSet<String>>>,
}

impl OrxFailpointSet {
    /// Creates a new empty failpoint set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Activates a specific failpoint by name.
    pub fn activate(&self, point: impl Into<String>) {
        self.active.lock().insert(point.into());
    }

    /// Returns `true` if `point` is active.
    pub fn is_active(&self, point: &str) -> bool {
        self.active.lock().contains(point)
    }

    /// Clears all active failpoints.
    pub fn clear(&self) {
        self.active.lock().clear();
    }
}

#[cfg(any(test, feature = "fault-injection"))]
#[macro_export]
macro_rules! purge_failpoint {
    ($name:expr) => {
        if std::env::var(concat!("PURGE_FAILPOINT_", $name)).is_ok() {
            return Err(
                $crate::compaction::purge_filter::PurgeBlocked::UnclassifiedFiles { count: 999 },
            );
        }
    };
}

#[cfg(not(test))]
#[macro_export]
macro_rules! purge_failpoint {
    ($name:expr) => {};
}
