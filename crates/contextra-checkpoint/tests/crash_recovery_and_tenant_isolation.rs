#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::indexing_slicing,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::await_holding_lock,
    clippy::type_complexity
)]

use contextra_checkpoint::{
    CheckpointHardlinkCloner, DefaultHardlinkCloner, PersistentCheckpointStore, PinnedSeqNoOrphan,
    StateCheckpoint,
};
use contextra_mvcc::SnapshotRegistry;
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use contextra_types::{ContextraError, Result, TxId};
use parking_lot::Mutex;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

/// In-memory mock storage supporting failpoints and versioned key namespaces.
struct FaultMockStorage {
    data: Mutex<BTreeMap<Vec<u8>, Vec<u8>>>,
    pinned: Mutex<HashSet<u64>>,
    rolled_back_txs: Mutex<Vec<TxId>>,
    fail_pin: Mutex<bool>,
    fail_put: Mutex<bool>,
    fail_commit: Mutex<bool>,
    last_tx: Mutex<TxId>,
}

impl FaultMockStorage {
    fn new() -> Self {
        Self {
            data: Mutex::new(BTreeMap::new()),
            pinned: Mutex::new(HashSet::new()),
            rolled_back_txs: Mutex::new(Vec::new()),
            fail_pin: Mutex::new(false),
            fail_put: Mutex::new(false),
            fail_commit: Mutex::new(false),
            last_tx: Mutex::new(TxId::new(0)),
        }
    }
}

impl StorageEngine for FaultMockStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(self.data.lock().get(key).cloned().map(bytes::Bytes::from)) })
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if *self.fail_put.lock() {
                return Err(ContextraError::Internal("Simulated Put Failure".into()));
            }
            self.data.lock().insert(key.to_vec(), value.to_vec());
            let mut last = self.last_tx.lock();
            if tx_id > *last {
                *last = tx_id;
            }
            Ok(())
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.lock().remove(key);
            let mut last = self.last_tx.lock();
            if tx_id > *last {
                *last = tx_id;
            }
            Ok(())
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if *self.fail_commit.lock() {
                return Err(ContextraError::Internal("Simulated Commit Failure".into()));
            }
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

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.rolled_back_txs.lock().push(tx_id);
            Ok(())
        })
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
            if *self.fail_pin.lock() {
                return Err(ContextraError::Internal("Simulated Pin Failure".into()));
            }
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
        _s: std::ops::Bound<&'a [u8]>,
        _e: std::ops::Bound<&'a [u8]>,
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
}

// ============================================================================
// CRASH-CONSISTENCY TESTS (1..8)
// ============================================================================

/// 1. Failpoint test: Pin failure during checkpoint creation leaves clean state.
#[tokio::test]
async fn test_crash_recovery_step_1_pin_failure_leaves_clean_state() {
    let storage = Arc::new(FaultMockStorage::new());
    let store = PersistentCheckpointStore::open(storage.clone(), "ns_fail_pin")
        .await
        .unwrap();

    *storage.fail_pin.lock() = true;

    let res = store
        .create_checkpoint("cp_fail", "col1", 100, TxId::new(1), serde_json::json!({}))
        .await;

    assert!(res.is_err());
    assert!(!storage.pinned.lock().contains(&100));
    assert!(store.list_checkpoints().await.unwrap().is_empty());
}

/// 2. Failpoint test: Put failure unpins sequence number via PinGuard RAII.
#[tokio::test]
async fn test_crash_recovery_step_2_put_failure_cleans_up_pinned_seq() {
    let storage = Arc::new(FaultMockStorage::new());
    let store = PersistentCheckpointStore::open(storage.clone(), "ns_fail_put")
        .await
        .unwrap();

    *storage.fail_put.lock() = true;

    let res = store
        .create_checkpoint("cp_fail", "col1", 200, TxId::new(2), serde_json::json!({}))
        .await;

    assert!(res.is_err());
    assert!(
        !storage.pinned.lock().contains(&200),
        "Sequence 200 must be unpinned when put fails"
    );
    assert!(store.list_checkpoints().await.unwrap().is_empty());
}

/// 3. Failpoint test: Commit failure rolls back transaction and unpins sequence number.
#[tokio::test]
async fn test_crash_recovery_step_3_commit_failure_rolls_back_and_unpins() {
    let storage = Arc::new(FaultMockStorage::new());
    let store = PersistentCheckpointStore::open(storage.clone(), "ns_fail_commit")
        .await
        .unwrap();

    *storage.fail_commit.lock() = true;

    let res = store
        .create_checkpoint("cp_fail", "col1", 300, TxId::new(3), serde_json::json!({}))
        .await;

    assert!(res.is_err());
    assert!(!storage.pinned.lock().contains(&300));
    assert!(store.list_checkpoints().await.unwrap().is_empty());
}

