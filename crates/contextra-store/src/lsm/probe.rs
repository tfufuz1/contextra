//! Store probe implementation for WitStoreProbe interface (T-2026-9220).

use crate::lsm::engine::LsmStorage;
use crate::lsm::fence::ErasureFence;
use crate::manifest::Manifest;
use crate::sstable::SstableReader;
use contextra_crypto::deletion_proof::probe::{WitStoreGeneration, WitStoreProbe};
use contextra_crypto::deletion_proof::DeletionLayer;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Concrete store probe implementation for `LsmStorage`.
pub struct ZeuStoreProbe {
    sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>,
    _manifest: Arc<Manifest>,
    erasure_fence: Arc<ErasureFence>,
    flush_counter: Arc<AtomicU64>,
    segment_counter: Arc<AtomicU64>,
}

impl ZeuStoreProbe {
    /// Creates a new `ZeuStoreProbe` instance attached to `LsmStorage` components.
    pub fn new(
        sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>,
        manifest: Arc<Manifest>,
        erasure_fence: Arc<ErasureFence>,
        flush_counter: Arc<AtomicU64>,
        segment_counter: Arc<AtomicU64>,
    ) -> Self {
        Self {
            sstables,
            _manifest: manifest,
            erasure_fence,
            flush_counter,
            segment_counter,
        }
    }
}

impl WitStoreProbe for ZeuStoreProbe {
    fn current_generation(&self, layer: &DeletionLayer) -> WitStoreGeneration {
        match layer {
            DeletionLayer::LsmMemtable => {
                WitStoreGeneration(self.flush_counter.load(Ordering::Acquire))
            }
            DeletionLayer::SsTableAllLevels | DeletionLayer::Manifest => {
                WitStoreGeneration(self.flush_counter.load(Ordering::Acquire) + self.segment_counter.load(Ordering::Acquire))
            }
            DeletionLayer::WalAllSegments { .. } => {
                WitStoreGeneration(self.segment_counter.load(Ordering::Acquire))
            }
            _ => WitStoreGeneration(0),
        }
    }

    fn scope_write_seq(&self, _scope_hash: &[u8; 32]) -> u64 {
        self.erasure_fence.scope_write_seq()
    }

    fn manifest_file_count(&self) -> Option<u32> {
        let count = self.sstables.try_read().ok()?.len();
        u32::try_from(count).ok()
    }
}

impl LsmStorage {
    /// Returns a thread-safe [`WitStoreProbe`] handle attached to this `LsmStorage` instance.
    pub fn store_probe(&self) -> Arc<dyn WitStoreProbe> {
        Arc::new(ZeuStoreProbe::new(
            Arc::clone(&self.sstables),
            Arc::clone(&self.manifest),
            Arc::clone(&self.erasure_fence),
            Arc::clone(&self.flush_counter),
            Arc::clone(&self.segment_counter),
        ))
    }
}
