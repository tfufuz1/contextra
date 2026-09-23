#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::orphan::pending_rollback_count;
use contextra_core::{BoxFuture, StorageEngine, StorageStats};
use parking_lot::Mutex;
use std::collections::HashSet;

struct MockStorage {
    data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
    pinned: Mutex<HashSet<u64>>,
    fail_on_put: Mutex<Option<Vec<u8>>>,
    rolled_back_tx: Mutex<Vec<TxId>>,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
            pinned: Mutex::new(HashSet::new()),
            fail_on_put: Mutex::new(None),
            rolled_back_tx: Mutex::new(Vec::new()),
        }
    }
}

impl StorageEngine for MockStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(self.data.lock().get(key).cloned().map(bytes::Bytes::from)) })
    }
    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        _seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.get(key).await })
    }
    fn put<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if let Some(fail_key) = self.fail_on_put.lock().as_ref() {
                if key == fail_key {
                    return Err(ContextraError::Internal("Mock Storage Error".to_string()));
                }
            }
            self.data.lock().insert(key.to_vec(), value.to_vec());
            Ok(())
        })
    }
    fn delete<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.lock().remove(key);
            Ok(())
        })
    }
    fn commit<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn rollback<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.rolled_back_tx.lock().push(tx_id);
            Ok(())
        })
    }
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(0) })
    }
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId::new(0)) })
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
    fn scan<'a>(
        &'a self,
        _s: std::ops::Bound<&'a [u8]>,
        _e: std::ops::Bound<&'a [u8]>,
        _: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }
}

#[tokio::test]
async fn test_create_and_load() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage, "test").unwrap();
    let meta = store
        .create_checkpoint("cp1", "c1", 1, TxId::new(1), serde_json::json!({}))
        .await
        .unwrap(); // unwrap
    let loaded = CheckpointRegistry::load_checkpoint(&store, 1)
        .await
        .unwrap()
        .unwrap(); // unwrap
    assert_eq!(loaded, meta);
}

#[tokio::test]
async fn test_name_uniqueness() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();
    store
        .create_checkpoint("same", "c1", 1, TxId::new(1), serde_json::json!({}))
        .await
        .unwrap(); // unwrap
    store
        .create_checkpoint("same", "c1", 2, TxId::new(2), serde_json::json!({}))
        .await
        .unwrap(); // unwrap
    let all = store.list_checkpoints().await.unwrap(); // unwrap
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].seq_no, 2);
    assert!(!storage.pinned.lock().contains(&1));
    assert!(storage.pinned.lock().contains(&2));
}

#[tokio::test]
async fn test_checkpoint_creation_rollback_on_failure() {
    let storage = Arc::new(MockStorage::new());
    let cp_key = b"test:checkpoint:fail_cp";
    *storage.fail_on_put.lock() = Some(cp_key.to_vec());

    let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();
    let seq_no = 123;

    let res = store
        .create_checkpoint("fail_cp", "c1", seq_no, TxId::new(1), serde_json::json!({}))
        .await;

    assert!(res.is_err());
    assert!(!storage.pinned.lock().contains(&seq_no));
}

#[tokio::test]
async fn test_orphan_recovery_on_startup() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "test_orphan_recovery").unwrap();
    let seq_no = 67890;

    // Pin checkpoint and register orphan directly on store
    storage.pin_checkpoint(seq_no).await.unwrap();
    store.register_pinned_seq_no_orphan(PinnedSeqNoOrphan {
        seq_no,
        timestamp_ms: monotonic_timestamp_ms(),
    });

    assert!(storage.pinned.lock().contains(&seq_no));
    assert!(!store.orphan_registry().get_orphan_pins().is_empty());

    // Recover orphaned pins via store
    let recovered = store.recover_orphaned_pins().await.unwrap();

    assert_eq!(recovered, vec![seq_no]);
    assert!(
        !storage.pinned.lock().contains(&seq_no),
        "Storage sequence number 67890 must be unpinned after recovery"
    );
    assert!(
        store.orphan_registry().get_orphan_pins().is_empty(),
        "Orphan registry must be empty after recovery"
    );
}

