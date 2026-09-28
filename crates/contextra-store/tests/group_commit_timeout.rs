//! Testverifikation für Follower-Timeout während blockiertem Group-Commit Leader.
//! Nacharbeit zu Commit 694fa8c2 (Catch-Up Verification): Stellt sicher, dass
//! Follower nach `tx_timeout` verlässlichen `ContextraError::CommitTimeout` erhalten,
//! die Pending Queue bereinigt wird und Folgetransaktionen erfolgreich durchlaufen.
//!
//! Ausführung: cargo test -p contextra-store --test group_commit_timeout --features fault-injection

#![cfg(feature = "fault-injection")]

use contextra_core::{ContextraError, StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::wal::{DELAY_APPEND_FOR_TX, DELAY_APPEND_MS};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_group_commit_follower_timeout_and_queue_recovery() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        tx_timeout: Duration::from_millis(100),
        group_commit_window_micros: 20_000, // 20ms batching window
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await.expect("new storage"));

    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    // 1. Stage KV entries for both transactions
    storage.put(tx1, b"k_tx1", b"v_tx1").await.expect("put tx1");
    storage.put(tx2, b"k_tx2", b"v_tx2").await.expect("put tx2");

    // 2. Configure Fault Injection: delay WAL append batch containing tx1 by 500ms
    DELAY_APPEND_FOR_TX.store(tx1.inner(), Ordering::SeqCst);
    DELAY_APPEND_MS.store(500, Ordering::SeqCst);

    let barrier = Arc::new(tokio::sync::Barrier::new(2));

    let storage_a = Arc::clone(&storage);
    let barrier_a = Arc::clone(&barrier);
    let task_a = tokio::spawn(async move {
        barrier_a.wait().await;
        storage_a.commit(tx1).await
    });

    let storage_b = Arc::clone(&storage);
    let barrier_b = Arc::clone(&barrier);
    let task_b = tokio::spawn(async move {
        barrier_b.wait().await;
        storage_b.commit(tx2).await
    });

    let (res_a, res_b) = tokio::join!(task_a, task_b);
    let res_a = res_a.expect("task a join");
    let res_b = res_b.expect("task b join");

    // Reset Fault-Injection Flags
    DELAY_APPEND_FOR_TX.store(0, Ordering::SeqCst);
    DELAY_APPEND_MS.store(0, Ordering::SeqCst);

    // 3. Verifiziere: Mindestens eine Transaktion schlägt fehl mit CommitTimeout (der Follower)
    let follower_res = if res_a.is_err() {
        res_a
    } else if res_b.is_err() {
        res_b
    } else {
        panic!("One transaction must time out and return Err(CommitTimeout)");
    };

    let err = follower_res.expect_err("Expected error");
    assert!(
        matches!(err, ContextraError::CommitTimeout { .. }),
        "Expected CommitTimeout, got {:?}",
        err
    );

    // 4. Dritte Transaktion durchführen (neuer Leader) zur Bestätigung,
    // dass pending_commit_queue geleert ist und Folgetransaktionen problemlos durchlaufen.
    let tx_subsequent = TxId::new(3);
    storage
        .put(tx_subsequent, b"k_subsequent", b"v_subsequent")
        .await
        .expect("put subsequent");
    let subsequent_res = storage.commit(tx_subsequent).await;
    assert!(
        subsequent_res.is_ok(),
        "Subsequent transaction commit must succeed: {:?}",
        subsequent_res
    );

    let v_subsequent = storage
        .get(b"k_subsequent")
        .await
        .expect("get subsequent key");
    assert_eq!(
        v_subsequent,
        Some(bytes::Bytes::from_static(b"v_subsequent"))
    );
}

#[tokio::test]
async fn test_group_commit_follower_timeout_preserves_other_followers() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        tx_timeout: Duration::from_millis(100),
        group_commit_window_micros: 20_000, // 20ms batching window
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await.expect("new storage"));

    let tx_leader = TxId::new(10);
    let tx_follower_timeout = TxId::new(11);

    storage
        .put(tx_leader, b"k_leader", b"v_leader")
        .await
        .expect("put leader");
    storage
        .put(tx_follower_timeout, b"k_f1", b"v_f1")
        .await
        .expect("put f1");

    // Configure Fault Injection: delay WAL append batch containing tx_leader by 400ms
    DELAY_APPEND_FOR_TX.store(tx_leader.inner(), Ordering::SeqCst);
    DELAY_APPEND_MS.store(400, Ordering::SeqCst);

    let barrier = Arc::new(tokio::sync::Barrier::new(2));

    let storage_leader = Arc::clone(&storage);
    let barrier_leader = Arc::clone(&barrier);
    let task_leader = tokio::spawn(async move {
        barrier_leader.wait().await;
        storage_leader.commit(tx_leader).await
    });

    let storage_f1 = Arc::clone(&storage);
    let barrier_f1 = Arc::clone(&barrier);
    let task_f1 = tokio::spawn(async move {
        barrier_f1.wait().await;
        storage_f1.commit(tx_follower_timeout).await
    });

    let (res_leader, res_f1) = tokio::join!(task_leader, task_f1);
    let res_leader = res_leader.expect("leader join");
    let res_f1 = res_f1.expect("f1 join");

    DELAY_APPEND_FOR_TX.store(0, Ordering::SeqCst);
    DELAY_APPEND_MS.store(0, Ordering::SeqCst);

    let (follower_res, leader_res) = if res_leader.is_err() {
        (res_leader, res_f1)
    } else {
        (res_f1, res_leader)
    };

    assert!(
        leader_res.is_ok(),
        "Leader commit must succeed: {:?}",
        leader_res
    );
    let follower_err = follower_res.expect_err("Follower must time out");
    assert!(
        matches!(follower_err, ContextraError::CommitTimeout { .. }),
        "Expected CommitTimeout, got {:?}",
        follower_err
    );

    // Verify leader data was written properly and queue is clear for new transaction
    let tx_next = TxId::new(12);
    storage
        .put(tx_next, b"k_next", b"v_next")
        .await
        .expect("put next");
    let res_next = storage.commit(tx_next).await;
    assert!(
        res_next.is_ok(),
        "Subsequent commit must succeed: {:?}",
        res_next
    );

    let v_leader = storage.get(b"k_leader").await.expect("get leader");
    assert_eq!(v_leader, Some(bytes::Bytes::from_static(b"v_leader")));
}
