#![allow(clippy::unwrap_used, deprecated)]

use contextra_checkpoint::{
    register_pinned_seq_no_orphan, PersistentCheckpointStore, PinnedSeqNoOrphan,
};
use contextra_core::SnapshotRegistry;
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use contextra_types::{ContextraError, Result, TxId};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

struct WireTestMockStorage {
    data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
    pinned: Mutex<HashSet<u64>>,
    last_tx: Mutex<TxId>,
}

impl WireTestMockStorage {
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

impl StorageEngine for WireTestMockStorage {
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

/// Tests that `register_pinned_seq_no_orphan` calls `register_orphan` on the global orphan registry,
/// and that `store.recover_orphaned_pins()` invokes `recover_and_clean` to unpin and clear the registered orphan.
#[tokio::test]
async fn test_orphan_registry_register_and_recover_wire() -> Result<()> {
    let storage = Arc::new(WireTestMockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "j21_wire_test")?;

    let test_seq = 888_999_u64;
    storage.pin_manually(test_seq);
    assert!(storage.is_pinned(test_seq));

    // Calling the top-level public function delegates to global_orphan_registry().register_orphan(seq_no)
    register_pinned_seq_no_orphan(PinnedSeqNoOrphan {
        seq_no: test_seq,
        timestamp_ms: 123456,
    });

    // Calling store.recover_orphaned_pins() invokes global_orphan_registry().recover_and_clean(&*storage)
    let recovered = store.recover_orphaned_pins().await?;

    assert!(
        recovered.contains(&test_seq),
        "Recovered pins list must contain the orphaned sequence pin {test_seq}"
    );
    assert!(
        !storage.is_pinned(test_seq),
        "Storage engine must be unpinned for sequence pin {test_seq} after recovery"
    );

    Ok(())
}

/// Tests `PersistentCheckpointStore::create_hardlink_clone` production interface.
#[tokio::test]
async fn test_persistent_checkpoint_store_create_hardlink_clone_wire() -> Result<()> {
    let source_dir = TempDir::new().map_err(ContextraError::Io)?;
    let target_dir = TempDir::new().map_err(ContextraError::Io)?;

    let sst1 = source_dir.path().join("000001.sst");
    let sst2 = source_dir.path().join("000002.sst");
    fs::write(&sst1, b"sst1_data_j21").map_err(ContextraError::Io)?;
    fs::write(&sst2, b"sst2_data_j21").map_err(ContextraError::Io)?;

    let storage = Arc::new(WireTestMockStorage::new());
    let store = PersistentCheckpointStore::new(storage, "j21_clone_test")?;
    let snapshot_registry = Arc::new(SnapshotRegistry::new());

    let clone_res = store
        .create_hardlink_clone(
            500,
            source_dir.path(),
            target_dir.path(),
            &snapshot_registry,
        )
        .await?;

    assert_eq!(clone_res.source_seq_no, 500);
    assert_eq!(clone_res.linked_files.len(), 2);

    let target1 = target_dir.path().join("000001.sst");
    let target2 = target_dir.path().join("000002.sst");

    assert!(target1.exists());
    assert!(target2.exists());
    assert_eq!(
        fs::read(&target1).map_err(ContextraError::Io)?,
        b"sst1_data_j21"
    );
    assert_eq!(
        fs::read(&target2).map_err(ContextraError::Io)?,
        b"sst2_data_j21"
    );

    Ok(())
}