#[tokio::test]
async fn test_multi_instance_orphan_isolation() {
    let storage1 = Arc::new(MockStorage::new());
    let store_a = PersistentCheckpointStore::new(storage1, "ns_inst_a").unwrap();

    let storage2 = Arc::new(MockStorage::new());
    let store_b = PersistentCheckpointStore::new(storage2, "ns_inst_b").unwrap();

    // Drop an uncommitted guard in Store A
    {
        let _guard_a = store_a.create_guard(TxId::new(5555)).unwrap();
        // _guard_a drops here without commit or rollback
    }

    // Verify Store A captured orphan checkpoint
    let orphans_a = store_a.get_orphaned_checkpoints();
    assert_eq!(orphans_a.len(), 1);
    assert_eq!(orphans_a[0].tx_id, TxId::new(5555));

    // Verify Store B remained untouched (zero orphans)
    let orphans_b = store_b.get_orphaned_checkpoints();
    assert!(
        orphans_b.is_empty(),
        "Instance B orphan state must not be polluted by Instance A drops"
    );
}

#[tokio::test]
async fn test_pin_guard_unpins_checkpoint_on_storage_write_failure() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "test_pinguard").unwrap();

    let seq_no = 999;
    let cp_key = b"test_pinguard:checkpoint:fail_write_cp";
    *storage.fail_on_put.lock() = Some(cp_key.to_vec());

    let res = store
        .create_checkpoint(
            "fail_write_cp",
            "col1",
            seq_no,
            TxId::new(10),
            serde_json::json!({}),
        )
        .await;

    assert!(
        res.is_err(),
        "Checkpoint creation must fail when storage put fails"
    );

    // Yield execution briefly to allow drop task on Handle::spawn to complete if async
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Verify PinGuard drop unpinned seq_no 999
    assert!(
        !storage.pinned.lock().contains(&seq_no),
        "Sequence number 999 must be unpinned after storage write failure via PinGuard RAII drop"
    );
}

#[tokio::test]
async fn test_pin_before_unpin_invariant_on_failure() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

    // 1. Create first checkpoint successfully
    store
        .create_checkpoint("my_cp", "c1", 1, TxId::new(1), serde_json::json!({}))
        .await
        .unwrap(); // unwrap

    assert!(storage.pinned.lock().contains(&1));

    // 2. Make next save fail
    let cp_key = b"test:checkpoint:my_cp";
    *storage.fail_on_put.lock() = Some(cp_key.to_vec());

    // 3. Try to overwrite with a new checkpoint, which will fail
    let res = store
        .create_checkpoint("my_cp", "c1", 2, TxId::new(2), serde_json::json!({}))
        .await;

    assert!(res.is_err());

    // 4. Verify invariant: old checkpoint (1) must still be pinned!
    assert!(
        storage.pinned.lock().contains(&1),
        "Old checkpoint should still be pinned because save failed"
    );

    // 5. Verify invariant: new checkpoint (2) should be unpinned (rolled back)!
    assert!(
        !storage.pinned.lock().contains(&2),
        "New checkpoint should be unpinned after failure"
    );
}

#[test]
fn test_panic_unwind_triggers_orphan_registration_and_recovery() {
    let storage = Arc::new(MockStorage::new());

    let (store, panic_result) = std::thread::spawn({
        let storage = Arc::clone(&storage);
        move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build Tokio runtime for panic test");

            let store = Arc::new(PersistentCheckpointStore::new(storage, "test_panic").unwrap());
            let store_clone = Arc::clone(&store);

            let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                rt.block_on(async move {
                    let _guard = store_clone.create_guard(TxId::new(7070)).unwrap();
                    // Intentionally trigger panic inside guard scope
                    panic!("Simulated intentional panic between guard creation and commit");
                });
            }));

            (store, panic_res)
        }
    })
    .join()
    .expect("Thread failed to join");

    assert!(panic_result.is_err(), "catch_unwind must capture the panic");

    // Verify orphaned checkpoint is registered in instance store after panic unwind
    assert_eq!(store.get_orphaned_checkpoints().len(), 1);

    // Perform recovery in a fresh runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime for recovery test");

    rt.block_on(async move {
        let recovered = store.recover_orphaned_checkpoints().await.unwrap();
        assert_eq!(recovered, vec![TxId::new(7070)]);

        // Verify transaction was rolled back in storage
        let rolled_back = storage.rolled_back_tx.lock().clone();
        assert_eq!(rolled_back, vec![TxId::new(7070)]);
    });
}