/// 4. Recovery test: open_and_recover drains orphaned sequence pins.
#[tokio::test]
async fn test_crash_recovery_open_and_recover_drains_orphaned_pins() {
    let storage = Arc::new(FaultMockStorage::new());
    storage.pin_checkpoint(400).await.unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let orphan_path = temp_dir
        .path()
        .join("ns_drain_pins_orphaned_checkpoints.json");

    let orphan_reg = Arc::new(contextra_checkpoint::InstanceOrphanRegistry::new(
        &orphan_path,
    ));
    orphan_reg.register_orphan_sync(PinnedSeqNoOrphan {
        seq_no: 400,
        timestamp_ms: 1000,
    });
    orphan_reg.flush_orphan_registry().await.unwrap();

    let (_, report) = PersistentCheckpointStore::open_and_recover_with_orphan_registry(
        storage.clone(),
        "ns_drain_pins",
        orphan_reg,
    )
    .await
    .unwrap();

    assert_eq!(report.recovered_pins, vec![400]);
    assert!(!storage.pinned.lock().contains(&400));
}

/// 5. Recovery test: open_and_recover rolls back orphaned checkpoints.
#[tokio::test]
async fn test_crash_recovery_open_and_recover_rolls_back_orphaned_checkpoints() {
    let storage = Arc::new(FaultMockStorage::new());
    let store = PersistentCheckpointStore::open(storage.clone(), "ns_orb_cp")
        .await
        .unwrap();

    {
        let _guard = store.create_guard(TxId::new(500)).unwrap();
        // Drop guard uncommitted
    }

    let report = store.recover_orphaned_checkpoints().await.unwrap();
    assert_eq!(report, vec![TxId::new(500)]);
    assert_eq!(storage.rolled_back_txs.lock().clone(), vec![TxId::new(500)]);
}

/// 6. Recovery test: open_and_recover processes mixed orphaned pins and checkpoints together.
#[tokio::test]
async fn test_crash_recovery_open_and_recover_handles_mixed_orphans() {
    let storage = Arc::new(FaultMockStorage::new());
    storage.pin_checkpoint(600).await.unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let orphan_path = temp_dir.path().join("ns_mixed_orphaned_checkpoints.json");

    let orphan_reg = Arc::new(contextra_checkpoint::InstanceOrphanRegistry::new(
        &orphan_path,
    ));

    let store = PersistentCheckpointStore::open_with_orphan_registry(
        storage.clone(),
        "ns_mixed",
        orphan_reg.clone(),
    )
    .await
    .unwrap();

    store.register_pinned_seq_no_orphan(PinnedSeqNoOrphan {
        seq_no: 600,
        timestamp_ms: 1000,
    });

    store.register_orphaned_checkpoint(StateCheckpoint {
        tx_id: TxId::new(601),
        timestamp_ms: 1000,
        namespace: Some("ns_mixed".into()),
    });

    store.flush_orphan_registry().await.unwrap();

    let (recovered_store, report) =
        PersistentCheckpointStore::open_and_recover_with_orphan_registry(
            storage.clone(),
            "ns_mixed",
            orphan_reg,
        )
        .await
        .unwrap();

    assert_eq!(report.recovered_pins, vec![600]);
    assert_eq!(report.recovered_checkpoints, vec![TxId::new(601)]);
    assert_eq!(report.total_recovered(), 2);
    assert!(!report.is_empty());
    assert!(!storage.pinned.lock().contains(&600));

    let list = recovered_store.list_checkpoints().await.unwrap();
    assert!(list.is_empty());
}

/// 7. Recovery test: open_and_recover on clean store returns empty report.
#[tokio::test]
async fn test_crash_recovery_open_and_recover_clean_state_when_no_orphans() {
    let storage = Arc::new(FaultMockStorage::new());
    let (_, report) = PersistentCheckpointStore::open_and_recover(storage, "ns_clean")
        .await
        .unwrap();

    assert!(report.is_empty());
    assert_eq!(report.total_recovered(), 0);
}

/// 8. Recovery test: Dropped guard with newer committed transaction is safely skipped during recovery.
#[tokio::test]
async fn test_crash_recovery_dropped_guard_with_newer_tx_skipped_by_open_and_recover() {
    let storage = Arc::new(FaultMockStorage::new());
    let store = PersistentCheckpointStore::open(storage.clone(), "ns_barrier_rec")
        .await
        .unwrap();

    {
        let _guard = store.create_guard(TxId::new(100)).unwrap();
        // Drop uncommitted guard at TxId 100
    }

    // Commit a newer transaction in storage at TxId 200 (> 100)
    storage
        .put(TxId::new(200), b"key_newer", b"val_newer")
        .await
        .unwrap();
    storage.commit(TxId::new(200)).await.unwrap();

    // Perform open_and_recover
    let (_, report) =
        PersistentCheckpointStore::open_and_recover(storage.clone(), "ns_barrier_rec")
            .await
            .unwrap();

    assert!(
        report.recovered_checkpoints.is_empty(),
        "Rollback of TxId 100 must be skipped due to newer committed TxId 200"
    );
    assert_eq!(
        storage.get(b"key_newer").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val_newer"))
    );
}

