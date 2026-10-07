//! Snapshot floor GC interface and implementation.

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

/// Untergrenze für GC. Wird unter demselben Lock gebildet wie Registrierungen.
pub trait SnapshotFloor: Send + Sync {
    /// Untergrenze für GC. Wird unter demselben Lock gebildet wie Registrierungen.
    fn floor(&self) -> u64;
}

/// Garbage collection floor tracking active snapshot bounds.
#[derive(Debug)]
pub struct GcFloor<T: Clone = (Vec<u8>, Vec<u8>)> {
    _registry: Arc<crate::snapshot::SnapshotRegistry>,
    _tx_buffer: Arc<crate::tx_buffer::TxBuffer<T>>,
    _last_applied: Arc<AtomicU64>,
}

impl<T: Clone> GcFloor<T> {
    /// Constructs a new `GcFloor` instance.
    pub fn new(
        registry: Arc<crate::snapshot::SnapshotRegistry>,
        tx_buffer: Arc<crate::tx_buffer::TxBuffer<T>>,
        last_applied: Arc<AtomicU64>,
    ) -> Self {
        Self {
            _registry: registry,
            _tx_buffer: tx_buffer,
            _last_applied: last_applied,
        }
    }
}

impl<T: Clone + Send + Sync> SnapshotFloor for GcFloor<T> {
    fn floor(&self) -> u64 {
        0
    }
}