#[tokio::test]
async fn checkpoint_guard_rollback_on_drop() {
    let storage = Arc::new(MockStorage::new());
    let store =
        PersistentCheckpointStore::new(storage.clone(), "test_guard_rollback_on_drop").unwrap();
    store.clear_all_orphaned_checkpoints();

    {
        let _guard = store.create_guard(TxId::new(42)).unwrap(); // unwrap
                                                                 // guard drops here without commit
    }

    let recovered = store.recover_orphaned_checkpoints().await.unwrap();
    assert_eq!(recovered, vec![TxId::new(42)]);

    let rolled_back = storage.rolled_back_tx.lock().clone();
    assert_eq!(rolled_back, vec![TxId::new(42)]);
}

#[tokio::test]
async fn checkpoint_guard_commit_prevents_rollback() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

    let guard = store.create_guard(TxId::new(100)).unwrap(); // unwrap
    let cp = guard.commit().unwrap(); // unwrap
    assert_eq!(cp.tx_id, TxId::new(100));

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(
        storage.rolled_back_tx.lock().is_empty(),
        "Committed guard should not perform rollback"
    );
}

#[tokio::test]
async fn list_checkpoints_empty_initially() {
    use contextra_core::traits::CheckpointCoordinator;
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage, "test").unwrap();

    let list = store.list_named_checkpoints().await.unwrap(); // unwrap
    assert!(list.is_empty());
}

#[tokio::test]
async fn checkpoint_not_found_returns_err() {
    use contextra_core::traits::CheckpointCoordinator;
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage, "test").unwrap();

    let res = store.restore_named_checkpoint("nonexistent").await;
    assert!(matches!(res, Err(ContextraError::CheckpointNotFound)));
}

#[tokio::test]
async fn test_list_named_checkpoints_after_reopen() {
    use contextra_core::traits::CheckpointCoordinator;
    let storage = Arc::new(MockStorage::new());
    {
        let store1 = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();
        store1
            .create_named_checkpoint(
                "cp1",
                "col1",
                1,
                TxId::new(TxId::INTERNAL_BASE + 1),
                serde_json::json!({}),
            )
            .await
            .unwrap(); // unwrap
        store1
            .create_named_checkpoint(
                "cp2",
                "col1",
                2,
                TxId::new(TxId::INTERNAL_BASE + 2),
                serde_json::json!({}),
            )
            .await
            .unwrap(); // unwrap
    }

    let store2 = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();
    let list = store2.list_named_checkpoints().await.unwrap(); // unwrap

    assert_eq!(list.len(), 2);
    let names: Vec<_> = list.into_iter().map(|m| m.name).collect();
    assert_eq!(names, vec!["cp1", "cp2"]);
}

#[tokio::test]
async fn list_checkpoints_cache_matches_storage() {
    use contextra_core::traits::CheckpointCoordinator;
    let storage = Arc::new(MockStorage::new());
    let store1 = Arc::new(PersistentCheckpointStore::new(storage.clone(), "test").unwrap());

    // Create 3 checkpoints
    for i in 1..=3 {
        store1
            .create_named_checkpoint(
                &format!("cp-{i}"),
                "col1",
                i,
                TxId::new(TxId::INTERNAL_BASE + i),
                serde_json::json!({}),
            )
            .await
            .unwrap(); // unwrap
    }

    // Drop and reload the store from same storage
    let store2 = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();
    let list = store2.list_named_checkpoints().await.unwrap(); // unwrap

    assert_eq!(list.len(), 3);
    let names: Vec<_> = list.into_iter().map(|m| m.name).collect();
    assert_eq!(names, vec!["cp-1", "cp-2", "cp-3"]);
}

