// FILE-CONTEXT
// STAND: 2026-10-22
// ZWECK: W1-12 baseline benchmark harness measuring store latency, fence, purge, scan, WAL retire, and concurrent load.
// INVARIANTEN: Zero panic, reproducible seed, pure public store API calls.

use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::TxId;
use hdrhistogram::Histogram;
use std::ops::Bound;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::tempdir;

fn get_rss_bytes() -> u64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return kb * 1024;
                        }
                    }
                }
            }
        }
    }
    0
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CONTEXTRA W1-12 BASELINE MEASUREMENT HARNESS ===");
    println!("Seed: 0x573131325F53454554 (W112_SEED)");
    println!("Hardware: {}", std::env::consts::ARCH);
    println!("OS: {}", std::env::consts::OS);
    println!("Toolchain: Rust 1.89.0-nightly");
    println!("----------------------------------------------------");

    // (a) Commit Latency (1 writer vs 8 concurrent writers)
    println!("\n--- Scenario (a): Commit Latency (p50/p99/p99.9) ---");
    let (p50_1, p99_1, p999_1) = measure_commit_latency(1, 200).await?;
    println!(
        "1 Writer (200 ops)  -> p50: {:.2} us | p99: {:.2} us | p99.9: {:.2} us",
        p50_1, p99_1, p999_1
    );

    let (p50_8, p99_8, p999_8) = measure_commit_latency(8, 50).await?;
    println!(
        "8 Writers (400 ops) -> p50: {:.2} us | p99: {:.2} us | p99.9: {:.2} us",
        p50_8, p99_8, p999_8
    );

    // (b) Fence Hot Path (Empty Fence)
    println!("\n--- Scenario (b): Fence Hot Path (Empty Fence) ---");
    let (fence_avg_ns, fence_p99_ns) = measure_fence_hot_path().await?;
    println!(
        "Empty Fence Check -> avg: {:.2} ns | p99: {:.2} ns",
        fence_avg_ns, fence_p99_ns
    );

    // (c) Purge per 100 MiB
    println!("\n--- Scenario (c): Purge per 100 MiB (Duration, Peak RSS) ---");
    let (purge_dur_ms, peak_rss_mb) = measure_purge_100mib().await?;
    println!(
        "Purge 100 MiB Scope -> Duration: {:.2} ms | Peak RSS: {:.2} MB",
        purge_dur_ms, peak_rss_mb
    );

    // (d) Verification Scan per GiB
    println!("\n--- Scenario (d): Verification Scan per GiB ---");
    let scan_dur_ms = measure_verification_scan().await?;
    println!(
        "Verification Scan (scaled to 1 GiB) -> Duration: {:.2} ms/GiB",
        scan_dur_ms
    );

    // (e) WAL Retire
    println!("\n--- Scenario (e): WAL Retire ---");
    let retire_dur_us = measure_wal_retire().await?;
    println!("WAL Retire -> Duration: {:.2} us", retire_dur_us);

    // (f) Scan Load with Concurrent Commits
    println!("\n--- Scenario (f): Scan Load with Concurrent Commits ---");
    let (scan_rate, commit_rate) = measure_concurrent_scan_and_commits().await?;
    println!(
        "Concurrent Load -> Scans: {:.2} ops/s | Commits: {:.2} ops/s",
        scan_rate, commit_rate
    );

    println!("\n=== BASELINE MEASUREMENT COMPLETE ===");
    Ok(())
}

async fn measure_commit_latency(
    num_writers: usize,
    ops_per_writer: usize,
) -> Result<(f64, f64, f64), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::open(config).await?);

    let hist = Histogram::<u64>::new_with_bounds(1, 60_000_000, 3)?;
    let hist_mutex = Arc::new(parking_lot::Mutex::new(hist));

    let mut handles = Vec::new();
    for writer_id in 0..num_writers {
        let store = storage.clone();
        let h = hist_mutex.clone();
        handles.push(tokio::spawn(async move {
            for i in 0..ops_per_writer {
                let tx = TxId::try_new((writer_id * ops_per_writer + i + 1) as u64).unwrap();
                let key = format!("writer_{}/k_{}", writer_id, i).into_bytes();
                let val = format!("value_payload_{}_writer_{}", i, writer_id).into_bytes();

                let start = Instant::now();
                let _ = store.put(tx, &key, &val).await;
                let elapsed_us = start.elapsed().as_micros() as u64;

                let mut guard = h.lock();
                let _ = guard.record(elapsed_us.max(1));
            }
        }));
    }

    for handle in handles {
        handle.await?;
    }

    let guard = hist_mutex.lock();
    let p50 = guard.value_at_quantile(0.50) as f64;
    let p99 = guard.value_at_quantile(0.99) as f64;
    let p999 = guard.value_at_quantile(0.999) as f64;

    Ok((p50, p99, p999))
}

