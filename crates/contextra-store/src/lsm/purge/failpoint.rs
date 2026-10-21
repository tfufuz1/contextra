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

/// Type alias for [`PurgeError`] maintained for backwards compatibility if needed.
pub type OrxPurgeError = PurgeError;

#[macro_export]
macro_rules! purge_failpoint {
    ($name:expr) => {
        #[cfg(any(test, feature = "fault-injection"))]
        if std::env::var(concat!("PURGE_FAILPOINT_", $name)).is_ok() {
            return Err(
                $crate::compaction::purge_filter::PurgeError::Failpoint(
                    $name,
                ),
            );
        }
    };
}