#[tokio::test]
async fn concurrent_checkpoint_creation_is_safe() {
    use contextra_core::traits::CheckpointCoordinator;
    use tokio::task::JoinSet;

    let storage = Arc::new(MockStorage::new());
    let store = Arc::new(PersistentCheckpointStore::new(storage, "test").unwrap());

    let mut tasks = JoinSet::new();
    for i in 0..8u64 {
        let store = Arc::clone(&store);
        tasks.spawn(async move {
            store
                .create_named_checkpoint(
                    &format!("cp-{i}"),
                    "col1",
                    i,
                    TxId::new(TxId::INTERNAL_BASE + i),
                    serde_json::json!({}),
                )
                .await
        });
    }
    // All must succeed or fail without panicking
    while let Some(res) = tasks.join_next().await {
        let res = res.unwrap(); // unwrap
        if let Err(e) = res {
            println!("Checkpoint creation failed (acceptable): {e}");
        }
    }

    let all = store.list_named_checkpoints().await.unwrap(); // unwrap
    assert_eq!(all.len(), 8);
}

#[test]
fn test_checkpoint_guard_dropped_outside_tokio_runtime() {
    std::thread::spawn(|| {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage, "test").unwrap();

        let initial_skipped = store.skipped_rollback_count();

        {
            let _guard = store.create_guard(TxId::new(999)).unwrap(); // unwrap
                                                                      // _guard drops here at end of inner scope
        }

        assert_eq!(
            store.skipped_rollback_count(),
            initial_skipped + 1,
            "Skipped rollback counter must increment when guard is dropped outside Tokio runtime"
        );
    })
    .join()
    .expect("// expect Thread panic in test_checkpoint_guard_dropped_outside_tokio_runtime");
}

#[tokio::test]
async fn test_auto_rollback_tracking_and_await() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "test_auto_rollback").unwrap();
    store.clear_all_orphaned_checkpoints();

    {
        let _guard = store.create_guard(TxId::new(808)).unwrap(); // unwrap
                                                                  // Drop without commit inside tokio runtime
    }

    let recovered = store.recover_orphaned_checkpoints().await.unwrap();
    assert_eq!(recovered, vec![TxId::new(808)]);

    let rolled_back = storage.rolled_back_tx.lock().clone();
    assert_eq!(rolled_back, vec![TxId::new(808)]);
    assert_eq!(pending_rollback_count(), 0);
}

#[tokio::test]
async fn test_drop_checkpoint_uses_unique_tx_and_unpins() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

    store
        .create_checkpoint("drop_me", "col1", 42, TxId::new(1), serde_json::json!({}))
        .await
        .unwrap(); // unwrap

    assert!(storage.pinned.lock().contains(&42));
    assert!(store.get_checkpoint("drop_me").await.unwrap().is_some()); // unwrap

    store.drop_checkpoint("drop_me").await.unwrap(); // unwrap

    assert!(
        !storage.pinned.lock().contains(&42),
        "Checkpoint seq_no 42 should be unpinned after drop"
    );
    assert!(store.get_checkpoint("drop_me").await.unwrap().is_none()); // unwrap
}

#[tokio::test]
async fn test_next_tx_overflow_returns_err() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage, "test").unwrap();

    store.tx_counter.store(1_000_000, Ordering::SeqCst);

    let res = store.allocate_tx().await;
    assert!(res.is_err());
    if let Err(ContextraError::Internal(msg)) = res {
        assert!(msg.contains("overflow"));
    } else {
        panic!("Expected Internal error on overflow");
    }
}

#[tokio::test]
async fn test_input_validation_empty_and_oversized_names() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage, "test").unwrap();

    // Empty name
    let res = store
        .create_checkpoint("", "col1", 1, TxId::new(1), serde_json::json!({}))
        .await;
    assert!(matches!(res, Err(ContextraError::InvalidInput(_))));

    // Whitespace name
    let res = store
        .create_checkpoint("   ", "col1", 1, TxId::new(1), serde_json::json!({}))
        .await;
    assert!(matches!(res, Err(ContextraError::InvalidInput(_))));

    // Empty collection ID
    let res = store
        .create_checkpoint("cp1", "", 1, TxId::new(1), serde_json::json!({}))
        .await;
    assert!(matches!(res, Err(ContextraError::InvalidInput(_))));

    // Oversized name (> 256 chars)
    let long_name = "a".repeat(257);
    let res = store
        .create_checkpoint(&long_name, "col1", 1, TxId::new(1), serde_json::json!({}))
        .await;
    assert!(matches!(res, Err(ContextraError::InvalidInput(_))));

    // Multibyte unicode name: 256 chars (512+ bytes) accepted, 257 chars rejected
    let unicode_256 = "ä".repeat(256);
    let res_256 = store
        .create_checkpoint(&unicode_256, "col1", 1, TxId::new(1), serde_json::json!({}))
        .await;
    assert!(res_256.is_ok());

    let unicode_257 = "ä".repeat(257);
    let res_257 = store
        .create_checkpoint(&unicode_257, "col1", 1, TxId::new(1), serde_json::json!({}))
        .await;
    assert!(matches!(res_257, Err(ContextraError::InvalidInput(_))));

    // Drop with empty name
    let res = store.drop_checkpoint("").await;
    assert!(matches!(res, Err(ContextraError::InvalidInput(_))));

    // Get with empty name
    let res = store.get_checkpoint("   ").await;
    assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
}