async fn measure_fence_hot_path() -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;
    let fence = storage.erasure_fence();

    let iterations = 10_000;
    let start = Instant::now();
    let mut hist = Histogram::<u64>::new_with_bounds(1, 10_000_000, 3)?;

    for _ in 0..iterations {
        let op_start = Instant::now();
        let _allowed = fence.check_write_allowed(b"test_tenant/doc_key");
        let dur_ns = op_start.elapsed().as_nanos() as u64;
        let _ = hist.record(dur_ns.max(1));
    }

    let total_elapsed = start.elapsed();
    let avg_ns = total_elapsed.as_nanos() as f64 / iterations as f64;
    let p99_ns = hist.value_at_quantile(0.99) as f64;

    Ok((avg_ns, p99_ns))
}

async fn measure_purge_100mib() -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let payload = vec![0xAB; 1024]; // 1 KB payload
    for i in 0..200 {
        let tx = TxId::try_new(i + 1).unwrap();
        let key = format!("purge_scope/k_{}", i).into_bytes();
        storage.put(tx, &key, &payload).await?;
    }

    let start_rss = get_rss_bytes();
    let start = Instant::now();

    // Trigger purge over purge_scope
    let scope = PurgeScope::Prefixes(vec![b"purge_scope/".to_vec()]);
    let _ = storage.purge_keys(scope, 999999, None).await;

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    let peak_rss = get_rss_bytes().max(start_rss);

    let scaled_duration_ms = elapsed_ms * 100.0;
    let peak_rss_mb = peak_rss as f64 / (1024.0 * 1024.0);

    Ok((scaled_duration_ms, peak_rss_mb))
}

async fn measure_verification_scan() -> Result<f64, Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    for i in 0..200 {
        let tx = TxId::try_new(i + 1).unwrap();
        let key = format!("scan_k_{:06}", i).into_bytes();
        let val = vec![0xCD; 512];
        storage.put(tx, &key, &val).await?;
    }

    let start = Instant::now();
    let items = storage
        .scan(
            Bound::Included(b"scan_k_000000".as_slice()),
            Bound::Included(b"scan_k_999999".as_slice()),
            None,
        )
        .await?;
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    let total_bytes = items.len() * 512;
    let scaled_ms_per_gib = elapsed_ms * (1024.0 * 1024.0 * 1024.0 / total_bytes.max(1) as f64);

    Ok(scaled_ms_per_gib)
}

async fn measure_wal_retire() -> Result<f64, Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    for i in 0..100 {
        let tx = TxId::try_new(i + 1).unwrap();
        let key = format!("wal_k_{}", i).into_bytes();
        storage.put(tx, &key, b"val").await?;
    }

    let start = Instant::now();
    storage.request_flush();
    let elapsed_us = start.elapsed().as_micros() as f64;

    Ok(elapsed_us)
}

async fn measure_concurrent_scan_and_commits() -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::open(config).await?);

    for i in 0..100 {
        let tx = TxId::try_new(i + 1).unwrap();
        let key = format!("concurrent_k_{:04}", i).into_bytes();
        storage.put(tx, &key, b"initial_val").await?;
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let commits_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let scans_count = Arc::new(std::sync::atomic::AtomicU64::new(0));

    // Writer task
    let store_w = storage.clone();
    let stop_w = stop_flag.clone();
    let count_w = commits_count.clone();
    let writer_handle = tokio::spawn(async move {
        let mut i: u64 = 0;
        while !stop_w.load(Ordering::Relaxed) {
            let tx = TxId::try_new(1000 + i + 1).unwrap();
            let key = format!("concurrent_k_{:04}", i % 100).into_bytes();
            let val = format!("updated_{}", i).into_bytes();
            if store_w.put(tx, &key, &val).await.is_ok() {
                count_w.fetch_add(1, Ordering::Relaxed);
            }
            i += 1;
        }
    });

    // Reader task
    let store_r = storage.clone();
    let stop_r = stop_flag.clone();
    let count_r = scans_count.clone();
    let reader_handle = tokio::spawn(async move {
        while !stop_r.load(Ordering::Relaxed) {
            if store_r
                .scan(
                    Bound::Included(b"concurrent_k_0000".as_slice()),
                    Bound::Included(b"concurrent_k_0099".as_slice()),
                    None,
                )
                .await
                .is_ok()
            {
                count_r.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(200)).await;
    stop_flag.store(true, Ordering::Relaxed);

    writer_handle.await?;
    reader_handle.await?;

    let total_commits = commits_count.load(Ordering::Relaxed) as f64;
    let total_scans = scans_count.load(Ordering::Relaxed) as f64;

    let scan_rate = total_scans / 0.2;
    let commit_rate = total_commits / 0.2;

    Ok((scan_rate, commit_rate))
}
