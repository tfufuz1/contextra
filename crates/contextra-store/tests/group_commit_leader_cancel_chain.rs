// FILE-CONTEXT
// ZWECK: Testet, dass der Abbruch (Cancellation) der Leader-Future das Queue-Integritäts-Verhalten nicht korrumpiert und wartende Follower geordnet fehlschlagen.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_leader_cancellation_chain_integrity() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();

    let config = LsmConfig {
        path: path.clone(),
        group_commit_window_micros: 200_000, // 200ms group commit window
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

    // Wait slightly so Leader enters group commit window wait
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // Follower Tx 2 enters queue
    let tx2 = TxId::new(2);
    storage.put(tx2, b"foll_k", b"foll_v").await.unwrap();

    let st2 = Arc::clone(&storage);
    let follower_task = tokio::spawn(async move { st2.commit(tx2).await });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // ABORT the Leader task while waiting in group window
    leader_task.abort();

    // Follower task should receive error or timeout gracefully, not hang forever
    let follower_res = follower_task.await;
    assert!(follower_res.is_ok(), "Follower task should complete join");

    // Subsequent Tx 3 commit must succeed cleanly
    let tx3 = TxId::new(3);
    storage.put(tx3, b"fresh_k", b"fresh_v").await.unwrap();
    let res3 = storage.commit(tx3).await;
    assert!(
        res3.is_ok(),
        "Subsequent commit after leader cancel must succeed"
    );

    // Close and drop storage, allowing Tokio runtime to complete task cleanup
    let _ = storage.close().await;
    drop(storage);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let config_reopen = LsmConfig {
        path: path.clone(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage_reopen = LsmStorage::new(config_reopen)
        .await
        .expect("Reopen store failed; HMAC chain must be intact after leader cancellation");

    assert_eq!(
        storage_reopen.get(b"fresh_k").await.unwrap(),
        Some(bytes::Bytes::from_static(b"fresh_v"))
    );
}
