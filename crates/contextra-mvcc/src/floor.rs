//! `GcFloor` calculation for tombstone garbage collection watermark.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::snapshot::SnapshotRegistry;
use crate::tx_buffer::TxBuffer;

/// Untergrenze für GC.
pub trait SnapshotFloor: Send + Sync {
    /// Untergrenze für GC.
    fn floor(&self) -> u64;
}

/// Calculator for the MVCC garbage collection sequence floor.
///
/// Ensures tombstone GC never purges entries at or above the floor sequence number.
#[derive(Debug)]
pub struct GcFloor<T: Clone> {
    registry: Arc<SnapshotRegistry>,
    tx_buffer: Option<Arc<TxBuffer<T>>>,
    last_applied: Arc<AtomicU64>,
}

impl<T: Clone> GcFloor<T> {
    /// Creates a new `GcFloor` evaluator.
    pub fn new(
        registry: Arc<SnapshotRegistry>,
        tx_buffer: Option<Arc<TxBuffer<T>>>,
        last_applied: Arc<AtomicU64>,
    ) -> Self {
        Self {
            registry,
            tx_buffer,
            last_applied,
        }
    }

    /// Calculates the GC sequence floor under a unified evaluation contract.
    ///
    /// The floor is computed as:
    /// `min(registry.min_active_seqno(), tx_buffer.min_read_snapshot(), last_applied.load(Acquire))`
    ///
    /// If no active readers or transaction reads exist, `last_applied` serves as the floor.
    pub fn floor(&self) -> u64 {
        let registry_min = self.registry.min_active_seqno();
        let tx_buffer_min = self
            .tx_buffer
            .as_ref()
            .and_then(|tb| tb.min_read_snapshot())
            .unwrap_or(u64::MAX);
        let last_applied = self.last_applied.load(Ordering::Acquire);

        registry_min.min(tx_buffer_min).min(last_applied)
    }
}
