use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[tokio::test]
async fn test_group_commit_concurrency_stress_200_tasks() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 64 * 1024 * 1024,
        max_ram_mb: 512,
        group_commit_window_micros: 500,
        ..Default::default()
    };

    let storage = Arc::new(
        LsmStorage::new(config.clone())
            .await
            .expect("create storage"),
    );
    let num_tasks = 200u64;

    let start_time = Instant::now();
    let mut set = tokio::task::JoinSet::new();

    for i in 1..=num_tasks {
        let st = Arc::clone(&storage);
        set.spawn(async move {
            let tx = TxId::new(i);
            let key = format!("gc_key_{:04}", i).into_bytes();
            let val = format!("gc_val_{:04}", i).into_bytes();
            st.put(tx, &key, &val).await.expect("put");
            st.commit(tx).await.expect("commit");
            (key, val)
        });
    }

    let mut written = Vec::new();
    while let Some(res) = set.join_next().await {
        let (k, v) = res.expect("task panicked");
        written.push((k, v));
    }

    let elapsed = start_time.elapsed();
    println!("Group-Commit Stress: 200 commits finished in {:?}", elapsed);

    assert_eq!(written.len(), 200);

    // Verify all keys are readable from in-memory storage
    for (k, v) in &written {
        let read_val = storage.get(k).await.expect("get").expect("value exists");
        assert_eq!(&read_val, v);
    }

    drop(storage);

    // Reopen storage and verify byte-for-byte replay equality
    let storage_reopened = LsmStorage::new(config).await.expect("reopen storage");
    for (k, v) in &written {
        let read_val = storage_reopened
            .get(k)
            .await
            .expect("get after reopen")
            .expect("value exists after reopen");
        assert_eq!(&read_val, v);
    }
}

async fn run_latency_benchmark(
    num_writers: usize,
    num_commits_per_writer: usize,
) -> (Duration, Duration, Duration) {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 64 * 1024 * 1024,
        max_ram_mb: 512,
        group_commit_window_micros: 500,
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await.expect("create storage"));

    let mut set = tokio::task::JoinSet::new();

    for w in 0..num_writers {
        let st = Arc::clone(&storage);
        set.spawn(async move {
            let mut latencies = Vec::with_capacity(num_commits_per_writer);
            for c in 0..num_commits_per_writer {
                let tx_num = (w * num_commits_per_writer + c + 1) as u64;
                let tx = TxId::new(tx_num);
                let key = format!("bench_k_{}_{}", w, c).into_bytes();
                let val = format!("bench_v_{}_{}", w, c).into_bytes();
                st.put(tx, &key, &val).await.expect("put");

                let t0 = Instant::now();
                st.commit(tx).await.expect("commit");
                latencies.push(t0.elapsed());
            }
            latencies
        });
    }

    let mut all_latencies = Vec::with_capacity(num_writers * num_commits_per_writer);
    while let Some(res) = set.join_next().await {
        let latencies = res.expect("task panicked");
        all_latencies.extend(latencies);
    }

    all_latencies.sort();
    let n = all_latencies.len();
    let p50 = all_latencies[n / 2];
    let p99 = all_latencies[(n * 99) / 100];
    let avg = all_latencies.iter().sum::<Duration>() / (n as u32);

    (avg, p50, p99)
}

#[tokio::test]
async fn test_group_commit_latency_benchmark_zero_wait() {
    let (avg_1, p50_1, p99_1) = run_latency_benchmark(1, 50).await;
    let (avg_16, p50_16, p99_16) = run_latency_benchmark(16, 20).await;
    let (avg_64, p50_64, p99_64) = run_latency_benchmark(64, 10).await;

    println!("\n=== GROUP COMMIT ZERO-WAIT LATENCY BENCHMARK ===");
    println!(
        "1  writer  (50 commits):  avg = {:?}, p50 = {:?}, p99 = {:?}",
        avg_1, p50_1, p99_1
    );
    println!(
        "16 writers (320 commits): avg = {:?}, p50 = {:?}, p99 = {:?}",
        avg_16, p50_16, p99_16
    );
    println!(
        "64 writers (640 commits): avg = {:?}, p50 = {:?}, p99 = {:?}",
        avg_64, p50_64, p99_64
    );
    println!("================================================\n");

    // Single writer P50 must be reasonable for I/O + fsync without adding the mandatory group_commit_window delay.
    assert!(
        p50_1 < Duration::from_millis(10),
        "Single writer P50 latency ({:?}) should be reasonable (<10ms) without mandatory group_commit_window delay",
        p50_1
    );
}
