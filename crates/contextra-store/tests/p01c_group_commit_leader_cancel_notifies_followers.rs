// FILE-CONTEXT
// STAND: 2026-10-06
// ZWECK: Verifiziert, dass Leader-Cancellation im Group-Commit alle Follower deterministisch mit Storage-Error benachrichtigt und Follower keine Flags fremder Batches lesen.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{ContextraError, StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::time::Duration;

#[tokio::test]
async fn test_leader_cancel_notifies_followers_even_under_queue_lock_contention() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 200_000, // 200ms
        tx_timeout: Duration::from_secs(5),
        ..Default::default()
    };

    let storage = Arc::new(
        LsmStorage::new(config)
            .await
            .expect("LsmStorage creation failed"),
    );

    // Leader Tx 1
    let tx1 = TxId::new(1);
    storage.put(tx1, b"lead_k", b"lead_v").await.unwrap();

    let st1 = Arc::clone(&storage);
    let leader_task = tokio::spawn(async move { st1.commit(tx1).await });

    // Wait until leader enters group commit window
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Follower Tx 2
    let tx2 = TxId::new(2);
    storage.put(tx2, b"foll_k2", b"foll_v2").await.unwrap();
    let st2 = Arc::clone(&storage);
    let follower2_task = tokio::spawn(async move { st2.commit(tx2).await });

    // Follower Tx 3
    let tx3 = TxId::new(3);
    storage.put(tx3, b"foll_k3", b"foll_v3").await.unwrap();
    let st3 = Arc::clone(&storage);
    let follower3_task = tokio::spawn(async move { st3.commit(tx3).await });

    // Follower Tx 4
    let tx4 = TxId::new(4);
    storage.put(tx4, b"foll_k4", b"foll_v4").await.unwrap();
    let st4 = Arc::clone(&storage);
    let follower4_task = tokio::spawn(async move { st4.commit(tx4).await });

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Abort the Leader task while followers are in queue or acquiring lock
    leader_task.abort();

    let res2 = follower2_task.await.unwrap();
    let res3 = follower3_task.await.unwrap();
    let res4 = follower4_task.await.unwrap();

    // Verify all followers receive Leader cancelled storage error deterministically
    assert!(
        matches!(&res2, Err(ContextraError::Storage(msg)) if msg.contains("Leader cancelled group commit")),
        "Follower 2 must receive Leader cancelled group commit error, got: {res2:?}"
    );
    assert!(
        matches!(&res3, Err(ContextraError::Storage(msg)) if msg.contains("Leader cancelled group commit")),
        "Follower 3 must receive Leader cancelled group commit error, got: {res3:?}"
    );
    assert!(
        matches!(&res4, Err(ContextraError::Storage(msg)) if msg.contains("Leader cancelled group commit")),
        "Follower 4 must receive Leader cancelled group commit error, got: {res4:?}"
    );

    let _ = storage.close().await;
}

#[tokio::test]
async fn test_follower_timeout_does_not_read_subsequent_batch_committed_flag() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 200_000,   // 200ms
        tx_timeout: Duration::from_millis(50), // short timeout for follower
        ..Default::default()
    };

    let storage = Arc::new(
        LsmStorage::new(config)
            .await
            .expect("LsmStorage creation failed"),
    );

    // Leader 1 (Tx 10)
    let tx10 = TxId::new(10);
    storage.put(tx10, b"l1_k", b"l1_v").await.unwrap();
    let st10 = Arc::clone(&storage);
    let leader1_task = tokio::spawn(async move { st10.commit(tx10).await });

    tokio::time::sleep(Duration::from_millis(20)).await;

    // Follower (Tx 11) joins Leader 1
    let tx11 = TxId::new(11);
    storage.put(tx11, b"f11_k", b"f11_v").await.unwrap();
    let st11 = Arc::clone(&storage);
    let follower_task = tokio::spawn(async move { st11.commit(tx11).await });

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Cancel Leader 1
    leader1_task.abort();

    let follower_res = follower_task.await.unwrap();

    // Must NOT be Ok(()) under any circumstance!
    assert!(
        follower_res.is_err(),
        "Follower must receive error when leader is cancelled, never Ok"
    );

    let _ = storage.close().await;
}
