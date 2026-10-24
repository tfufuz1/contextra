//! Integration test for SSI tracked read sequence clamping in LsmStorage.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{ContextraError, TxId};
use contextra_ports::StorageEngine;
use contextra_store::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::TempDir;

/// Scenario A: tx0 writes k1 and commits.
/// tx1 calls get_at_seq_tracked(tx1, b"k1", u64::MAX).
/// tx2 concurrently updates k1 and commits.
/// tx1 writes k2 and commits.
/// Expected: commit(tx1) returns Err(ContextraError::Conflict(_)).
#[tokio::test]
async fn test_scenario_a_get_at_seq_tracked_u64_max_conflict() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let lsm = Arc::new(LsmStorage::new(config).await.expect("lsm init"));

    // Init key k1
    let tx0 = TxId::new(1);
    lsm.put(tx0, b"k1", b"v0").await.expect("put k1");
    lsm.commit(tx0).await.expect("commit tx0");

    // Tx1 calls get_at_seq_tracked with snapshot_seq = u64::MAX
    let tx1 = TxId::new(10);
    let val1 = lsm
        .get_at_seq_tracked(tx1, b"k1", u64::MAX)
        .await
        .expect("get_at_seq_tracked");
    assert_eq!(val1, Some(bytes::Bytes::from_static(b"v0")));

    // Tx2 concurrently updates k1 and commits
    let tx2 = TxId::new(20);
    lsm.put(tx2, b"k1", b"v2").await.expect("put k1 tx2");
    lsm.commit(tx2).await.expect("commit tx2");

    // Tx1 writes k2 and attempts to commit
    lsm.put(tx1, b"k2", b"v1_tx1").await.expect("put k2 tx1");
    let res1 = lsm.commit(tx1).await;

    assert!(
        matches!(res1, Err(ContextraError::Conflict(_))),
        "Expected Conflict on get_at_seq_tracked(u64::MAX) after concurrent write to k1, got: {:?}",
        res1
    );
}

/// Scenario B (Counter-probe): Identical to Scenario A, but without tx2's write to k1.
/// Expected: commit(tx1) succeeds with Ok(()).
#[tokio::test]
async fn test_scenario_b_get_at_seq_tracked_u64_max_no_conflict() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let lsm = Arc::new(LsmStorage::new(config).await.expect("lsm init"));

    // Init key k1
    let tx0 = TxId::new(1);
    lsm.put(tx0, b"k1", b"v0").await.expect("put k1");
    lsm.commit(tx0).await.expect("commit tx0");

    // Tx1 calls get_at_seq_tracked with snapshot_seq = u64::MAX
    let tx1 = TxId::new(10);
    let val1 = lsm
        .get_at_seq_tracked(tx1, b"k1", u64::MAX)
        .await
        .expect("get_at_seq_tracked");
    assert_eq!(val1, Some(bytes::Bytes::from_static(b"v0")));

    // No tx2 write!

    // Tx1 writes k2 and attempts to commit
    lsm.put(tx1, b"k2", b"v1_tx1").await.expect("put k2 tx1");
    let res1 = lsm.commit(tx1).await;

    assert!(
        res1.is_ok(),
        "Expected Ok(()) on commit(tx1) when no concurrent write occurred, got: {:?}",
        res1
    );
}

/// Scenario C: get_at_seq_tracked with an explicit lower seq value continues to read
/// the value at that sequence number (read semantics remain unchanged), and registers at most that seq.
#[tokio::test]
async fn test_scenario_c_get_at_seq_tracked_explicit_lower_seq() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let lsm = Arc::new(LsmStorage::new(config).await.expect("lsm init"));

    // tx0 writes k1 = v0
    let tx0 = TxId::new(1);
    lsm.put(tx0, b"k1", b"v0").await.expect("put k1 v0");
    lsm.commit(tx0).await.expect("commit tx0");
    let seq_v0 = lsm.last_seq_no().await.expect("last_seq_no");

    // tx0_2 updates k1 = v1
    let tx0_2 = TxId::new(2);
    lsm.put(tx0_2, b"k1", b"v1").await.expect("put k1 v1");
    lsm.commit(tx0_2).await.expect("commit tx0_2");

    // tx1 reads k1 at seq_v0 (explicit lower sequence number)
    let tx1 = TxId::new(10);
    let val1 = lsm
        .get_at_seq_tracked(tx1, b"k1", seq_v0)
        .await
        .expect("get_at_seq_tracked at seq_v0");

    // Read semantics must return "v0", NOT "v1"
    assert_eq!(
        val1,
        Some(bytes::Bytes::from_static(b"v0")),
        "Explicit lower seq query must return historical snapshot value"
    );

    // Concurrent tx2 updates k1 after tx1's read point
    let tx2 = TxId::new(20);
    lsm.put(tx2, b"k1", b"v2").await.expect("put k1 tx2");
    lsm.commit(tx2).await.expect("commit tx2");

    // tx1 writes k2 and attempts to commit -> must detect conflict since tx2's write (commit_seq) > seq_v0
    lsm.put(tx1, b"k2", b"v1_tx1").await.expect("put k2 tx1");
    let res1 = lsm.commit(tx1).await;

    assert!(
        matches!(res1, Err(ContextraError::Conflict(_))),
        "Expected Conflict on get_at_seq_tracked(seq_v0) after concurrent write to k1, got: {:?}",
        res1
    );
}
