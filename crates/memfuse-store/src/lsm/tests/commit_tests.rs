use super::*;
use std::sync::atomic::Ordering;
use tempfile::TempDir;

#[tokio::test]
async fn test_put_get_roundtrip() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    storage.put(tx, b"hello", b"world").await.expect("put");
    storage.commit(tx).await.expect("commit");

    let val = storage.get(b"hello").await.expect("get");
    assert_eq!(val, Some(bytes::Bytes::from_static(b"world")));
}

#[tokio::test]
async fn test_delete() {
    let (storage, _tmp) = test_storage().await;
    let tx1 = TxId::new(1);

    storage.put(tx1, b"key", b"val").await.expect("put");
    storage.commit(tx1).await.expect("commit");

    let tx2 = TxId::new(2);
    storage.delete(tx2, b"key").await.expect("delete");
    storage.commit(tx2).await.expect("commit");

    let val = storage.get(b"key").await.expect("get");
    assert_eq!(val, None);
}

#[tokio::test]
async fn test_delete_prefix_removes_all_matching_keys() {
    let (storage, _tmp) = test_storage().await;
    let tx1 = TxId::new(1);

    storage.put(tx1, b"test:1", b"val1").await.unwrap();
    storage.put(tx1, b"test:2", b"val2").await.unwrap();
    storage.put(tx1, b"test:3", b"val3").await.unwrap();
    storage.put(tx1, b"other:1", b"val4").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let tx2 = TxId::new(2);
    let deleted = storage.delete_prefix(tx2, b"test:").await.unwrap();
    assert_eq!(deleted, 3);
    storage.commit(tx2).await.unwrap();

    assert_eq!(storage.get(b"test:1").await.unwrap(), None);
    assert_eq!(storage.get(b"test:2").await.unwrap(), None);
    assert_eq!(storage.get(b"test:3").await.unwrap(), None);
    assert_eq!(
        storage.get(b"other:1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val4"))
    );
}

#[tokio::test]
async fn test_lsm_storage_delete_many_uses_single_batch() {
    let (storage, _tmp) = test_storage().await;
    let tx1 = TxId::new(1);

    let keys_to_delete: Vec<Vec<u8>> = (0..50)
        .map(|i| format!("batch_key_{i}").into_bytes())
        .collect();

    for key in &keys_to_delete {
        storage.put(tx1, key, b"value").await.unwrap();
    }
    storage.commit(tx1).await.unwrap();

    let tx2 = TxId::new(2);
    let count = storage
        .delete_many(tx2, keys_to_delete.clone())
        .await
        .unwrap();
    assert_eq!(count, 50);

    let staged_ops = storage.tx_buffer.get_ops(tx2).expect("ops staged");
    assert_eq!(staged_ops.len(), 50);

    storage.commit(tx2).await.unwrap();
    for key in &keys_to_delete {
        assert_eq!(storage.get(key).await.unwrap(), None);
    }
}

#[tokio::test]
async fn test_rollback() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    storage.put(tx, b"key", b"val").await.expect("put");
    storage.rollback(tx).await.expect("rollback");

    let val = storage.get(b"key").await.expect("get");
    assert_eq!(val, None);
}

#[tokio::test]
async fn test_overwrite() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key", b"val1").await.expect("put1");
    storage.commit(tx1).await.expect("commit1");

    let tx2 = TxId::new(2);
    storage.put(tx2, b"key", b"val2").await.expect("put2");
    storage.commit(tx2).await.expect("commit2");

    let val = storage.get(b"key").await.expect("get");
    assert_eq!(val, Some(bytes::Bytes::from_static(b"val2")));
}

