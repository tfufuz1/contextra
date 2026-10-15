use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::time::Duration;
use tempfile::tempdir;
use tokio::time::timeout;

#[tokio::test]
async fn test_commit_cancel_safety_future_dropped_leaves_chain_or_state_corrupt() {
    let dir = tempdir().expect("tempdir creation failed");
    let path = dir.path().to_path_buf();

    let config = LsmConfig {
        path: path.clone(),
        group_commit_window_micros: 0, // Single commit mode
        ..Default::default()
    };

    let storage = LsmStorage::new(config)
        .await
        .expect("LsmStorage creation failed");

    // 1. Commit baseline transaction tx1
    let tx1 = TxId::new(1);
    storage
        .put(tx1, b"key_base", b"val_base")
        .await
        .expect("put tx1 failed");
    storage.commit(tx1).await.expect("commit tx1 failed");

    // 2. Delay or simulate timeout during tx2 commit
    let tx2 = TxId::new(2);
    storage
        .put(tx2, b"key_cancel", b"val_cancel")
        .await
        .expect("put tx2 failed");

    // Drop the commit future abruptly (e.g., tokio timeout)
    let _ = timeout(Duration::from_nanos(1), storage.commit(tx2)).await;

    // 3. Perform tx3 commit after cancellation
    let tx3 = TxId::new(3);
    storage
        .put(tx3, b"key_after", b"val_after")
        .await
        .expect("put tx3 failed");

    let commit_tx3_res = storage.commit(tx3).await;
    // Inspect if tx3 commit succeeds or fails due to HMAC chain mismatch
    tracing::info!("commit_tx3_res after cancellation: {:?}", commit_tx3_res);

    drop(storage);

    // 4. Reopen and verify WAL replay integrity
    let storage_reopen = LsmStorage::new(LsmConfig {
        path: path.clone(),
        ..Default::default()
    })
    .await;

    assert!(
        storage_reopen.is_ok(),
        "Storage reopening must succeed without WAL HMAC chain error"
    );
}
