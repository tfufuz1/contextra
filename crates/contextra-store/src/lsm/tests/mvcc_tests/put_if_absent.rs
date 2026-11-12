//! # Put-If-Absent Concurrency & Deadlock Tests
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Prueft TOCTOU-Sicherheit und Deadlock-Freiheit von put_if_absent

use super::super::*;
use std::sync::atomic::Ordering;
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
        assert_eq!(&stored_val[..], b"val1");
    } else {
        assert_eq!(&stored_val[..], b"val2");
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
            Some(bytes::Bytes::from(expected_val)),
            "Key concurrent_key_{i} must be persisted and readable"
        );
    }
}

#[tokio::test]
async fn test_snapshot_single_load_invariant() {
    let tmp = tempfile::tempdir().unwrap();
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await.expect("create storage"));

    // Populate data with tx 1
    let tx1 = TxId::new(1);
    storage
        .put(tx1, b"key:single_load:1", b"val1")
        .await
        .unwrap();
    storage.commit(tx1).await.unwrap();

    // Verify get_at_seq reads snapshot consistently
    let res = storage.get_at_seq(b"key:single_load:1", 100).await.unwrap();
    assert_eq!(res, Some(bytes::Bytes::from_static(b"val1")));

    // Verify scan_prefix_at reads snapshot consistently
    let entries = storage
        .scan_prefix_at(b"key:single_load:", 100)
        .await
        .unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, b"key:single_load:1".to_vec());
    assert_eq!(entries[0].1, b"val1".to_vec());
}
