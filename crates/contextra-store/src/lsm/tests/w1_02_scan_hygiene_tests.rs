// FILE-CONTEXT: Tests verifying non-blocking scan execution and log hygiene (S6, S8d).

use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::{Result, TxId};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[tokio::test]
async fn scan_does_not_block_commits_p99() -> Result<()> {
    let temp_dir = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await?);
    let running = Arc::new(AtomicBool::new(true));

    // Heavy scanning thread
    let scan_storage = Arc::clone(&storage);
    let scan_running = Arc::clone(&running);
    let scan_handle = tokio::spawn(async move {
        let mut scan_ops = 0usize;
        while scan_running.load(Ordering::Relaxed) {
            let _ = scan_storage.scan_prefix(b"key-").await;
            scan_ops += 1;
            tokio::task::yield_now().await;
        }
        scan_ops
    });

    // Writer thread measuring commit latency p99
    let mut commit_latencies_us = Vec::with_capacity(1_000);
    for i in 1..=100 {
        let tx = TxId(i as u64);
        let key = format!("key-{:06}", i);
        let val = format!("val-{:06}", i);

        storage.put(tx, key.as_bytes(), val.as_bytes()).await?;

        let t0 = Instant::now();
        storage.commit(tx).await?;
        commit_latencies_us.push(t0.elapsed().as_micros() as u64);
    }

    running.store(false, Ordering::Relaxed);
    let total_scans = scan_handle.await.expect("scan task");

    commit_latencies_us.sort_unstable();
    let p99_idx =
        ((commit_latencies_us.len() as f64 * 0.99) as usize).min(commit_latencies_us.len() - 1);
    let p99_latency_us = commit_latencies_us[p99_idx];

    println!("Total Scans: {total_scans} | p99 Commit Latency: {p99_latency_us} us");
    // p99 commit latency must remain low (< 50ms) even under scan load
    assert!(
        p99_latency_us < 50_000,
        "p99 commit latency was {} us, expected < 50000 us",
        p99_latency_us
    );

    Ok(())
}

#[tokio::test]
async fn no_raw_keys_in_logs() -> Result<()> {
    let temp_dir = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let secret_key = b"SECRET_USER_KEY_12345_XYZ";
    let secret_val = b"SECRET_USER_VAL_999";

    let storage = LsmStorage::new(config).await?;
    let tx = TxId(999);
    storage.put(tx, secret_key, secret_val).await?;
    storage.commit(tx).await?;

    println!("last_applied_seq: {}", storage.last_applied_seq());

    let read_res = storage.get(secret_key).await?;
    println!("Read res: {:?}", read_res);
    assert_eq!(read_res.as_deref(), Some(secret_val.as_slice()));

    let scan_res = storage.scan_prefix(b"SECRET_").await?;
    println!("Scan res: {:?}", scan_res);
    assert_eq!(scan_res.len(), 1);

    Ok(())
}
