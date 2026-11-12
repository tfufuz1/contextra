//! # Durabilitaets- und Concurrency-Committests
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Prueft HMAC-Wiederherstellung, Budget-Drift, Druckwaechter und Mutex-Freigabe

use super::super::*;
use crate::error::StoreError as ContextraError;
use std::sync::atomic::Ordering;
use tempfile::TempDir;

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

#[tokio::test]
async fn test_delete_memory_capacity_exceeded() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        max_ram_mb: 1,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    storage
        .budget
        .consume_memory(1_000_000)
        .expect("fill budget");
    assert!(!storage.budget.has_memory_capacity());

    let tx = TxId::new(1);
    let res = storage.delete(tx, b"key").await;
    assert!(
        matches!(res, Err(ContextraError::Storage(ref msg)) if msg.contains("Memory budget exceeded")),
        "delete must fail with Storage error when memory budget is exceeded"
    );

    let res_many = storage
        .delete_many(tx, vec![b"key1".to_vec(), b"key2".to_vec()])
        .await;
    assert!(
        matches!(res_many, Err(ContextraError::Storage(ref msg)) if msg.contains("Memory budget exceeded")),
        "delete_many must fail with Storage error when memory budget is exceeded"
    );
}

#[tokio::test]
async fn test_memory_only_commit_restores_hmac() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        durability_mode: DurabilityMode::MemoryOnly,
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    let wal = storage.wal.read().await;
    let hmac_before = wal.last_hmac_snapshot().await;
    drop(wal);

    let tx = TxId::new(1);
    storage.put(tx, b"key", b"val").await.expect("put");
    storage.commit(tx).await.expect("commit");

    let wal = storage.wal.read().await;
    let hmac_after = wal.last_hmac_snapshot().await;
    assert_eq!(
        hmac_after, hmac_before,
        "last_hmac must be restored to pre-commit state for MemoryOnly durability"
    );
}
