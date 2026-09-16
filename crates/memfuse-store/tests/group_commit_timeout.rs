//! Testverifikation für Follower-Timeout während blockiertem Group-Commit Leader.
//! Nacharbeit zu Commit 694fa8c2 (Catch-Up Verification): Stellt sicher, dass
//! Follower nach `tx_timeout` verlässlichen `MemFuseError::CommitTimeout` erhalten,
//! die Pending Queue bereinigt wird und Folgetransaktionen erfolgreich durchlaufen.
//!
//! Ausführung: cargo test -p memfuse-store --test group_commit_timeout --features fault-injection

#![cfg(feature = "fault-injection")]

use memfuse_core::{MemFuseError, StorageEngine, TxId};
use memfuse_store::lsm::{LsmConfig, LsmStorage};
use memfuse_store::wal::{DELAY_APPEND_FOR_TX, DELAY_APPEND_MS};
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
        matches!(err, MemFuseError::CommitTimeout { .. }),
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
