#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::ops::Bound;
use std::time::Instant;
use tempfile::tempdir;

#[tokio::test]
async fn benchmark_and_verify_range_scan() {
    let dir = tempdir().unwrap();
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::open(config.clone()).await.unwrap();

    // Create 8 SSTables by inserting batch of keys and flushing each
    const SST_COUNT: usize = 8;
    const KEYS_PER_SST: usize = 2000;

    for sst_idx in 0..SST_COUNT {
        let mut entries = Vec::with_capacity(KEYS_PER_SST);

        for k_idx in 0..KEYS_PER_SST {
            let global_idx = sst_idx * KEYS_PER_SST + k_idx;
            let key = format!("k:{:06}", global_idx).into_bytes();
            let val = format!("v:{:06}", global_idx).into_bytes();
            entries.push((key, val));
        }

        let tx_id = TxId(10 + sst_idx as u64);
        storage.put_batch(tx_id, &entries).await.unwrap();
        storage.commit(tx_id).await.unwrap();
        storage.flush().await.unwrap();
    }

    // Also put some keys in active memtable
    let mem_tx = TxId(100);
    storage
        .put(mem_tx, b"k:004150", b"v:override_4150")
        .await
        .unwrap();
    storage.commit(mem_tx).await.unwrap();

    // Correctness Verification
    let start_bound = Bound::Included(b"k:004100".as_slice());
    let end_bound = Bound::Included(b"k:004200".as_slice());

    let results = storage.scan(start_bound, end_bound, None).await.unwrap();

    // k:004100 to k:004200 inclusive is 101 keys
    assert_eq!(results.len(), 101, "Expected 101 keys in narrow range scan");
    assert_eq!(results[0].0, b"k:004100");
    assert_eq!(results[100].0, b"k:004200");

    // Check memtable override
    let override_entry = results.iter().find(|(k, _)| k == b"k:004150").unwrap();
    assert_eq!(override_entry.1, b"v:override_4150");

    // Verify ordering
    for w in results.windows(2) {
        assert!(w[0].0 < w[1].0, "Results must be strictly sorted");
    }

    // Benchmark Narrow Range Scan over 500 iterations
    let iterations = 500;
    let start_time = Instant::now();

    for _ in 0..iterations {
        let res = storage.scan(start_bound, end_bound, None).await.unwrap();
        assert_eq!(res.len(), 101);
    }

    let elapsed = start_time.elapsed();
    let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
    println!(
        "[BENCHMARK BASELINE] Narrow range scan: {:?} total for {} iterations ({:.2} ops/sec, {:.2} µs/op)",
        elapsed,
        iterations,
        ops_per_sec,
        elapsed.as_micros() as f64 / iterations as f64
    );
}
