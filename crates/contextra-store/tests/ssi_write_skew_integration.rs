//! Integration tests for Serializable Snapshot Isolation (SSI) write-skew conflict detection.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{ContextraError, StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Barrier;

/// K5 Test 1: SSI write-skew detection with immediate commit (`group_commit_window_micros = 0`).
///
/// Scenario:
/// Two accounts A and B with balance 100 each. Total balance constraint >= 0.
/// Tx1 reads A and B, calculates A+B=200 >= 100, withdraws 80 from A (A=20).
/// Tx2 reads A and B, calculates A+B=200 >= 100, withdraws 80 from B (B=20).
/// Under SSI, one transaction must succeed and the other must be rejected with Conflict.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_ssi_write_skew_group_commit_disabled() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await.expect("new storage"));

    // Initial setup: Account A = 100, Account B = 100
    let tx0 = TxId::new(1);
    storage.put(tx0, b"account_a", b"100").await.expect("put A");
    storage.put(tx0, b"account_b", b"100").await.expect("put B");
    storage.commit(tx0).await.expect("commit tx0");

    let barrier = Arc::new(Barrier::new(2));

    let storage_1 = Arc::clone(&storage);
    let barrier_1 = Arc::clone(&barrier);
    let task1 = tokio::spawn(async move {
        let tx1 = TxId::new(10);
        let _a = storage_1.get_tracked(tx1, b"account_a").await.expect("tracked read A");
        let _b = storage_1.get_tracked(tx1, b"account_b").await.expect("tracked read B");

        barrier_1.wait().await;

        storage_1.put(tx1, b"account_a", b"20").await.expect("put A tx1");
        storage_1.commit(tx1).await
    });

    let storage_2 = Arc::clone(&storage);
    let barrier_2 = Arc::clone(&barrier);
    let task2 = tokio::spawn(async move {
        let tx2 = TxId::new(20);
        let _a = storage_2.get_tracked(tx2, b"account_a").await.expect("tracked read A");
        let _b = storage_2.get_tracked(tx2, b"account_b").await.expect("tracked read B");

        barrier_2.wait().await;

        storage_2.put(tx2, b"account_b", b"20").await.expect("put B tx2");
        storage_2.commit(tx2).await
    });

    let res1 = task1.await.expect("task1 join");
    let res2 = task2.await.expect("task2 join");

    let (successes, conflicts) = match (res1, res2) {
        (Ok(_), Err(ContextraError::Conflict(_))) | (Err(ContextraError::Conflict(_)), Ok(_)) => (1, 1),
        (r1, r2) => panic!("Expected exactly 1 Ok and 1 Conflict, got: res1={:?}, res2={:?}", r1, r2),
    };

    assert_eq!(successes, 1);
    assert_eq!(conflicts, 1);
}

/// K5 Test 2: SSI write-skew detection with group commit enabled (`group_commit_window_micros > 0`).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_ssi_write_skew_group_commit_enabled() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 5_000, // 5ms group commit window
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await.expect("new storage"));

    // Initial setup: doctor_1 = active, doctor_2 = active
    let tx0 = TxId::new(1);
    storage.put(tx0, b"doctor_1", b"active").await.expect("put doc 1");
    storage.put(tx0, b"doctor_2", b"active").await.expect("put doc 2");
    storage.commit(tx0).await.expect("commit tx0");

    let barrier = Arc::new(Barrier::new(2));

    let storage_1 = Arc::clone(&storage);
    let barrier_1 = Arc::clone(&barrier);
    let task1 = tokio::spawn(async move {
        let tx1 = TxId::new(100);
        let _d1 = storage_1.get_tracked(tx1, b"doctor_1").await.expect("tracked read d1");
        let _d2 = storage_1.get_tracked(tx1, b"doctor_2").await.expect("tracked read d2");

        barrier_1.wait().await;

        storage_1.put(tx1, b"doctor_1", b"inactive").await.expect("put d1 tx1");
        storage_1.commit(tx1).await
    });

    let storage_2 = Arc::clone(&storage);
    let barrier_2 = Arc::clone(&barrier);
    let task2 = tokio::spawn(async move {
        let tx2 = TxId::new(200);
        let _d1 = storage_2.get_tracked(tx2, b"doctor_1").await.expect("tracked read d1");
        let _d2 = storage_2.get_tracked(tx2, b"doctor_2").await.expect("tracked read d2");

        barrier_2.wait().await;

        storage_2.put(tx2, b"doctor_2", b"inactive").await.expect("put d2 tx2");
        storage_2.commit(tx2).await
    });

    let res1 = task1.await.expect("task1 join");
    let res2 = task2.await.expect("task2 join");

    let (successes, conflicts) = match (res1, res2) {
        (Ok(_), Err(ContextraError::Conflict(_))) | (Err(ContextraError::Conflict(_)), Ok(_)) => (1, 1),
        (r1, r2) => panic!("Expected exactly 1 Ok and 1 Conflict in group commit, got: res1={:?}, res2={:?}", r1, r2),
    };

    assert_eq!(successes, 1);
    assert_eq!(conflicts, 1);
}

