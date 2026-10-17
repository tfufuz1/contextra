//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-11
//! Zweck: Optionaler Replay-Filter-Hook fuer WAL-Replay.

use std::sync::Arc;

/// Trait for filtering records during WAL replay.
pub trait WalReplayFilter: Send + Sync {
    /// Returns `true` if the record with `key` and transaction ID `tx` should be dropped during replay.
    fn drop_record(&self, key: &[u8], tx: u64) -> bool;
}

#[derive(Clone, Default)]
pub struct WalReplayOptions {
    pub filter: Option<Arc<dyn WalReplayFilter>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WalReplayReport {
    pub records_replayed: u64,
    pub records_dropped_by_filter: u64,
}
