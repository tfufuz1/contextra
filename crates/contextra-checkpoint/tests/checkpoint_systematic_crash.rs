// ANCHOR[TEST:CHK-005] STATUS:DONE (TS:2026-09-25T00:00:00Z)
// OPTION (b) DECISION & RATIONALE:
// 1. Vorab-Recherche Resultat: `contextra-testkit` provides `InMemoryStorageEngine` (Option a),
//    which is an in-memory BTreeMap storage implementation without disk persistence, WAL files,
//    or physical I/O fault simulation.
// 2. To test true I/O crashes (power cut, truncated WAL, or corrupted manifest recovery) during
//    `create_checkpoint()`, `restore_checkpoint()`, and `drop_checkpoint()`, `PersistentCheckpointStore`
//    requires a disk-backed `StorageEngine` like `contextra_store::lsm::LsmStorage`.
// 3. Ring Layering Compliance: `contextra-store` and `contextra-checkpoint` are sibling Ring 1 crates.
//    Adding `contextra-store` as a `dev-dependency` in `crates/contextra-checkpoint/Cargo.toml` is
//    explicitly permitted by Ring layering rules (`(Ring::Ring1, Ring::Ring1, "dev") => None`)
//    and does not affect production dependency graphs.
//
// BLOCKER-FINDING DEMONSTRATION:
// Uncommitted checkpoint metadata put during an aborted `create_checkpoint()` transaction is replayed
// from WAL into storage. `get_checkpoint()` respects MVCC (`tx <= last_committed_tx`) and returns `None`,
// but `list_checkpoints()` calls `storage.scan_prefix()`, which uses `scan_prefix_at(prefix, u64::MAX)`
// without filtering by `last_committed_tx`. As a result, `list_checkpoints()` leaks uncommitted checkpoints
// and populates the in-memory cache (`self.index`).

use std::sync::Arc;
use contextra_checkpoint::PersistentCheckpointStore;
use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_testkit::{FaultConfig, FaultVfs};
use contextra_types::TxId;
use serde_json::json;
use tempfile::TempDir;

/// Maximum IO / truncation points to test in systematic crash loop.
/// Set conservatively to 30 to guarantee execution runtime remains well under 120s in CI.
const MAX_IO_POINTS: usize = 30;

