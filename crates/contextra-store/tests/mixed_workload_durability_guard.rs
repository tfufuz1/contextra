//! Mixed workload timing benchmark and durability guarantee test.
//! Validates group commit latency breakdown and crash/reopen durability parity.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::time::Instant;
use tempfile::TempDir;

#[tokio::test]
async fn test_mixed_workload_benchmark_and_timing_breakdown() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 500,
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await.expect("new storage");

    let num_ops = 5000;
    let start_all = Instant::now();

    for i in 1..=num_ops {
        let tx = TxId::new(i);
        let key = format!("k_{:06}", i).into_bytes();
        let val = format!("v_{:06}", i).into_bytes();

        // Mixed workload: read previous key if i > 1
        if i > 1 {
            let prev_key = format!("k_{:06}", i - 1).into_bytes();
            let read_val = storage.get(&prev_key).await.expect("get succeeds");
            assert!(read_val.is_some(), "previous committed key must be present");
        }

        storage.put(tx, &key, &val).await.expect("put succeeds");
        storage.commit(tx).await.expect("commit succeeds");
    }

    let elapsed = start_all.elapsed();
    let total_secs = elapsed.as_secs_f64();
    let per_commit_micros = (elapsed.as_micros() as f64) / (num_ops as f64);
    let ops_per_sec = (num_ops as f64) / total_secs;

    println!(
        "[GEMESSEN] Mixed 50/50 Benchmark (5,000 single commits + reads, window=500us): Total={:.3}s, Per Commit={:.2}us ({:.2}ms), Throughput={:.1} ops/s",
        total_secs,
        per_commit_micros,
        per_commit_micros / 1000.0,
        ops_per_sec
    );

    // Direct single commit path comparison (window = 0)
    let tmp0 = TempDir::new().expect("temp dir 0");
    let config0 = LsmConfig {
        path: tmp0.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage0 = LsmStorage::new(config0).await.expect("new storage 0");
    let start0 = Instant::now();
    for i in 1..=num_ops {
        let tx = TxId::new(i);
        let key = format!("k_{:06}", i).into_bytes();
        let val = format!("v_{:06}", i).into_bytes();

        if i > 1 {
            let prev_key = format!("k_{:06}", i - 1).into_bytes();
            let read_val = storage0.get(&prev_key).await.expect("get succeeds");
            assert!(read_val.is_some());
        }

        storage0.put(tx, &key, &val).await.expect("put succeeds");
        storage0.commit(tx).await.expect("commit succeeds");
    }
    let elapsed0 = start0.elapsed();
    let total_secs0 = elapsed0.as_secs_f64();
    let per_commit_micros0 = (elapsed0.as_micros() as f64) / (num_ops as f64);
    let ops_per_sec0 = (num_ops as f64) / total_secs0;

    println!(
        "[GEMESSEN] Mixed 50/50 Benchmark (5,000 single commits + reads, window=0us): Total={:.3}s, Per Commit={:.2}us ({:.2}ms), Throughput={:.1} ops/s",
        total_secs0,
        per_commit_micros0,
        per_commit_micros0 / 1000.0,
        ops_per_sec0
    );
}

#[tokio::test]
async fn test_mixed_workload_durability_guarantee() {
    let tmp = TempDir::new().expect("temp dir");
    let path = tmp.path().to_path_buf();

    // Independent Oracle: list of confirmed commits
    let mut oracle_commits: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

    let num_commits = 50;
    for i in 1..=num_commits {
        let key = format!("dur_key_{:04}", i).into_bytes();
        let val = format!("dur_val_{:04}", i).into_bytes();

        // 1. Perform write and commit
        {
            let config = LsmConfig {
                path: path.clone(),
                group_commit_window_micros: 500,
                ..Default::default()
            };
            let storage = LsmStorage::new(config).await.expect("open storage");

            // Read verification of previous committed oracle entries
            for (k, expected_v) in &oracle_commits {
                let actual = storage.get(k).await.expect("get succeeds");
                assert_eq!(
                    actual.as_deref(),
                    Some(expected_v.as_slice()),
                    "Oracle check failed before commit {i} for key {:?}",
                    String::from_utf8_lossy(k)
                );
            }

            let tx = TxId::new(i as u64);
            storage.put(tx, &key, &val).await.expect("put succeeds");
            storage.commit(tx).await.expect("commit succeeds");

            // Record into independent oracle after commit confirmation
            oracle_commits.push((key.clone(), val.clone()));
        }

        // 2. Reopen store after simulated crash/restart and verify oracle invariants
        {
            let config = LsmConfig {
                path: path.clone(),
                group_commit_window_micros: 500,
                ..Default::default()
            };
            let storage = LsmStorage::new(config).await.expect("reopen storage");

            for (k, expected_v) in &oracle_commits {
                let actual = storage.get(k).await.expect("get succeeds");
                assert_eq!(
                    actual.as_deref(),
                    Some(expected_v.as_slice()),
                    "Oracle check failed after reopen at commit {i} for key {:?}",
                    String::from_utf8_lossy(k)
                );
            }
        }
    }
}
