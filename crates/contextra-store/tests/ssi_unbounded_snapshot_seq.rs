//! Integration test verifying SSI tracked reads when specifying `u64::MAX` as `snapshot_seq` (B-16).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{ContextraError, StorageEngine, TxId};
use contextra_store::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::TempDir;

/// Test (a): Two transactions read the same key with `get_at_seq_tracked(tx, key, u64::MAX)`.
/// Tx1 updates `shared_key` and commits. Tx2 writes its own key and attempts commit -> MUST fail with SSI conflict.
#[tokio::test]
async fn test_b16_concurrent_reads_with_u64_max_causes_conflict() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::new(config).await.expect("lsm init"));

    // Seed initial key
    let tx0 = TxId::new(1);
    lsm.put(tx0, b"shared_key", b"v0")
        .await
        .expect("put initial");
    lsm.commit(tx0).await.expect("commit tx0");

    // Tx1 reads shared_key with u64::MAX
    let tx1 = TxId::new(10);
    let val1 = lsm
        .get_at_seq_tracked(tx1, b"shared_key", u64::MAX)
        .await
        .expect("get_at_seq_tracked tx1");
    assert_eq!(val1, Some(bytes::Bytes::from_static(b"v0")));

    // Tx2 reads shared_key with u64::MAX
    let tx2 = TxId::new(20);
    let val2 = lsm
        .get_at_seq_tracked(tx2, b"shared_key", u64::MAX)
        .await
        .expect("get_at_seq_tracked tx2");
    assert_eq!(val2, Some(bytes::Bytes::from_static(b"v0")));

    // Tx1 updates shared_key and commits
    lsm.put(tx1, b"shared_key", b"v1").await.expect("put tx1");
    lsm.commit(tx1).await.expect("commit tx1");

    // Tx2 writes tx2_key and attempts commit -> MUST trigger SSI Conflict on shared_key
    lsm.put(tx2, b"tx2_key", b"v2").await.expect("put tx2");
    let res2 = lsm.commit(tx2).await;

    assert!(
        matches!(res2, Err(ContextraError::Conflict(_))),
        "Expected SSI Conflict for Tx2, got: {:?}",
        res2
    );
}

/// Test (b): Transaction A reads with `u64::MAX`, Transaction B commits a write to the same key AFTER A's read,
/// Transaction A writes another key and commits -> conflict expected.
#[tokio::test]
async fn test_b16_concurrent_write_after_read_with_u64_max_causes_conflict() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::new(config).await.expect("lsm init"));

    // Seed initial key
    let tx0 = TxId::new(1);
    lsm.put(tx0, b"key_a", b"v0").await.expect("put initial");
    lsm.commit(tx0).await.expect("commit tx0");

    // TxA reads key_a with u64::MAX
    let tx_a = TxId::new(100);
    let val_a = lsm
        .get_at_seq_tracked(tx_a, b"key_a", u64::MAX)
        .await
        .expect("read tx_a");
    assert_eq!(val_a, Some(bytes::Bytes::from_static(b"v0")));

    // TxB modifies key_a and commits AFTER TxA's read
    let tx_b = TxId::new(200);
    lsm.put(tx_b, b"key_a", b"v_b").await.expect("put tx_b");
    lsm.commit(tx_b).await.expect("commit tx_b");

    // TxA writes another key and commits -> MUST detect write-skew / conflict on key_a
    lsm.put(tx_a, b"key_other", b"v_a").await.expect("put tx_a");
    let res_a = lsm.commit(tx_a).await;

    assert!(
        matches!(res_a, Err(ContextraError::Conflict(_))),
        "Expected SSI Conflict for TxA, got: {:?}",
        res_a
    );
}

/// Test (d): Disjoint keys produce no false-positive conflict.
#[tokio::test]
async fn test_ssi_disjoint_keys_no_conflict() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::new(config).await.expect("lsm init"));

    let tx0 = TxId::new(1);
    lsm.put(tx0, b"key_1", b"v1").await.expect("put key_1");
    lsm.put(tx0, b"key_2", b"v2").await.expect("put key_2");
    lsm.commit(tx0).await.expect("commit tx0");

    // Tx1 reads key_1 with u64::MAX
    let tx1 = TxId::new(10);
    let _ = lsm
        .get_at_seq_tracked(tx1, b"key_1", u64::MAX)
        .await
        .expect("get tx1");

    // Tx2 reads key_2 with u64::MAX
    let tx2 = TxId::new(20);
    let _ = lsm
        .get_at_seq_tracked(tx2, b"key_2", u64::MAX)
        .await
        .expect("get tx2");

    // Tx1 writes to key_1_new and commits
    lsm.put(tx1, b"key_1_new", b"val_1").await.expect("put tx1");
    lsm.commit(tx1).await.expect("commit tx1");

    // Tx2 writes to key_2_new and commits -> no conflict because read sets are disjoint
    lsm.put(tx2, b"key_2_new", b"val_2").await.expect("put tx2");
    let res2 = lsm.commit(tx2).await;
    assert!(
        res2.is_ok(),
        "Disjoint keys should commit successfully without conflict, got: {:?}",
        res2
    );
}

/// Test (e): Explicitly older `snapshot_seq` is registered and triggers conflict when concurrent write happens after `snapshot_seq`.
#[tokio::test]
async fn test_explicit_older_snapshot_seq_registered_unchanged() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::new(config).await.expect("lsm init"));

    let tx0 = TxId::new(1);
    lsm.put(tx0, b"historical_key", b"v_old")
        .await
        .expect("put v_old");
    lsm.commit(tx0).await.expect("commit tx0");

    let old_seq = lsm.last_applied_seq();

    // Advance last_applied_seq with more commits to historical_key
    let tx1 = TxId::new(2);
    lsm.put(tx1, b"historical_key", b"v_new")
        .await
        .expect("put v_new");
    lsm.commit(tx1).await.expect("commit tx1");

    // Tx2 reads historical_key with explicitly older snapshot_seq = old_seq
    let tx2 = TxId::new(10);
    let val_tx2 = lsm
        .get_at_seq_tracked(tx2, b"historical_key", old_seq)
        .await
        .expect("get_at_seq_tracked old_seq");
    assert_eq!(val_tx2, Some(bytes::Bytes::from_static(b"v_old")));

    // Tx2 writes historical_key_write and commits -> MUST fail with conflict because tx1 modified historical_key at sequence > old_seq
    lsm.put(tx2, b"historical_key_write", b"v_tx2")
        .await
        .expect("put tx2");
    let res2 = lsm.commit(tx2).await;
    assert!(
        matches!(res2, Err(ContextraError::Conflict(_))),
        "Expected SSI Conflict for Tx2 registered at old_seq, got: {:?}",
        res2
    );
}