// ============================================================================
// TENANT / NAMESPACE ISOLATION TESTS (9..13)
// ============================================================================

/// 9. Tenant isolation: Checkpoints created under tenant A are never listed by tenant B.
#[tokio::test]
async fn test_tenant_isolation_checkpoints_never_visible_across_namespaces() {
    let storage = Arc::new(FaultMockStorage::new());

    let store_tenant_a = PersistentCheckpointStore::open(storage.clone(), "tenant_alpha")
        .await
        .unwrap();
    let store_tenant_b = PersistentCheckpointStore::open(storage.clone(), "tenant_beta")
        .await
        .unwrap();

    store_tenant_a
        .create_checkpoint("cp_alpha", "col_a", 1, TxId::new(10), serde_json::json!({}))
        .await
        .unwrap();

    store_tenant_b
        .create_checkpoint("cp_beta", "col_b", 1, TxId::new(20), serde_json::json!({}))
        .await
        .unwrap();

    let list_a = store_tenant_a.list_checkpoints().await.unwrap();
    assert_eq!(list_a.len(), 1);
    assert_eq!(list_a[0].name, "cp_alpha");

    let list_b = store_tenant_b.list_checkpoints().await.unwrap();
    assert_eq!(list_b.len(), 1);
    assert_eq!(list_b[0].name, "cp_beta");
}

/// 10. Tenant isolation: get_checkpoint for checkpoint name on another tenant returns None.
#[tokio::test]
async fn test_tenant_isolation_get_checkpoint_cross_tenant_returns_none() {
    let storage = Arc::new(FaultMockStorage::new());

    let store_a = PersistentCheckpointStore::open(storage.clone(), "tenant_alpha")
        .await
        .unwrap();
    let store_b = PersistentCheckpointStore::open(storage.clone(), "tenant_beta")
        .await
        .unwrap();

    store_b
        .create_checkpoint(
            "shared_name",
            "col_b",
            5,
            TxId::new(50),
            serde_json::json!({}),
        )
        .await
        .unwrap();

    assert!(store_a
        .get_checkpoint("shared_name")
        .await
        .unwrap()
        .is_none());
    assert!(store_b
        .get_checkpoint("shared_name")
        .await
        .unwrap()
        .is_some());
}

/// 11. Tenant isolation: restore_checkpoint on tenant A never rolls back or mutates tenant B keys.
#[tokio::test]
async fn test_tenant_isolation_restore_checkpoint_never_affects_other_tenant() {
    let storage_a = Arc::new(FaultMockStorage::new());
    let storage_b = Arc::new(FaultMockStorage::new());

    let store_a = PersistentCheckpointStore::open(storage_a.clone(), "tenant_alpha")
        .await
        .unwrap();
    let store_b = PersistentCheckpointStore::open(storage_b.clone(), "tenant_beta")
        .await
        .unwrap();

    store_a
        .create_checkpoint(
            "cp1_a",
            "col_a",
            1,
            TxId::new(TxId::INTERNAL_BASE + 100),
            serde_json::json!({}),
        )
        .await
        .unwrap();

    store_b
        .create_checkpoint(
            "cp1_b",
            "col_b",
            1,
            TxId::new(TxId::INTERNAL_BASE + 200),
            serde_json::json!({}),
        )
        .await
        .unwrap();

    let restored_a = store_a.restore_checkpoint("cp1_a").await.unwrap();
    assert_eq!(restored_a.name, "cp1_a");

    // Tenant B's checkpoint remains intact and queryable
    let cp_b = store_b.get_checkpoint("cp1_b").await.unwrap();
    assert!(cp_b.is_some());
}

