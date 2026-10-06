#![allow(clippy::unwrap_used, deprecated)]

use contextra_checkpoint::{
    CheckpointRegistry, InstanceOrphanRegistry, OrphanRegistry, PersistentCheckpointStore,
};
use contextra_core::SnapshotRegistry;
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use contextra_types::{ContextraError, Result, TxId};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

struct ClosureMockStorage {
    data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
    pinned: Mutex<HashSet<u64>>,
    last_tx: Mutex<TxId>,
}

impl ClosureMockStorage {
    fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
            pinned: Mutex::new(HashSet::new()),
            last_tx: Mutex::new(TxId::new(0)),
        }
    }

    fn pin_manually(&self, seq_no: u64) {
        self.pinned.lock().insert(seq_no);
    }

    fn is_pinned(&self, seq_no: u64) -> bool {
        self.pinned.lock().contains(&seq_no)
    }
}

impl StorageEngine for ClosureMockStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(self.data.lock().get(key).cloned().map(bytes::Bytes::from)) })
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.lock().insert(key.to_vec(), value.to_vec());
            let mut last = self.last_tx.lock();
            if tx_id > *last {
                *last = tx_id;
            }
            Ok(())
        })
    }

    fn delete<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.lock().remove(key);
            Ok(())
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut last = self.last_tx.lock();
            if tx_id > *last {
                *last = tx_id;
            }
            Ok(())
        })
    }

    fn rollback<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move {
            Ok(StorageStats {
                num_segments: 0,
                total_size_bytes: 0,
                memtable_size_bytes: 0,
            })
        })
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.pinned.lock().insert(seq_no);
            Ok(())
        })
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.pinned.lock().remove(&seq_no);
            Ok(())
        })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        _seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.get(key).await })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(0) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(*self.last_tx.lock()) })
    }

    fn scan<'a>(
        &'a self,
        _start: std::ops::Bound<&'a [u8]>,
        _end: std::ops::Bound<&'a [u8]>,
        _: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let data = self.data.lock();
            Ok(data
                .iter()
                .filter(|(k, _)| k.starts_with(prefix))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect())
        })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        _seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { self.scan_prefix(prefix).await })
    }
}

/// Tests `InstanceOrphanRegistry::register_orphan` and `InstanceOrphanRegistry::recover_and_clean`.
#[tokio::test]
async fn test_instance_orphan_registry_register_and_recover_closure() -> Result<()> {
    let storage = ClosureMockStorage::new();
    let registry = InstanceOrphanRegistry::new("");

    let test_pin = 777_111_u64;
    storage.pin_manually(test_pin);
    assert!(storage.is_pinned(test_pin));

    registry
        .register_orphan(test_pin)
        .map_err(ContextraError::Io)?;
    assert_eq!(registry.get_orphan_pins().len(), 1);
    assert_eq!(registry.get_orphan_pins()[0].seq_no, test_pin);

    let recovered = registry.recover_and_clean(&storage).await?;
    assert_eq!(recovered, vec![test_pin]);
    assert!(
        !storage.is_pinned(test_pin),
        "Storage must be unpinned after InstanceOrphanRegistry recovery"
    );
    assert!(
        registry.get_orphan_pins().is_empty(),
        "Orphan registry pins must be empty after recovery"
    );

    Ok(())
}

/// Tests `OrphanRegistry::register_orphan` and `OrphanRegistry::recover_and_clean`.
#[tokio::test]
async fn test_deprecated_orphan_registry_register_and_recover_closure() -> Result<()> {
    let storage = ClosureMockStorage::new();
    let registry = OrphanRegistry::new("");

    let test_pin = 888_222_u64;
    storage.pin_manually(test_pin);
    assert!(storage.is_pinned(test_pin));

    registry
        .register_orphan(test_pin)
        .map_err(ContextraError::Io)?;
    assert_eq!(registry.get_orphans(), vec![test_pin]);

    let recovered = registry.recover_and_clean(&storage).await?;
    assert_eq!(recovered, vec![test_pin]);
    assert!(
        !storage.is_pinned(test_pin),
        "Storage must be unpinned after OrphanRegistry recovery"
    );
    assert!(
        registry.get_orphans().is_empty(),
        "Orphan registry must be empty after recovery"
    );

    Ok(())
}

/// Tests `CheckpointRegistry` trait methods `register_orphan` and `create_hardlink_clone` on `PersistentCheckpointStore`.
#[tokio::test]
async fn test_checkpoint_registry_trait_closure_wiring() -> Result<()> {
    let source_dir = TempDir::new().map_err(ContextraError::Io)?;
    let target_dir = TempDir::new().map_err(ContextraError::Io)?;

    let sst_file = source_dir.path().join("000001.sst");
    fs::write(&sst_file, b"closure_sst_content").map_err(ContextraError::Io)?;

    let storage = Arc::new(ClosureMockStorage::new());
    let store = Arc::new(PersistentCheckpointStore::new(
        storage.clone(),
        "j21_closure_ns",
    )?);
    let registry: Arc<dyn CheckpointRegistry> = store.clone();

    let test_pin = 999_333_u64;
    storage.pin_manually(test_pin);

    // Call register_orphan via CheckpointRegistry trait
    registry
        .register_orphan(test_pin)
        .map_err(ContextraError::Io)?;

    // Recover via trait
    let recovered = registry.recover_orphaned_pins().await?;
    assert!(recovered.contains(&test_pin));
    assert!(!storage.is_pinned(test_pin));

    // Call create_hardlink_clone via CheckpointRegistry trait
    let snap_reg = Arc::new(SnapshotRegistry::new());
    let clone_res = registry
        .create_hardlink_clone(100, source_dir.path(), target_dir.path(), &snap_reg)
        .await?;

    assert_eq!(clone_res.source_seq_no, 100);
    assert_eq!(clone_res.linked_files.len(), 1);
    assert!(target_dir.path().join("000001.sst").exists());

    Ok(())
}
