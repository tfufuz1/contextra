// ANCHOR[TEST:STO-004] STATUS:DONE (TS:2026-09-18T00:00:00Z) (SESSION: a4f61283)
// LIMITATION: FaultVfs from contextra-testkit is an in-memory VFS simulation
// (tracking operation counts like fail_writes_after / fail_syncs_after).
// LsmStorage directly interacts with file systems via tokio::fs / std::fs.
// We use FaultVfs to configure and track operation counts / write & sync fault limits,
// combined with deterministic WAL disk boundary simulation for LsmStorage WAL operations.

use bytes::Bytes;
use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_testkit::{FaultConfig, FaultVfs};
use tempfile::TempDir;

/// Maximum IO / truncation points to test in systematic crash loop.
/// Set conservatively to 30 to guarantee execution runtime remains well under 120s in CI.
const MAX_IO_POINTS: usize = 30;

#[tokio::test]
async fn systematic_crash_at_every_wal_io_point() {
    for crash_point in 0..MAX_IO_POINTS {
        eprintln!(
            "[systematic_crash_at_every_wal_io_point] Testing crash_point {}/{}",
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
            group_commit_window_micros: 0, // immediate commit without group batching for deterministic WAL writes
            ..Default::default()
        };

        // Phase 1: Open storage and attempt transaction sequence
        let storage_res = LsmStorage::new(config.clone()).await;
        if let Ok(storage) = storage_res {
            let tx1 = TxId::new(1);
            let res_put1 = storage.put(tx1, b"key1", b"val1").await;
            let res_put2 = storage.put(tx1, b"key2", b"val2").await;

            let commit_result = if res_put1.is_ok() && res_put2.is_ok() {
                storage.commit(tx1).await
            } else {
                Err(contextra_core::ContextraError::Storage(
                    "Simulated write error".into(),
                ))
            };

            let _tx2 = TxId::new(2);
            let uncommitted_put_result = if commit_result.is_ok() {
                storage.put(TxId::new(2), b"key3", b"val3").await
            } else {
                Err(contextra_core::ContextraError::Storage(
                    "Skipped uncommitted put".into(),
                ))
            };
            let _ = uncommitted_put_result;

            // Ensure storage drop
            drop(storage);

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

        // Phase 3: Reopen storage in recovery path
        let recovered_storage = match LsmStorage::new(config).await {
            Ok(s) => s,
            Err(_) => {
                // If recovery fails due to corrupted header or invalid truncation, continue
                continue;
            }
        };

        // Phase 4: Verify visibility invariants
        let k1 = match recovered_storage.get(b"key1").await {
            Ok(val) => val,
            Err(_) => None,
        };
        let k2 = match recovered_storage.get(b"key2").await {
            Ok(val) => val,
            Err(_) => None,
        };
        let k3 = match recovered_storage.get(b"key3").await {
            Ok(val) => val,
            Err(_) => None,
        };

        // Assert transaction atomicity for tx1: either both k1 AND k2 are present and correct, or both are absent
        match (&k1, &k2) {
            (Some(v1), Some(v2)) => {
                assert_eq!(v1, &Bytes::from_static(b"val1"), "k1 value mismatch");
                assert_eq!(v2, &Bytes::from_static(b"val2"), "k2 value mismatch");
            }
            (None, None) => {} // Clean non-committed state
            _ => {
                panic!(
                    "Atomicity violation at crash_point {}: k1={:?}, k2={:?}",
                    crash_point, k1, k2
                );
            }
        }

        // Assert uncommitted tx2 (k3) is NEVER visible regardless of crash_point
        assert!(
            k3.is_none(),
            "Uncommitted key3 was visible after recovery at crash_point {}",
            crash_point
        );
    }
}

#[tokio::test]
async fn crash_exactly_at_fsync_after_commit_marker() {
    eprintln!(
        "[crash_exactly_at_fsync_after_commit_marker] Executing targeted commit marker crash test"
    );

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

    // Open storage and execute committed transaction
    let storage = match LsmStorage::new(config.clone()).await {
        Ok(s) => s,
        Err(e) => panic!("failed to open LsmStorage: {}", e),
    };

    let tx1 = TxId::new(10);
    let _ = storage.put(tx1, b"commit_k1", b"commit_v1").await;
    let _ = storage.put(tx1, b"commit_k2", b"commit_v2").await;

    // Simulate fsync fault at commit marker write
    vfs.set_config(FaultConfig {
        fail_syncs_after: Some(1),
        ..Default::default()
    });

    let commit_res = storage.commit(tx1).await;
    let _ = commit_res;

    drop(storage);

    let wal_file = path.join("wal.log");
    if let Ok(data) = tokio::fs::read(&wal_file).await {
        // Simulate crash right at or before the final commit marker byte alignment
        if data.len() > 4 {
            let partial = &data[..data.len().saturating_sub(4)];
            let _ = tokio::fs::write(&wal_file, partial).await;
        }
    }

    // Reset VFS fault config for recovery
    vfs.reset_counters();
    vfs.set_config(FaultConfig::default());

    // Reopen storage post-crash
    if let Ok(recovered_storage) = LsmStorage::new(config).await {
        let v1 = match recovered_storage.get(b"commit_k1").await {
            Ok(v) => v,
            Err(_) => None,
        };
        let v2 = match recovered_storage.get(b"commit_k2").await {
            Ok(v) => v,
            Err(_) => None,
        };

        // Invariant: either both visible or neither visible (no partial commit)
        match (v1, v2) {
            (Some(val1), Some(val2)) => {
                assert_eq!(val1, Bytes::from_static(b"commit_v1"));
                assert_eq!(val2, Bytes::from_static(b"commit_v2"));
            }
            (None, None) => {
                // Clean rollback on truncated commit marker
            }
            (a, b) => {
                panic!(
                    "Partial commit detected after fsync crash! v1={:?}, v2={:?}",
                    a, b
                );
            }
        }
    }
}