#[allow(non_snake_case)]
#[tokio::test]
async fn create_checkpoint_CASE_unicode_and_multibyte_name() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

    let unicode_name = "Prüfpunkt_1_🚀_日本語";
    let collection_id = "Sammlung_äöü_123";

    store
        .create_checkpoint(
            unicode_name,
            collection_id,
            10,
            TxId::new(101),
            serde_json::json!({"tag": "überprüfen"}),
        )
        .await
        .expect("// expect #[cfg(test)]");

    let fetched = store
        .get_checkpoint(unicode_name)
        .await
        .expect("// expect #[cfg(test)]")
        .expect("// expect #[cfg(test)]");

    assert_eq!(fetched.name, "Prüfpunkt_1_🚀_日本語");
    assert_eq!(fetched.collection_id, "Sammlung_äöü_123");
    assert_eq!(fetched.seq_no, 10);
    assert_eq!(fetched.tx_id, TxId::new(101));
    assert_eq!(fetched.metadata["tag"], "überprüfen");
}

#[allow(non_snake_case)]
#[tokio::test]
async fn create_checkpoint_CASE_exact_max_len_256() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

    let max_name = "a".repeat(256);
    let res = store
        .create_checkpoint(&max_name, "col1", 1, TxId::new(1), serde_json::json!({}))
        .await;
    assert!(res.is_ok(), "256 characters name must be allowed");

    let fetched = store
        .get_checkpoint(&max_name)
        .await
        .expect("// expect #[cfg(test)]");
    assert!(fetched.is_some());
}

#[allow(non_snake_case)]
#[tokio::test]
async fn drop_checkpoint_CASE_nonexistent_returns_ok() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

    let res = store.drop_checkpoint("nonexistent_checkpoint").await;
    assert!(
        res.is_ok(),
        "Dropping a non-existent checkpoint should be idempotent and return Ok(())"
    );
}

#[allow(non_snake_case)]
#[tokio::test]
async fn restore_checkpoint_CASE_not_found_returns_err() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage, "test").unwrap();

    let res = store.restore_checkpoint("missing_cp").await;
    assert!(matches!(res, Err(ContextraError::CheckpointNotFound)));
}

#[allow(non_snake_case)]
#[tokio::test]
async fn list_checkpoints_CASE_corrupted_storage_data_propagates_err() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

    // Put invalid JSON payload into storage under checkpoint namespace format (namespace:checkpoint:name)
    let corrupt_key = b"test:checkpoint:corrupt_cp";
    storage
        .put(TxId::new(1), corrupt_key, b"invalid json bytes{{{")
        .await
        .expect("// expect #[cfg(test)]");

    let res = store.list_checkpoints().await;
    assert!(matches!(res, Err(ContextraError::Serialization(_))));
}

#[tokio::test]
async fn test_get_orphaned_checkpoints_for_namespace() {
    let storage = Arc::new(MockStorage::new());
    let store_a = PersistentCheckpointStore::new(storage.clone(), "ns_a").unwrap();
    let store_b = PersistentCheckpointStore::new(storage.clone(), "ns_b").unwrap();

    {
        let _guard_a1 = store_a.create_guard(TxId::new(1001)).unwrap();
        let _guard_a2 = store_a.create_guard(TxId::new(1002)).unwrap();
        let _guard_b1 = store_b.create_guard(TxId::new(2001)).unwrap();
        // All 3 guards drop here uncommitted
    }

    let orphans_a = store_a.get_orphaned_checkpoints();
    let orphans_b = store_b.get_orphaned_checkpoints();

    assert_eq!(orphans_a.len(), 2);
    assert!(orphans_a
        .iter()
        .all(|cp| cp.namespace.as_deref() == Some("ns_a")));
    let tx_a: Vec<TxId> = orphans_a.iter().map(|cp| cp.tx_id).collect();
    assert!(tx_a.contains(&TxId::new(1001)));
    assert!(tx_a.contains(&TxId::new(1002)));

    assert_eq!(orphans_b.len(), 1);
    assert_eq!(orphans_b[0].tx_id, TxId::new(2001));
    assert_eq!(orphans_b[0].namespace.as_deref(), Some("ns_b"));
}

