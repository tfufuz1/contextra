use super::*;
use tempfile::TempDir;

async fn test_storage() -> (LsmStorage, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");
    (storage, tmp)
}

#[tokio::test]
async fn test_delete_prefix_batch_single_tx_buffer_lock() {
    let dir = tempfile::tempdir().unwrap();
    let storage = LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    })
    .await
    .unwrap();

    let tx1 = TxId::new(1);
    for i in 0..10u32 {
        storage
            .put(tx1, format!("pfx:key{}", i).as_bytes(), b"val")
            .await
            .unwrap();
    }
    storage.commit(tx1).await.unwrap();
    storage.flush().await.unwrap();

    let tx2 = TxId::new(2);
    let deleted = storage.delete_prefix(tx2, b"pfx:").await.unwrap();
    assert_eq!(deleted, 10);

    storage.commit(tx2).await.unwrap();
    let remaining = storage.scan_prefix(b"pfx:").await.unwrap();
    assert!(remaining.is_empty(), "All prefixed keys must be deleted");
}

#[tokio::test]
async fn test_flush_during_read_transaction_snapshot_isolation() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key_flush", b"v1").await.expect("put t1");
    storage.commit(tx1).await.expect("commit t1");
    let seq1 = storage.last_seq_no().await.expect("seq1");

    let snap_tx1 = storage.last_tx_id().await.expect("last tx1");

    storage.flush().await.expect("flush tx1");

    let tx2 = TxId::new(2);
    storage.put(tx2, b"key_flush", b"v2").await.expect("put t2");
    storage.commit(tx2).await.expect("commit t2");

    storage.flush().await.expect("flush tx2");

    let val = storage
        .get_at_seq(b"key_flush", seq1)
        .await
        .expect("get_at_seq");
    assert_eq!(val, Some(b"v1".to_vec()));
    assert_eq!(snap_tx1, TxId::new(1));
}

#[tokio::test]
async fn test_concurrent_flush_and_get_at_seq_isolation() {
    let (storage, _tmp) = test_storage().await;
    let storage = Arc::new(storage);

    let tx_base = TxId::new(1);
    storage
        .put(tx_base, b"key_race", b"val_base")
        .await
        .expect("put base");
    storage.commit(tx_base).await.expect("commit base");

    let mut handles = Vec::new();

    let s_writer = Arc::clone(&storage);
    handles.push(tokio::spawn(async move {
        for i in 2..=1000u64 {
            let tx = TxId::new(i);
            let val = format!("val_{i}").into_bytes();
            s_writer.put(tx, b"key_race", &val).await.expect("put loop");
            s_writer.commit(tx).await.expect("commit loop");
            if i % 10 == 0 {
                s_writer.flush().await.expect("flush loop");
            }
        }
    }));

    let s_reader = Arc::clone(&storage);
    handles.push(tokio::spawn(async move {
        for _ in 0..1000 {
            let last_tx = s_reader.last_tx_id().await.expect("last_tx").inner();
            let last_seq = s_reader.last_seq_no().await.expect("last_seq");

            let res = s_reader
                .get_at_seq(b"key_race", last_seq)
                .await
                .expect("get_at_seq");

            if let Some(val_bytes) = res {
                let val_str = String::from_utf8(val_bytes).expect("utf8");
                if let Some(num_str) = val_str.strip_prefix("val_") {
                    if num_str != "base" {
                        let tx_num: u64 = num_str.parse().expect("parse tx num");
                        assert!(
                            tx_num <= last_tx,
                            "MVCC Invariant Violation: Read tx {} higher than snapshot_tx {}",
                            tx_num,
                            last_tx
                        );
                    }
                }
            }
            tokio::task::yield_now().await;
        }
    }));

    for h in handles {
        h.await.expect("task join");
    }
}

