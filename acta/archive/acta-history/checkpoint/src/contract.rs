//! # Crate-Verträge und minimale Umwelt-Schnittstellen
//!
//! Dieses Modul definiert die Kern-Verträge (Traits) und Modelle, die von `acta-checkpoint`
//! benötigt und exportiert werden.

use bytes::Bytes;
use acta_ids::TxId;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub use crate::error::{CheckpointError, CheckpointResult};
pub use crate::fs_seam::{FsSeam, StdFsSeam};
pub use crate::meta::WorkflowState;
pub use crate::snapshot_source::{BoxFuture, SnapshotSource};
pub use crate::vfs::CheckpointVfs;

/// Vollständige, I/O-freie Referenzimplementierung einer `SnapshotSource`.
///
/// Geeignet für Standalone-Einsätze, Tests und In-Memory-Verwendung ohne Speicher-Backend.
/// Typalias für versionierte Byte-Einträge in der In-Memory-Quelle.
pub type VersionedKeyMap = BTreeMap<Vec<u8>, BTreeMap<u64, Option<Vec<u8>>>>;

/// Vollständige, I/O-freie Referenzimplementierung einer `SnapshotSource`.
///
/// Geeignet für Standalone-Einsätze, Tests und In-Memory-Verwendung ohne Speicher-Backend.
#[derive(Debug, Default)]
pub struct InMemorySnapshotSource {
    data: RwLock<VersionedKeyMap>,
    last_seq: AtomicU64,
    last_tx: AtomicU64,
    pinned: RwLock<std::collections::BTreeSet<u64>>,
}

impl InMemorySnapshotSource {
    /// Erzeugt eine neue, leere In-Memory-Quelle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Synchroner Datenzugriff für Tests oder Debugging.
    pub fn get_sync(&self, key: &[u8]) -> Option<Vec<u8>> {
        let guard = self.data.read();
        guard
            .get(key)
            .and_then(|v| v.values().next_back())
            .cloned()
            .flatten()
    }
}

impl SnapshotSource for InMemorySnapshotSource {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, CheckpointResult<Option<Bytes>>> {
        Box::pin(async move {
            let guard = self.data.read();
            let val = guard
                .get(key)
                .and_then(|v| v.values().next_back())
                .cloned()
                .flatten()
                .map(Bytes::from);
            Ok(val)
        })
    }

    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            self.data
                .write()
                .entry(key.to_vec())
                .or_default()
                .insert(tx_id.inner(), Some(value.to_vec()));
            self.last_tx.fetch_max(tx_id.inner(), Ordering::SeqCst);
            Ok(())
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            self.data
                .write()
                .entry(key.to_vec())
                .or_default()
                .insert(tx_id.inner(), None);
            self.last_tx.fetch_max(tx_id.inner(), Ordering::SeqCst);
            Ok(())
        })
    }

    fn commit<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            self.last_seq.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    }

    fn rollback<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            let target_tx = tx_id.inner();
            let mut guard = self.data.write();
            for versions in guard.values_mut() {
                versions.retain(|&v_tx, _| v_tx <= target_tx || v_tx >= TxId::INTERNAL_BASE);
            }
            guard.retain(|_, v| !v.is_empty());
            self.last_tx.store(target_tx, Ordering::SeqCst);
            Ok(())
        })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, CheckpointResult<u64>> {
        Box::pin(async move { Ok(self.last_seq.load(Ordering::SeqCst)) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, CheckpointResult<TxId>> {
        Box::pin(async move { Ok(TxId::new(0)) })
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            self.pinned.write().insert(seq_no);
            Ok(())
        })
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            self.pinned.write().remove(&seq_no);
            Ok(())
        })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, CheckpointResult<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let guard = self.data.read();
            let mut results = Vec::new();
            for (k, versions) in guard.iter() {
                if k.starts_with(prefix) {
                    if let Some(Some(val)) = versions.values().next_back() {
                        results.push((k.clone(), val.clone()));
                    }
                }
            }
            Ok(results)
        })
    }
}