/// K5 Test 3: Conflict rejection safety verifying no sequence gaps or uncommitted WAL entries are left.
#[tokio::test]
async fn test_ssi_conflict_rejection_no_sequence_gap_or_wal_leak() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = LsmStorage::new(config.clone()).await.expect("new storage");

    // Initialize key_x = "v1"
    let tx0 = TxId::new(1);
    storage.put(tx0, b"key_x", b"v1").await.expect("put key_x");
    storage.commit(tx0).await.expect("commit tx0");

    let seq_before = storage.next_seq_no_for_test();

    // Tx1 reads key_x
    let tx1 = TxId::new(10);
    let val = storage.get_tracked(tx1, b"key_x").await.expect("get_tracked");
    assert_eq!(val, Some(bytes::Bytes::from_static(b"v1")));

    // Tx2 concurrently mutates key_x and commits
    let tx2 = TxId::new(20);
    storage.put(tx2, b"key_x", b"v2").await.expect("put key_x tx2");
    storage.commit(tx2).await.expect("commit tx2");

    let seq_after_tx2 = storage.next_seq_no_for_test();

    // Tx1 now stages a write to key_y and attempts to commit
    storage.put(tx1, b"key_y", b"attempted_y").await.expect("put key_y tx1");
    let commit_res = storage.commit(tx1).await;

    assert!(
        matches!(commit_res, Err(ContextraError::Conflict(_))),
        "Expected Conflict error, got: {:?}",
        commit_res
    );

    // Assert: next_seq_no after rejected commit MUST equal next_seq_no after tx2 (no sequence gap)
    let seq_after_rejected = storage.next_seq_no_for_test();
    assert_eq!(
        seq_after_rejected, seq_after_tx2,
        "next_seq_no must not increase on rejected SSI commit (seq_before={seq_before}, seq_after_tx2={seq_after_tx2}, seq_after_rejected={seq_after_rejected})"
    );

    // Verify key_y is not in memory
    let y_mem = storage.get(b"key_y").await.expect("get key_y");
    assert_eq!(y_mem, None);

    // Close storage and reopen to verify no WAL entries were appended for tx1
    storage.close().await.expect("close storage");

    let reopened = LsmStorage::new(config).await.expect("reopen storage");
    let y_reopened = reopened.get(b"key_y").await.expect("get key_y reopened");
    assert_eq!(y_reopened, None, "Uncommitted key_y must not exist in replayed storage");
}

/// K5 Test 4 / Task 9 Regressionstest: Confirm that untracked `get()` reads do NOT trigger SSI conflicts.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_untracked_get_no_conflict_regression() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await.expect("new storage"));

    // Initial setup: Account A = 100, Account B = 100
    let tx0 = TxId::new(1);
    storage.put(tx0, b"account_a", b"100").await.expect("put A");
    storage.put(tx0, b"account_b", b"100").await.expect("put B");
    storage.commit(tx0).await.expect("commit tx0");

    let barrier = Arc::new(Barrier::new(2));

    let storage_1 = Arc::clone(&storage);
    let barrier_1 = Arc::clone(&barrier);
    let task1 = tokio::spawn(async move {
        let tx1 = TxId::new(10);
        // Untracked reads via StorageEngine::get / LsmStorage::get
        let _a = storage_1.get(b"account_a").await.expect("untracked read A");
        let _b = storage_1.get(b"account_b").await.expect("untracked read B");

        barrier_1.wait().await;

        storage_1.put(tx1, b"account_a", b"20").await.expect("put A tx1");
        storage_1.commit(tx1).await
    });

    let storage_2 = Arc::clone(&storage);
    let barrier_2 = Arc::clone(&barrier);
    let task2 = tokio::spawn(async move {
        let tx2 = TxId::new(20);
        // Untracked reads via StorageEngine::get / LsmStorage::get
        let _a = storage_2.get(b"account_a").await.expect("untracked read A");
        let _b = storage_2.get(b"account_b").await.expect("untracked read B");

        barrier_2.wait().await;

        storage_2.put(tx2, b"account_b", b"20").await.expect("put B tx2");
        storage_2.commit(tx2).await
    });

    let res1 = task1.await.expect("task1 join");
    let res2 = task2.await.expect("task2 join");

    // Both commits succeed because reads were UNTRACKED (documented behavior)
    assert!(res1.is_ok(), "Tx1 with untracked get() should succeed: {:?}", res1);
    assert!(res2.is_ok(), "Tx2 with untracked get() should succeed: {:?}", res2);
}