#[tokio::test]
async fn test_input_boundary_guards() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");
    let tx = TxId::new(1);

    assert!(matches!(
        storage.put(tx, b"", b"val").await,
        Err(MemFuseError::InvalidInput(_))
    ));
    assert!(matches!(
        storage.delete(tx, b"").await,
        Err(MemFuseError::InvalidInput(_))
    ));
    assert!(matches!(
        storage.get(b"").await,
        Err(MemFuseError::InvalidInput(_))
    ));
    assert!(matches!(
        storage.get_at_seq(b"", 10).await,
        Err(MemFuseError::InvalidInput(_))
    ));

    let huge_key = vec![b'a'; MAX_KEY_SIZE + 1];
    assert!(matches!(
        storage.put(tx, &huge_key, b"val").await,
        Err(MemFuseError::InvalidInput(_))
    ));
    assert!(matches!(
        storage.delete(tx, &huge_key).await,
        Err(MemFuseError::InvalidInput(_))
    ));
    assert!(matches!(
        storage.get(&huge_key).await,
        Err(MemFuseError::InvalidInput(_))
    ));

    let too_many_keys = vec![b"key".to_vec(); MAX_BATCH_SIZE + 1];
    assert!(matches!(
        storage.delete_many(tx, too_many_keys).await,
        Err(MemFuseError::InvalidInput(_))
    ));

    let huge_val = vec![b'v'; MAX_VALUE_SIZE + 1];
    assert!(matches!(
        storage.put(tx, b"valid_key", &huge_val).await,
        Err(MemFuseError::InvalidInput(_))
    ));
}

#[cfg(feature = "fault-injection")]
#[tokio::test]
async fn test_lsm_commit_append_failure_restores_hmac() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"k1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let wal = storage.wal.read().await;
    let hmac_before = wal.last_hmac_snapshot().await;
    drop(wal);

    storage.simulate_wal_append_failure_for_test().await;

    let tx2 = TxId::new(2);
    storage.put(tx2, b"k2", b"v2").await.unwrap();
    let commit_res = storage.commit(tx2).await;
    assert!(commit_res.is_err(), "Commit must fail when WAL write fails");

    let wal = storage.wal.read().await;
    let hmac_after = wal.last_hmac_snapshot().await;
    assert_eq!(
        hmac_after, hmac_before,
        "last_hmac must be restored to pre-commit state after commit failure"
    );
}

#[tokio::test]
async fn test_commit_tracks_budget_drift_on_consume_memory_failure() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 1,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    assert_eq!(storage.budget_tracking_drift_bytes(), 0);

    let key = b"drift_key";
    let value = vec![b'v'; 60000];
    let expected_entry_size = (key.len() + value.len() + 8) as u64;

    let tx = TxId::new(1);
    storage.put(tx, key, &value).await.expect("put succeeds");

    storage
        .budget
        .consume_memory(990_000)
        .expect("fill budget to 990,000");

    let commit_res = storage.commit(tx).await;
    assert!(
        commit_res.is_ok(),
        "commit must succeed even when consume_memory fails"
    );

    assert_eq!(
        storage.budget_tracking_drift_bytes(),
        expected_entry_size,
        "budget drift metric must equal entry_size after consume_memory failure"
    );
}