#[tokio::test]
async fn test_flush_during_active_snapshot_isolation_stress() {
    let (storage, _tmp) = test_storage().await;
    let storage = Arc::new(storage);

    let tx_base = TxId::new(1);
    storage.put(tx_base, b"snap:key", b"v_1").await.unwrap();
    storage.commit(tx_base).await.unwrap();

    let s_writer = Arc::clone(&storage);
    let writer_handle = tokio::spawn(async move {
        for i in 2..=1000u64 {
            let tx = TxId::new(i);
            let val = format!("v_{i}").into_bytes();
            s_writer.put(tx, b"snap:key", &val).await.unwrap();
            s_writer.commit(tx).await.unwrap();
            if i % 5 == 0 {
                s_writer.flush().await.unwrap();
            }
        }
    });

    let s_reader = Arc::clone(&storage);
    let reader_handle = tokio::spawn(async move {
        for _ in 0..1000 {
            let snapshot_tx = s_reader.last_tx_id().await.unwrap().inner();
            let snapshot_seq = s_reader.last_seq_no().await.unwrap();

            let get_val = s_reader
                .get_at_seq(b"snap:key", snapshot_seq)
                .await
                .unwrap();
            if let Some(bytes) = get_val {
                let val_str = String::from_utf8(bytes).unwrap();
                let tx_num: u64 = val_str.strip_prefix("v_").unwrap().parse().unwrap();
                assert!(
                    tx_num <= snapshot_tx,
                    "MVCC Invariant Violation during flush: read tx {} exceeds snapshot_tx {}",
                    tx_num,
                    snapshot_tx
                );
            }

            let scan_res = s_reader
                .scan_prefix_at(b"snap:", snapshot_seq)
                .await
                .unwrap();
            assert!(!scan_res.is_empty());
            let val_str = String::from_utf8(scan_res[0].1.clone()).unwrap();
            let tx_num: u64 = val_str.strip_prefix("v_").unwrap().parse().unwrap();
            assert!(
                tx_num <= snapshot_tx,
                "MVCC Invariant Violation in scan_prefix_at during flush: read tx {} exceeds snapshot_tx {}",
                tx_num,
                snapshot_tx
            );

            tokio::task::yield_now().await;
        }
    });

    writer_handle.await.unwrap();
    reader_handle.await.unwrap();
}

#[tokio::test]
async fn test_concurrent_get_and_flush_latency() {
    let (storage, _tmp) = test_storage().await;
    let storage = Arc::new(storage);

    for i in 0..100 {
        let tx = TxId::new(i + 1);
        let k = format!("key-{}", i);
        let v = format!("val-{}", i);
        storage.put(tx, k.as_bytes(), v.as_bytes()).await.unwrap();
        storage.commit(tx).await.unwrap();
    }

    let mut handles = Vec::new();
    for task_idx in 0..4 {
        let storage_clone = Arc::clone(&storage);
        let handle = tokio::spawn(async move {
            let mut latencies = Vec::with_capacity(50);
            let key = format!("key-{}", task_idx * 10);
            for _ in 0..50 {
                let req_start = std::time::Instant::now();
                let val = storage_clone.get(key.as_bytes()).await.unwrap();
                let elapsed = req_start.elapsed();
                assert!(val.is_some());
                latencies.push(elapsed);
                tokio::time::sleep(std::time::Duration::from_micros(100)).await;
            }
            latencies
        });
        handles.push(handle);
    }

    storage.flush().await.unwrap();

    let mut all_latencies = Vec::with_capacity(200);
    for handle in handles {
        let latencies = handle.await.unwrap();
        all_latencies.extend(latencies);
    }

    all_latencies.sort();
    let p95 = all_latencies[190];
    let max_lat = *all_latencies.last().unwrap_or(&p95);
    assert!(
        p95 < std::time::Duration::from_millis(5),
        "p95 get() latency took {:?}, max took {:?}, exceeding 5 ms p95 latency threshold under concurrent flush",
        p95,
        max_lat
    );
}

