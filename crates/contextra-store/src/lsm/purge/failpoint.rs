// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Failpoint-Infrastruktur und Typen für Purge-Orchestrator Crash-Tests.

use crate::compaction::purge_filter::{PurgeBlocked, PurgeError};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Canonical list of all 14 purge failpoint stage names.
pub const CANONICAL_FAILPOINTS: &[&str] = &[
    "p0_init",
    "after_intent",
    "after_fence",
    "after_flush",
    "mid_compaction_output",
    "after_compaction_before_manifest",
    "after_manifest_before_unlink",
    "mid_dispose_journal",
    "after_unlink",
    "after_wal_callback",
    "after_witness",
    "after_finalize",
    "after_release",
    "after_intent_cleanup",
];

/// Active set of failpoints for testing purge crash consistency.
#[derive(Debug, Clone, Default)]
pub struct OrxFailpointSet {
    active: Arc<Mutex<HashSet<String>>>,
}

impl OrxFailpointSet {
    /// Creates a new empty `OrxFailpointSet`.
    pub fn new() -> Self {
        Self {
            active: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Activates a failpoint by name.
    pub fn activate(&self, name: &str) {
        if let Ok(mut set) = self.active.lock() {
            set.insert(name.to_string());
        }
    }

    /// Checks if a failpoint is active.
    pub fn is_active(&self, name: &str) -> bool {
        if let Ok(set) = self.active.lock() {
            set.contains(name)
        } else {
            false
        }
    }
}

/// Errors occurring during purge orchestration and recovery.
#[derive(Debug, Error)]
pub enum OrxPurgeError {
    #[error("I/O error during purge operation: {0}")]
    Io(#[from] std::io::Error),

    #[error("Purge operation blocked: {0:?}")]
    Blocked(PurgeBlocked),

    #[error("Corrupted purge intent: {0}")]
    CorruptedIntent(String),

    #[error("Failpoint triggered: {0}")]
    FailpointTriggered(String),
}

impl From<PurgeBlocked> for OrxPurgeError {
    fn from(b: PurgeBlocked) -> Self {
        OrxPurgeError::Blocked(b)
    }
}

impl From<PurgeError> for OrxPurgeError {
    fn from(err: PurgeError) -> Self {
        match err {
            PurgeError::Io(e) => OrxPurgeError::Io(e),
            PurgeError::CorruptedIntent(s) => OrxPurgeError::CorruptedIntent(s),
            PurgeError::FailpointTriggered(s) => OrxPurgeError::FailpointTriggered(s),
            other => OrxPurgeError::Blocked(other.into()),
        }
    }
}

impl From<OrxPurgeError> for PurgeError {
    fn from(err: OrxPurgeError) -> Self {
        match err {
            OrxPurgeError::Io(e) => PurgeError::Io(e),
            OrxPurgeError::Blocked(b) => b.into(),
            OrxPurgeError::CorruptedIntent(s) => PurgeError::CorruptedIntent(s),
            OrxPurgeError::FailpointTriggered(s) => PurgeError::FailpointTriggered(s),
        }
    }
}

impl From<OrxPurgeError> for PurgeBlocked {
    fn from(err: OrxPurgeError) -> Self {
        let pe: PurgeError = err.into();
        pe.into()
    }
}

#[macro_export]
macro_rules! purge_failpoint {
    ($name:expr) => {
        #[cfg(feature = "fault-injection")]
        if std::env::var(concat!("PURGE_FAILPOINT_", $name)).is_ok() {
            return Err(
                $crate::lsm::purge::failpoint::OrxPurgeError::FailpointTriggered($name.to_string()),
            );
        }
    };
}