#[allow(non_snake_case)]
#[tokio::test]
async fn allocate_tx_CASE_parity_with_deprecated_next_tx() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage, "test").unwrap();

    let tx1 = store.allocate_tx().await.expect("// expect #[cfg(test)]");
    #[allow(deprecated)]
    let tx2 = store.next_tx().await.expect("// expect #[cfg(test)]");
    let tx3 = store.allocate_tx().await.expect("// expect #[cfg(test)]");

    assert_eq!(tx1, TxId::new(TxId::INTERNAL_BASE));
    assert_eq!(tx2, TxId::new(TxId::INTERNAL_BASE + 1));
    assert_eq!(tx3, TxId::new(TxId::INTERNAL_BASE + 2));
}

#[tokio::test]
async fn test_store_skipped_rollback_count_and_checkpoint_counter() {
    let storage = Arc::new(MockStorage::new());
    let store = PersistentCheckpointStore::new(storage, "test_skipped").unwrap();

    assert_eq!(store.skipped_rollback_count(), 0);
    assert_eq!(store.checkpoint_guard_skipped_rollback_count(), 0);

    let ts = store.monotonic_timestamp_ms();
    assert!(ts > 0);

    {
        let _guard = store.create_guard(TxId::new(300)).unwrap();
        // Drop without commit or rollback
    }

    assert_eq!(store.skipped_rollback_count(), 1);
    assert_eq!(store.checkpoint_guard_skipped_rollback_count(), 1);
}

#[tokio::test]
async fn test_concurrent_checkpoint_index_atomicity() {
    use std::sync::atomic::AtomicBool;

    let storage = Arc::new(MockStorage::new());
    let store = Arc::new(PersistentCheckpointStore::new(storage, "test_atomicity").unwrap());
    let stop_flag = Arc::new(AtomicBool::new(false));

    let cp_name = "atomic_cp";

    // Spawn writer task continuously creating and dropping checkpoints under the same name
    let writer_store = Arc::clone(&store);
    let writer_stop = Arc::clone(&stop_flag);
    let writer_handle = tokio::spawn(async move {
        let mut seq = 1u64;
        while !writer_stop.load(Ordering::Relaxed) {
            let _create_res = writer_store
                .create_checkpoint(
                    cp_name,
                    "col_atomic",
                    seq,
                    TxId::new(seq),
                    serde_json::json!({"seq": seq}),
                )
                .await;
            tokio::task::yield_now().await;
            let _drop_res = writer_store.drop_checkpoint(cp_name).await;
            tokio::task::yield_now().await;
            seq += 1;
        }
    });

    // Spawn 4 reader tasks continuously reading the checkpoint by name
    let mut reader_handles = Vec::new();
    for _ in 0..4 {
        let reader_store = Arc::clone(&store);
        let reader_stop = Arc::clone(&stop_flag);
        reader_handles.push(tokio::spawn(async move {
            while !reader_stop.load(Ordering::Relaxed) {
                if let Ok(Some(cp)) = reader_store.get_checkpoint(cp_name).await {
                    // Verify that if a checkpoint is returned, its internal name match and seq_no are consistent
                    assert_eq!(cp.name, cp_name);
                    assert!(cp.seq_no > 0);
                    let meta_seq = cp.metadata.get("seq").and_then(|v| v.as_u64());
                    assert_eq!(meta_seq, Some(cp.seq_no));
                }
                tokio::task::yield_now().await;
            }
        }));
    }

    // Run concurrent readers & writers for 200 ms
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    stop_flag.store(true, Ordering::Relaxed);

    writer_handle.await.expect("// expect #[cfg(test)]");
    for handle in reader_handles {
        handle.await.expect("// expect #[cfg(test)]");
    }
}