#[tokio::test]
async fn test_two_instances_independent_flush_counters() {
    let tmp1 = TempDir::new().unwrap();
    let tmp2 = TempDir::new().unwrap();

    let storage1 = LsmStorage::new(LsmConfig {
        path: tmp1.path().to_path_buf(),
        ..Default::default()
    })
    .await
    .unwrap();

    let storage2 = LsmStorage::new(LsmConfig {
        path: tmp2.path().to_path_buf(),
        ..Default::default()
    })
    .await
    .unwrap();

    let tx1 = TxId::new(1);
    storage1.put(tx1, b"key1", b"val1").await.unwrap();
    storage1.commit(tx1).await.unwrap();
    storage1.force_flush().await.unwrap();

    let tx2 = TxId::new(1);
    storage2.put(tx2, b"key2", b"val2").await.unwrap();
    storage2.commit(tx2).await.unwrap();
    storage2.force_flush().await.unwrap();

    assert_eq!(storage1.flush_counter.load(Ordering::Relaxed), 1);
    assert_eq!(storage2.flush_counter.load(Ordering::Relaxed), 1);

    assert!(tmp1.path().join("wal-00000000000000000000.log").exists());
    assert!(tmp2.path().join("wal-00000000000000000000.log").exists());
}

#[tokio::test]
async fn test_parallel_flush_counter_no_race() {
    let tmp1 = TempDir::new().unwrap();
    let tmp2 = TempDir::new().unwrap();

    let storage1 = Arc::new(
        LsmStorage::new(LsmConfig {
            path: tmp1.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(),
    );

    let storage2 = Arc::new(
        LsmStorage::new(LsmConfig {
            path: tmp2.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(),
    );

    let tx1 = TxId::new(1);
    storage1.put(tx1, b"key1", b"val1").await.unwrap();
    storage1.commit(tx1).await.unwrap();

    let tx2 = TxId::new(1);
    storage2.put(tx2, b"key2", b"val2").await.unwrap();
    storage2.commit(tx2).await.unwrap();

    let s1 = Arc::clone(&storage1);
    let s2 = Arc::clone(&storage2);

    let (res1, res2) = tokio::join!(s1.force_flush(), s2.force_flush());
    res1.unwrap();
    res2.unwrap();

    assert_eq!(storage1.flush_counter.load(Ordering::Relaxed), 1);
    assert_eq!(storage2.flush_counter.load(Ordering::Relaxed), 1);
    assert!(tmp1.path().join("wal-00000000000000000000.log").exists());
    assert!(tmp2.path().join("wal-00000000000000000000.log").exists());
}

#[tokio::test]
async fn test_lsm_put_if_absent_parallel_two_tasks() {
    let (storage, _tmp) = test_storage().await;
    let storage = Arc::new(storage);
    let key = b"cas_key_2tasks";

    let s1 = Arc::clone(&storage);
    let h1 = tokio::spawn(async move {
        let tx = TxId::new(1);
        let res = s1.put_if_absent(tx, key, b"val1").await;
        if res.as_ref().copied().unwrap_or(false) {
            let _ = s1.commit(tx).await;
        }
        res
    });

    let s2 = Arc::clone(&storage);
    let h2 = tokio::spawn(async move {
        let tx = TxId::new(2);
        let res = s2.put_if_absent(tx, key, b"val2").await;
        if res.as_ref().copied().unwrap_or(false) {
            let _ = s2.commit(tx).await;
        }
        res
    });

    let r1 = h1.await.unwrap().unwrap();
    let r2 = h2.await.unwrap().unwrap();

    assert_ne!(
        r1, r2,
        "Exactly one task must succeed (true) and the other fail (false)"
    );

    let stored_val = storage.get(key).await.unwrap().expect("value must exist");
    if r1 {
        assert_eq!(stored_val, b"val1");
    } else {
        assert_eq!(stored_val, b"val2");
    }
}

#[tokio::test]
async fn test_put_if_absent_no_deadlock_and_no_commit_mutex_holding() {
    let (storage, _tmp) = test_storage().await;
    let storage = Arc::new(storage);

    let commit_guard = storage.commit_mutex.lock().await;

    let key = b"no_commit_mutex_block_key";
    let tx = TxId::new(100);

    let res = storage.put_if_absent(tx, key, b"val").await;
    assert!(
        res.is_ok() && res.unwrap(),
        "put_if_absent must proceed without being blocked by commit_mutex"
    );

    drop(commit_guard);
    storage.commit(tx).await.unwrap();
}

#[tokio::test]
async fn test_put_if_absent_sees_uncommitted_concurrent_stage() {
    let (storage, _tmp) = test_storage().await;

    let key = b"uncommitted_key";
    let tx_a = TxId::new(10);
    let tx_b = TxId::new(20);

    let res_a = storage.put_if_absent(tx_a, key, b"value_a").await.unwrap();
    assert!(res_a, "Transaction A must successfully stage the insert");

    let res_b = storage.put_if_absent(tx_b, key, b"value_b").await.unwrap();
    assert!(
        !res_b,
        "Transaction B must see uncommitted staged insert from Transaction A and return false"
    );
}

#[tokio::test]
async fn test_lsm_put_if_absent_stress_200_tasks() {
    let (storage, _tmp) = test_storage().await;
    let storage = Arc::new(storage);
    let key = b"cas_key_stress_200";

    let mut set = tokio::task::JoinSet::new();

    for i in 0..200u64 {
        let s = Arc::clone(&storage);
        let val = format!("val_{i}").into_bytes();
        set.spawn(async move {
            let tx = TxId::new(i + 1);
            let res = s.put_if_absent(tx, key, &val).await;
            if res.as_ref().copied().unwrap_or(false) {
                let _ = s.commit(tx).await;
            }
            (i, res)
        });
    }

    let mut true_count = 0;
    let mut false_count = 0;
    let mut winning_task_id = None;

    while let Some(res) = set.join_next().await {
        let (task_id, result) = res.unwrap();
        match result {
            Ok(true) => {
                true_count += 1;
                winning_task_id = Some(task_id);
            }
            Ok(false) => {
                false_count += 1;
            }
            Err(e) => panic!("Unexpected error in task {task_id}: {e:?}"),
        }
    }

    assert_eq!(true_count, 1, "Exactly 1 task must return Ok(true)");
    assert_eq!(false_count, 199, "199 tasks must return Ok(false)");

    let winner = winning_task_id.expect("winning task id");
    let expected_val = format!("val_{winner}").into_bytes();
    let stored_val = storage.get(key).await.unwrap().expect("value must exist");
    assert_eq!(
        stored_val, expected_val,
        "Stored value must match winning task's value"
    );
}

#[tokio::test]
async fn test_put_if_absent_no_commit_mutex_hold() {
    let (storage, _tmp) = test_storage().await;

    let tx_a = TxId::new(100);
    let tx_b = TxId::new(200);
    let tx_c = TxId::new(300);

    let key_shared = b"key_shared";
    let key_other = b"key_other";

    let res_a = storage
        .put_if_absent(tx_a, key_shared, b"val_a")
        .await
        .unwrap();
    assert!(res_a, "Tx A must stage insert successfully");

    let start = std::time::Instant::now();
    let res_b = storage
        .put_if_absent(tx_b, key_shared, b"val_b")
        .await
        .unwrap();
    let elapsed = start.elapsed();

    assert!(!res_b, "Tx B must see Tx A's intent lock and return false");
    assert!(
        elapsed < std::time::Duration::from_millis(100),
        "Tx B put_if_absent must complete without lock contention stall (elapsed: {elapsed:?})"
    );

    let res_c = storage
        .put_if_absent(tx_c, key_other, b"val_c")
        .await
        .unwrap();
    assert!(res_c, "Tx C must successfully stage key_other concurrently");
}

#[tokio::test]
async fn test_commit_mutex_released_before_flusher_await() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        group_commit_window_micros: 10_000, // 10ms window to force group commit
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await.expect("create storage"));

    let num_tasks = 10;
    let mut handles = Vec::new();

    for i in 1..=num_tasks {
        let storage = Arc::clone(&storage);
        handles.push(tokio::spawn(async move {
            let tx_id = TxId::new(i);
            let key = format!("concurrent_key_{i}").into_bytes();
            let val = format!("concurrent_val_{i}").into_bytes();

            storage.put(tx_id, &key, &val).await.unwrap();
            storage.commit(tx_id).await.unwrap();
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // Verify all keys written concurrently under group commit are readable and isolated
    for i in 1..=num_tasks {
        let key = format!("concurrent_key_{i}").into_bytes();
        let expected_val = format!("concurrent_val_{i}").into_bytes();
        let val = storage.get(&key).await.unwrap();
        assert_eq!(
            val,
            Some(expected_val),
            "Key concurrent_key_{i} must be persisted and readable"
        );
    }
}
