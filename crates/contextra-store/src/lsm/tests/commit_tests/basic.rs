//! # Basis-Committests & Input Boundary Guards
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Prueft Roundtrip, Delete, Delete-Prefix, Delete-Many, Rollback und Input-Validierung

use super::super::*;
use crate::error::StoreError as ContextraError;
use contextra_env::ManualClock;
use std::sync::atomic::Ordering;
use tempfile::TempDir;

async fn test_clock_port_integration_with_manual_clock() {
    let tmp = TempDir::new().expect("temp dir");
    let clock = Arc::new(ManualClock::new(100_000));
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::new(config)
        .await
        .expect("create storage")
        .with_clock(Arc::clone(&clock) as Arc<dyn contextra_env::Clock>);

    assert_eq!(storage.clock().now_unix_nanos(), 100_000);

    clock.advance(Duration::from_nanos(50_000));
    assert_eq!(storage.clock().now_unix_nanos(), 150_000);

    let tx = TxId::new(1);
    storage.put(tx, b"clock_key", b"clock_val").await.unwrap();
    storage.commit(tx).await.unwrap();

    let val = storage.get(b"clock_key").await.unwrap();
    assert_eq!(val, Some(bytes::Bytes::from_static(b"clock_val")));
}

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
        Err(ContextraError::InvalidInput(_))
    ));
    assert!(matches!(
        storage.delete(tx, b"").await,
        Err(ContextraError::InvalidInput(_))
    ));
    assert!(matches!(
        storage.get(b"").await,
        Err(ContextraError::InvalidInput(_))
    ));
    assert!(matches!(
        storage.get_at_seq(b"", 10).await,
        Err(ContextraError::InvalidInput(_))
    ));

    let huge_key = vec![b'a'; MAX_KEY_SIZE + 1];
    assert!(matches!(
        storage.put(tx, &huge_key, b"val").await,
        Err(ContextraError::InvalidInput(_))
    ));
    assert!(matches!(
        storage.delete(tx, &huge_key).await,
        Err(ContextraError::InvalidInput(_))
    ));
    assert!(matches!(
        storage.get(&huge_key).await,
        Err(ContextraError::InvalidInput(_))
    ));

    let too_many_keys = vec![b"key".to_vec(); MAX_BATCH_SIZE + 1];
    assert!(matches!(
        storage.delete_many(tx, too_many_keys).await,
        Err(ContextraError::InvalidInput(_))
    ));

    let huge_val = vec![b'v'; MAX_VALUE_SIZE + 1];
    assert!(matches!(
        storage.put(tx, b"valid_key", &huge_val).await,
        Err(ContextraError::InvalidInput(_))
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
    let expected_entry_size =
        (key.len() + value.len() + contextra_mvcc::tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES) as u64;

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
        last_seq, 20,
        "10 commits must generate 20 sequence numbers (Put + TxEnd per commit) monotonically"
    );
}