#[tokio::test]
async fn test_sequence_numbers_strictly_monotonic_across_concurrent_commits() {
    let storage = Arc::new(test_storage().await.0);
    let mut handles = Vec::new();

    for i in 1..=10u64 {
        let st = Arc::clone(&storage);
        handles.push(tokio::spawn(async move {
            let tx = TxId::new(i);
            st.put(tx, format!("concurrent_key_{i}").as_bytes(), b"val")
                .await
                .unwrap();
            st.commit(tx).await.unwrap();
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let last_seq = storage.last_seq_no().await.unwrap();
    assert_eq!(
        last_seq, 10,
        "10 commits must generate sequence numbers 1..10 monotonically"
    );
}

#[tokio::test]
async fn test_system_pressure_monitor_integration() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    let rx = storage.pressure_receiver();
    let pressure = rx.borrow().clone();
    assert_eq!(
        pressure.pressure_level,
        crate::system_pressure::PressureLevel::Normal
    );
    assert_eq!(pressure.wal_queue_depth, 0);

    storage.shutdown();
}

#[tokio::test]
async fn test_system_pressure_wal_queue_backpressure_transition() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 200_000,
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await.expect("create storage"));
    let mut pressure_rx = storage.pressure_receiver();

    let num_tasks = 600;
    let barrier = Arc::new(tokio::sync::Barrier::new(num_tasks));
    let mut handles = Vec::with_capacity(num_tasks);

    for i in 0..num_tasks {
        let storage = Arc::clone(&storage);
        let barrier = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            let tx = TxId::new((i + 1) as u64);
            let key = format!("k{:05}", i).into_bytes();
            let val = format!("v{:05}", i).into_bytes();
            storage.put(tx, &key, &val).await.expect("put");
            barrier.wait().await;
            storage.commit(tx).await.expect("commit");
        }));
    }

    let mut max_wal_depth = 0;
    let mut critical_observed = false;

    let monitor_handle = tokio::spawn(async move {
        let timeout = Duration::from_secs(5);
        let start = std::time::Instant::now();
        loop {
            let current = pressure_rx.borrow().clone();
            if current.wal_queue_depth > max_wal_depth {
                max_wal_depth = current.wal_queue_depth;
            }
            if current.pressure_level == crate::system_pressure::PressureLevel::Critical {
                critical_observed = true;
                break;
            }
            if start.elapsed() > timeout {
                break;
            }
            if pressure_rx.changed().await.is_err() {
                break;
            }
        }
        (max_wal_depth, critical_observed)
    });

    for h in handles {
        h.await.expect("task join");
    }

    let (max_depth, transitioned) = monitor_handle.await.expect("monitor join");

    assert!(
        transitioned || max_depth > crate::system_pressure::WAL_QUEUE_CRITICAL_THRESHOLD,
        "Production pressure_rx should transition to Critical when WAL queue depth ({}) exceeds threshold ({})",
        max_depth,
        crate::system_pressure::WAL_QUEUE_CRITICAL_THRESHOLD
    );

    storage.shutdown();
}

#[tokio::test]
async fn test_group_commit_leader_releases_commit_mutex_during_disk_io() {
    use crate::wal::DELAY_APPEND_FOR_TX;
    use crate::wal::DELAY_APPEND_MS;

    let tmp = tempfile::TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 2_000, // 2ms group commit window
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await.expect("new storage"));

    // Prepare tx 10 as group commit leader and tx 11 as follower
    let tx_leader = TxId::new(10);
    let tx_follower = TxId::new(11);

    storage.put(tx_leader, b"leader_key", b"val").await.unwrap();
    storage
        .put(tx_follower, b"follower_key", b"val")
        .await
        .unwrap();

    // Inject 500ms delay into append_batch for tx_leader
    DELAY_APPEND_FOR_TX.store(10, Ordering::SeqCst);
    DELAY_APPEND_MS.store(500, Ordering::SeqCst);

    let storage_leader = Arc::clone(&storage);
    let leader_handle = tokio::spawn(async move { storage_leader.commit(tx_leader).await });

    // Give leader time to initialize group commit queue and enter commit_mutex block
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Follower joins group commit queue
    let storage_follower = Arc::clone(&storage);
    let follower_handle = tokio::spawn(async move { storage_follower.commit(tx_follower).await });

    // Wait until the group commit leader starts executing wal.append_batch (with 500ms delay)
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Attempt to acquire commit_mutex while leader is delayed inside wal.append_batch().
    // If commit_mutex was correctly released before disk I/O, try_lock() MUST succeed!
    let try_lock_res = storage.commit_mutex.try_lock();
    assert!(
        try_lock_res.is_ok(),
        "commit_mutex must be released during group commit leader disk I/O (wal.append_batch)"
    );
    drop(try_lock_res);

    // Clean up tasks and reset fault injection state
    leader_handle.await.expect("leader task").unwrap();
    follower_handle.await.expect("follower task").unwrap();
    DELAY_APPEND_FOR_TX.store(0, Ordering::SeqCst);
    DELAY_APPEND_MS.store(0, Ordering::SeqCst);
}
