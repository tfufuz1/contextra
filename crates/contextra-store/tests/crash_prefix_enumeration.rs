// FILE-CONTEXT: Black-box prefix crash harness enumerating WAL byte truncations and verifying reference model invariants.
// STAND: 2026-09-28
// INVARIANTEN: No unsafe code, no modification of production store code, std + tokio test harness.

#![forbid(unsafe_code)]

#[path = "support/mod.rs"]
mod support;

use contextra_ports::StorageEngine;
use contextra_types::TxId;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_testkit::ReferenceModel;
use support::{copy_dir_recursive, truncate_wal_file, verify_storage_against_reference_model};
use tempfile::TempDir;

#[tokio::test]
async fn test_prefix_crash_recovery_enumeration() {
    let seed_dir = TempDir::new().expect("create seed temp dir");
    let seed_path = seed_dir.path();

    let config = LsmConfig {
        path: seed_path.to_path_buf(),
        ..Default::default()
    };

    let ref_model = ReferenceModel::new();

    // Setup initial DB state: 10 transactions across keys
    {
        let storage = LsmStorage::open(config.clone())
            .await
            .expect("open storage for initial state");

        for i in 1..=10 {
            let tx = TxId::new(i);
            let key = format!("k{:02}", i).into_bytes();
            let val = format!("v{:02}", i).into_bytes();

            storage
                .put(tx, &key, &val)
                .await
                .expect("put initial record");
            storage.commit(tx).await.expect("commit initial record");
            ref_model.put(key, val);
        }

        storage.close().await.expect("close initial storage");
    }

    let wal_path = seed_path.join("wal.log");
    let wal_len = std::fs::metadata(&wal_path)
        .expect("metadata of wal.log")
        .len();

    assert!(wal_len > 0, "wal.log must be non-empty");

    let step = 16.max(wal_len / 50);

    for cutoff in (0..wal_len).step_by(step as usize) {
        let test_dir = TempDir::new().expect("create test temp dir");
        let test_path = test_dir.path();

        copy_dir_recursive(seed_path, test_path);

        let test_wal = test_path.join("wal.log");
        truncate_wal_file(&test_wal, cutoff);

        let test_config = LsmConfig {
            path: test_path.to_path_buf(),
            ..Default::default()
        };

        match LsmStorage::open(test_config).await {
            Ok(storage) => {
                verify_storage_against_reference_model(&storage, &ref_model).await;
                let _ = storage.close().await;
            }
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("Storage")
                        || err_msg.contains("WAL")
                        || err_msg.contains("Corruption")
                        || err_msg.contains("Invalid")
                        || err_msg.contains("Header")
                        || err_msg.contains("IO")
                        || err_msg.contains("Format")
                        || err_msg.contains("Manifest"),
                    "Recovery error must be a defined error variant, got: {e:?}"
                );
            }
        }
    }
}
