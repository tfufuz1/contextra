// FILE-CONTEXT
// ZWECK: Testet, dass ein Follower, der per Timeout aus der Group-Commit-Queue ausscheidet, keine HMAC-Kette korrumpiert.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_follower_timeout_chain_integrity() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();

    let config = LsmConfig {
        path: path.clone(),
        group_commit_window_micros: 200_000, // 200ms window
        tx_timeout: std::time::Duration::from_millis(10), // Short 10ms follower timeout
        ..Default::default()
    };

    let storage = Arc::new(
        LsmStorage::new(config)
            .await
            .expect("LsmStorage creation failed"),
    );

    // Leader Tx 1 starts group commit
    let tx1 = TxId::new(1);
    storage.put(tx1, b"leader_k", b"leader_v").await.unwrap();

    // Follower Tx 2
    let tx2 = TxId::new(2);
    storage
        .put(tx2, b"follower_k", b"follower_v")
        .await
        .unwrap();

    let st1 = Arc::clone(&storage);
    let leader_handle = tokio::spawn(async move { st1.commit(tx1).await });

    // Small delay to ensure Leader sets up queue and enters window wait
    tokio::time::sleep(std::time::Duration::from_millis(2)).await;

    // Follower Tx 2 enters queue while Leader is sleeping in 200ms window
    let st2 = Arc::clone(&storage);
    let follower_handle = tokio::spawn(async move { st2.commit(tx2).await });

    let res2 = follower_handle.await.unwrap();
    assert!(
        res2.is_err(),
        "Follower with 10ms tx_timeout should time out while Leader waits in 200ms window"
    );

    let res1 = leader_handle.await.unwrap();
    assert!(res1.is_ok(), "Leader commit must succeed");

    // Subsequent Tx 3 commits
    let tx3 = TxId::new(3);
    storage.put(tx3, b"next_k", b"next_v").await.unwrap();
    storage.commit(tx3).await.unwrap();

    // Reopen store to verify WAL replay has NO HMAC chain breakage!
    drop(storage);

    let config_reopen = LsmConfig {
        path: path.clone(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage_reopen = LsmStorage::new(config_reopen)
        .await
        .expect("Reopen storage failed; HMAC chain must be intact despite follower timeout");

    assert_eq!(
        storage_reopen.get(b"leader_k").await.unwrap(),
        Some(bytes::Bytes::from_static(b"leader_v"))
    );
    assert_eq!(
        storage_reopen.get(b"next_k").await.unwrap(),
        Some(bytes::Bytes::from_static(b"next_v"))
    );
}
