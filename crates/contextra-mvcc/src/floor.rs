//! `GcFloor` calculation for tombstone garbage collection watermark.

use crate::snapshot::SnapshotRegistry;
use crate::sync::atomic::{AtomicU64, Ordering};
use crate::sync::Arc;
use crate::tx_buffer::TxBuffer;

/// Untergrenze für GC.
pub trait SnapshotFloor: Send + Sync {
    /// Untergrenze für GC.
    fn floor(&self) -> u64;
}

impl<T: SnapshotFloor + ?Sized> SnapshotFloor for Arc<T> {
    fn floor(&self) -> u64 {
        (**self).floor()
    }
}

impl<T: Send + Sync + Clone> SnapshotFloor for GcFloor<T> {
    fn floor(&self) -> u64 {
        Self::floor(self)
    }
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
        tx_buffer: impl Into<Option<Arc<TxBuffer<T>>>>,
        last_applied: Arc<AtomicU64>,
    ) -> Self {
        Self {
            registry,
            tx_buffer: tx_buffer.into(),
            last_applied,
        }
    }

    /// Returns the last applied sequence number watermark.
    pub fn last_applied(&self) -> u64 {
        self.last_applied.load(Ordering::Acquire)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_floor_without_tx_buffer() {
        let registry = Arc::new(SnapshotRegistry::new());
        let last_applied = Arc::new(AtomicU64::new(100));

        let floor_calc = GcFloor::<Vec<u8>>::new(registry.clone(), None, last_applied.clone());

        assert_eq!(floor_calc.last_applied(), 100);
        assert_eq!(floor_calc.floor(), 100);

        let lease = registry.acquire(|| 50);
        assert_eq!(floor_calc.floor(), 50);

        drop(lease);
        assert_eq!(floor_calc.floor(), 100);
    }

    #[test]
    fn test_gc_floor_with_tx_buffer_direct_arc() {
        let registry = Arc::new(SnapshotRegistry::new());
        let last_applied = Arc::new(AtomicU64::new(200));
        let tx_buffer = Arc::new(TxBuffer::<(Vec<u8>, Vec<u8>)>::new().with_snapshot_registry(registry.clone()));

        // Test passing Arc<TxBuffer> directly without Some wrapper
        let floor_calc = GcFloor::new(registry.clone(), tx_buffer.clone(), last_applied.clone());

        assert_eq!(floor_calc.last_applied(), 200);
        assert_eq!(floor_calc.floor(), 200);

        let lease = registry.acquire(|| 150);
        assert_eq!(floor_calc.floor(), 150);

        drop(lease);
        assert_eq!(floor_calc.floor(), 200);
    }
}