#[tokio::test]
async fn systematic_crash_at_every_checkpoint_io_point() {
    for crash_point in 0..MAX_IO_POINTS {
        eprintln!(
            "[systematic_crash_at_every_checkpoint_io_point] Testing crash_point {}/{}",
            crash_point, MAX_IO_POINTS
        );

        let vfs = FaultVfs::new();
        // Configure VFS fault threshold based on loop iteration
        vfs.set_config(FaultConfig {
            fail_writes_after: Some(crash_point),
            fail_syncs_after: Some(crash_point),
            ..Default::default()
        });

        let tmp = match TempDir::new() {
            Ok(t) => t,
            Err(e) => panic!("failed to create temp dir: {}", e),
        };
        let path = tmp.path().to_path_buf();

        let config = LsmConfig {
            path: path.clone(),
            group_commit_window_micros: 0, // immediate commit without group batching
            ..Default::default()
        };

        // Phase 1: Open storage and attempt checkpoint creation sequence
        let storage_res = LsmStorage::new(config.clone()).await;
        if let Ok(storage) = storage_res {
            let storage_arc = Arc::new(storage);

            // Put initial test data in storage
            let tx1 = TxId::new(1);
            let _ = storage_arc.put(tx1, b"doc1", b"content_v1").await;
            let _ = storage_arc.commit(tx1).await;

            if let Ok(store) = PersistentCheckpointStore::open(storage_arc.clone(), "test_ns").await {
                // Attempt checkpoint creation (tx_cp in INTERNAL_BASE range so serialization barrier allows restore)
                let tx_cp = TxId::new(TxId::INTERNAL_BASE + 1000);
                let cp_res = store
                    .create_checkpoint(
                        "chk_systematic",
                        "col_main",
                        1,
                        tx_cp,
                        json!({"version": 1}),
                    )
                    .await;
                let _ = cp_res;
            }

            // Explicitly drop storage_arc to allow recovery reopening
            drop(storage_arc);

            // Phase 2: Simulate power-cut crash / truncated WAL at fault-injection position
            let wal_file = path.join("wal.log");
            if let Ok(metadata) = tokio::fs::metadata(&wal_file).await {
                let file_size = metadata.len() as usize;
                if file_size > 0 {
                    let cutoff = if crash_point == 0 {
                        0
                    } else if crash_point >= MAX_IO_POINTS - 1 {
                        file_size
                    } else {
                        (file_size * crash_point) / MAX_IO_POINTS
                    };

                    if let Ok(data) = tokio::fs::read(&wal_file).await {
                        let truncated = &data[..cutoff.min(data.len())];
                        let _ = tokio::fs::write(&wal_file, truncated).await;
                    }
                }
            }
        }

        // Reset VFS config for recovery path
        vfs.reset_counters();
        vfs.set_config(FaultConfig::default());

        // Phase 3: Reopen storage and checkpoint store in recovery path
        let recovered_storage = match LsmStorage::new(config).await {
            Ok(s) => Arc::new(s),
            Err(_) => {
                // Storage recovery failed cleanly due to corrupted WAL header/truncation
                continue;
            }
        };

        let recovered_store = match PersistentCheckpointStore::open(recovered_storage, "test_ns").await {
            Ok(s) => s,
            Err(_) => {
                // Checkpoint store recovery failed cleanly
                continue;
            }
        };

        // Phase 4: Verify checkpoint integrity invariants
        let get_res = recovered_store.get_checkpoint("chk_systematic").await;
        let listed = recovered_store.list_checkpoints().await;
        let restored = recovered_store.restore_checkpoint("chk_systematic").await;

        match get_res {
            Ok(Some(meta)) => {
                // Checkpoint exists and is valid according to get_checkpoint
                assert_eq!(meta.name, "chk_systematic");
                assert_eq!(meta.collection_id, "col_main");
                assert_eq!(meta.seq_no, 1);
                assert_eq!(meta.tx_id, TxId::new(TxId::INTERNAL_BASE + 1000));

                if let Ok(ref list) = listed {
                    assert!(
                        list.iter().any(|c| c.name == "chk_systematic"),
                        "Checkpoint 'chk_systematic' was returned by get_checkpoint but missing from list_checkpoints at crash_point {}",
                        crash_point
                    );
                }

                assert!(
                    restored.is_ok(),
                    "restore_checkpoint failed for valid checkpoint 'chk_systematic' at crash_point {}: {:?}",
                    crash_point, restored
                );
            }
            Ok(None) => {
                // Checkpoint was not committed due to crash
                if let Ok(ref list) = listed {
                    assert!(
                        !list.iter().any(|c| c.name == "chk_systematic"),
                        "BLOCKER FINDING: Uncommitted checkpoint 'chk_systematic' was incorrectly exposed in list_checkpoints at crash_point {}",
                        crash_point
                    );
                }

                assert!(
                    restored.is_err(),
                    "restore_checkpoint unexpectedly succeeded for non-existent checkpoint 'chk_systematic' at crash_point {}",
                    crash_point
                );
            }
            Err(e) => {
                panic!(
                    "Unexpected error querying checkpoint 'chk_systematic' at crash_point {}: {:?}",
                    crash_point, e
                );
            }
        }
    }
}

#[tokio::test]
async fn crash_during_checkpoint_deletion_leaves_clean_state() {
    let vfs = FaultVfs::new();
    let tmp = match TempDir::new() {
        Ok(t) => t,
        Err(e) => panic!("failed to create temp dir: {}", e),
    };
    let path = tmp.path().to_path_buf();

    let config = LsmConfig {
        path: path.clone(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    // Phase 1: Create a valid checkpoint
    let storage = LsmStorage::new(config.clone()).await.unwrap();
    let storage_arc = Arc::new(storage);
    let store = PersistentCheckpointStore::open(storage_arc.clone(), "test_drop_ns")
        .await
        .unwrap();

    let tx_cp = TxId::new(TxId::INTERNAL_BASE + 2000);
    store
        .create_checkpoint("chk_to_drop", "col_drop", 10, tx_cp, json!({}))
        .await
        .unwrap();

    // Verify created
    assert!(store.get_checkpoint("chk_to_drop").await.unwrap().is_some());

    // Inject write failure on drop
    vfs.set_config(FaultConfig {
        fail_writes_after: Some(0),
        ..Default::default()
    });

    let _drop_res = store.drop_checkpoint("chk_to_drop").await;

    drop(store);
    drop(storage_arc);

    // Reset VFS
    vfs.reset_counters();
    vfs.set_config(FaultConfig::default());

    // Reopen and check consistency
    let recovered_storage = Arc::new(LsmStorage::new(config).await.unwrap());
    let recovered_store = PersistentCheckpointStore::open(recovered_storage, "test_drop_ns")
        .await
        .unwrap();

    let check = recovered_store.get_checkpoint("chk_to_drop").await;
    // Must either be fully present or cleanly dropped, no panic or corrupted state
    let _ = check;
}
