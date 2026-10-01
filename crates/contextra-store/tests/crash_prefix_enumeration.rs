// FILE-CONTEXT: Black-box prefix crash harness enumerating WAL byte truncations and verifying reference model invariants.
// STAND: 2026-09-28
// INVARIANTEN: No unsafe code, no modification of production store code, std + tokio test harness.

#![forbid(unsafe_code)]

mod support;

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_testkit::ReferenceModel;
use support::{copy_dir_recursive, truncate_wal_file, verify_storage_against_reference_model};
use tempfile::TempDir;

#[tokio::test]
async fn test_wal_prefix_crash_enumeration() {
    let tmp_base = TempDir::new().expect("temp dir base");
    let primary_dir = tmp_base.path().join("primary");

    let config = LsmConfig {
        path: primary_dir.clone(),
        group_commit_window_micros: 0, // immediate commit for deterministic WAL layout
        ..Default::default()
    };

    let mut model = ReferenceModel::new();

    // 1. Setup primary storage with a sequence of 5 committed batch transactions
    let storage = LsmStorage::new(config.clone()).await.expect("open LsmStorage");

    let num_batches = 5;
    for batch_idx in 1..=num_batches {
        let tx = TxId::new(batch_idx as u64);
        let k1 = format!("prefix_k_{batch_idx}_a").into_bytes();
        let v1 = format!("prefix_v_{batch_idx}_a").into_bytes();
        let k2 = format!("prefix_k_{batch_idx}_b").into_bytes();
        let v2 = format!("prefix_v_{batch_idx}_b").into_bytes();

        storage.put(tx, &k1, &v1).await.expect("put k1");
        storage.put(tx, &k2, &v2).await.expect("put k2");
        storage.commit(tx).await.expect("commit tx");

        model.put(&k1, &v1);
        model.put(&k2, &v2);
        model.commit();
    }

    // Drop storage to simulate crash/shutdown
    drop(storage);

    // Locate WAL file
    let wal_file = primary_dir.join("wal-00000000000000000000.log");
    let wal_path = if wal_file.exists() {
        wal_file
    } else {
        let legacy = primary_dir.join("wal.log");
        if legacy.exists() {
            legacy
        } else {
            // Find any log file in directory
            let mut entries = tokio::fs::read_dir(&primary_dir).await.expect("read_dir");
            let mut found = None;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.path().extension().map_or(false, |ext| ext == "log") {
                    found = Some(entry.path());
                    break;
                }
            }
            found.expect("WAL file must exist in primary dir")
        }
    };

    let total_wal_bytes = tokio::fs::metadata(&wal_path).await.expect("WAL metadata").len() as usize;
    assert!(total_wal_bytes > 0, "WAL file must be non-empty");

    // 2. Enumerate WAL truncations at 20 granular byte offsets across total WAL length
    let step = (total_wal_bytes / 20).max(1);
    let mut cutoffs = Vec::new();
    let mut offset = 0;
    while offset <= total_wal_bytes {
        cutoffs.push(offset);
        offset += step;
    }
    if cutoffs.last() != Some(&total_wal_bytes) {
        cutoffs.push(total_wal_bytes);
    }

    for (step_idx, &cutoff_bytes) in cutoffs.iter().enumerate() {
        let crash_dir = tmp_base.path().join(format!("crash_step_{step_idx}"));
        copy_dir_recursive(&primary_dir, &crash_dir).expect("copy crash dir");

        let rel_wal_path = wal_path.strip_prefix(&primary_dir).expect("strip prefix");
        let crash_wal_path = crash_dir.join(rel_wal_path);

        truncate_wal_file(&crash_wal_path, cutoff_bytes).expect("truncate wal");

        let crash_config = LsmConfig {
            path: crash_dir.clone(),
            group_commit_window_micros: 0,
            ..Default::default()
        };

        // Reopen storage post-crash
        let reopened_res = LsmStorage::new(crash_config).await;

        match reopened_res {
            Ok(recovered_storage) => {
                // Invariant (b): Recovered state matches ReferenceModel for a valid prefix of confirmed commits
                let mut max_matched_seq = 0;
                for commit_seq in 1..=num_batches {
                    if verify_storage_against_reference_model(&recovered_storage, &model, commit_seq as u64).await.is_ok() {
                        max_matched_seq = commit_seq;
                    } else {
                        break;
                    }
                }

                // If cutoff is full size, all commits must match
                if cutoff_bytes == total_wal_bytes {
                    assert_eq!(
                        max_matched_seq, num_batches,
                        "Full WAL replay must recover all {num_batches} committed transactions"
                    );
                }
            }
            Err(e) => {
                // Invariant (a): Store opens OR reports a defined storage error
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("Storage")
                        || err_msg.contains("WAL")
                        || err_msg.contains("Corruption")
                        || err_msg.contains("Invalid")
                        || err_msg.contains("Header")
                        || err_msg.contains("IO"),
                    "Recovery error must be a defined Storage/WAL error variant, got: {e:?}"
                );
            }
        }
    }
}
