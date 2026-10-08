use contextra_core::{SnapshotRegistry, TxBuffer};
use contextra_mvcc::snapshot::SnapshotFloor;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

/// Concrete GC floor implementation combining `SnapshotRegistry`, `TxBuffer`, and `last_applied` watermark.
pub struct GcFloor {
    snapshot_registry: Arc<SnapshotRegistry>,
    tx_buffer: Arc<TxBuffer<(Vec<u8>, Vec<u8>)>>,
    last_applied: Arc<AtomicU64>,
}

impl GcFloor {
    /// Constructs a new `GcFloor`.
    pub fn new(
        snapshot_registry: Arc<SnapshotRegistry>,
        tx_buffer: Arc<TxBuffer<(Vec<u8>, Vec<u8>)>>,
        last_applied: Arc<AtomicU64>,
    ) -> Self {
        Self {
            snapshot_registry,
            tx_buffer,
            last_applied,
        }
    }

    /// Returns the last applied sequence number.
    pub fn last_applied(&self) -> u64 {
        self.last_applied.load(std::sync::atomic::Ordering::Acquire)
    }
}

impl SnapshotFloor for GcFloor {
    fn floor(&self) -> u64 {
        let reg_min = self.snapshot_registry.min_active_seqno();
        let tx_min = self.tx_buffer.min_read_snapshot().unwrap_or(u64::MAX);
        reg_min.min(tx_min)
    }
}