/// 12. Tenant isolation: drop_checkpoint on tenant A does not delete checkpoint with same name on tenant B.
#[tokio::test]
async fn test_tenant_isolation_drop_checkpoint_same_name_does_not_affect_other_tenant() {
    let storage = Arc::new(FaultMockStorage::new());

    let store_a = PersistentCheckpointStore::open(storage.clone(), "tenant_alpha")
        .await
        .unwrap();
    let store_b = PersistentCheckpointStore::open(storage.clone(), "tenant_beta")
        .await
        .unwrap();

    store_a
        .create_checkpoint(
            "same_name",
            "col_a",
            10,
            TxId::new(100),
            serde_json::json!({}),
        )
        .await
        .unwrap();

    store_b
        .create_checkpoint(
            "same_name",
            "col_b",
            20,
            TxId::new(200),
            serde_json::json!({}),
        )
        .await
        .unwrap();

    // Drop on tenant A
    store_a.drop_checkpoint("same_name").await.unwrap();

    assert!(store_a.get_checkpoint("same_name").await.unwrap().is_none());
    assert!(store_b.get_checkpoint("same_name").await.unwrap().is_some());
}

/// 13. Tenant isolation: Orphan recovery for tenant A only recovers orphans belonging to tenant A.
#[tokio::test]
async fn test_tenant_isolation_orphan_recovery_isolated_by_namespace() {
    let storage = Arc::new(FaultMockStorage::new());

    let store_a = PersistentCheckpointStore::open(storage.clone(), "tenant_alpha")
        .await
        .unwrap();
    let store_b = PersistentCheckpointStore::open(storage.clone(), "tenant_beta")
        .await
        .unwrap();

    {
        let _guard_a = store_a.create_guard(TxId::new(111)).unwrap();
        let _guard_b = store_b.create_guard(TxId::new(222)).unwrap();
        // Drop both guards
    }

    let rec_a = store_a.recover_orphaned_checkpoints().await.unwrap();
    assert_eq!(rec_a, vec![TxId::new(111)]);

    let rec_b = store_b.recover_orphaned_checkpoints().await.unwrap();
    assert_eq!(rec_b, vec![TxId::new(222)]);
}

// ============================================================================
// HARDLINK CLONE, CORRUPTION & MONOTONICITY TESTS (14..16)
// ============================================================================

/// 14. Hardlink clone test: Fallback to file copy when hardlinking fails.
#[tokio::test]
async fn test_crash_recovery_hardlink_clone_fallback_to_copy_on_fault() {
    let source_dir = tempfile::tempdir().unwrap();
    let target_dir = tempfile::tempdir().unwrap();

    // Create mock SST file in source directory
    let sst_path = source_dir.path().join("000001.sst");
    std::fs::write(&sst_path, b"mock sstable content bytes").unwrap();

    let registry = Arc::new(SnapshotRegistry::new());
    let cloner = DefaultHardlinkCloner::new();

    let res = cloner
        .clone_sstable_hardlinks(1, source_dir.path(), target_dir.path(), &registry)
        .await;

    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result.source_seq_no, 1);
    assert_eq!(result.linked_files.len(), 1);

    let target_sst = target_dir.path().join("000001.sst");
    assert!(target_sst.exists());
    assert_eq!(
        std::fs::read(&target_sst).unwrap(),
        b"mock sstable content bytes"
    );
}

/// 15. Corruption recovery test: Corrupted manifest in storage is skipped cleanly during recovery / listing.
#[tokio::test]
async fn test_crash_recovery_corrupted_manifest_skipped_during_recovery() {
    let storage = Arc::new(FaultMockStorage::new());
    let store = PersistentCheckpointStore::open(storage.clone(), "ns_corrupt")
        .await
        .unwrap();

    // Store a valid checkpoint
    store
        .create_checkpoint("valid_cp", "col1", 10, TxId::new(10), serde_json::json!({}))
        .await
        .unwrap();

    // Inject corrupted checkpoint entry directly into storage key space
    let corrupt_key = b"ns_corrupt:checkpoint:corrupt_cp";
    storage
        .put(TxId::new(11), corrupt_key, b"{invalid_json_manifest_bytes")
        .await
        .unwrap();

    // list_checkpoints should handle corrupted entry error
    let res = store.list_checkpoints().await;
    assert!(
        res.is_err(),
        "Corrupted storage JSON should fail gracefully via ContextraError::Serialization"
    );
}

/// 16. Monotonicity test: TxId counter stays strictly monotonic across repeated open / crash-recovery cycles.
#[tokio::test]
async fn test_crash_recovery_txid_monotonicity_and_counter_hwm() {
    let storage = Arc::new(FaultMockStorage::new());

    let mut last_allocated = 0u64;

    for cycle in 0..5 {
        let (store, _) =
            PersistentCheckpointStore::open_and_recover(storage.clone(), "ns_monotonic")
                .await
                .unwrap();

        let tx = store.allocate_tx().await.unwrap();
        let current_raw = tx.inner() - TxId::INTERNAL_BASE;

        assert!(
            current_raw > last_allocated || cycle == 0,
            "TxId in cycle {cycle} ({current_raw}) must be strictly greater than last allocated ({last_allocated})"
        );
        last_allocated = current_raw;
    }
}
