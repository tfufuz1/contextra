use contextra_ports::StorageEngine;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::TxId;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_commit_failure_closed_flusher_sequence_and_chain_recovery() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0, // Single commit path
        tx_timeout: Duration::from_secs(5),
        ..Default::default()
    };

    let storage = LsmStorage::new(config.clone())
        .await
        .expect("create storage");

    // 1. First successful commit (seq 1, 2)
    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val1").await.expect("put 1");
    storage.commit(tx1).await.expect("commit 1");

    // 2. Close storage WAL flusher actor before commit so append fails
    let _ = storage.close().await;

    let tx_fail = TxId::new(99);
    storage
        .put(tx_fail, b"fail_key", b"fail_val")
        .await
        .expect("put fail");
    let commit_res = storage.commit(tx_fail).await;
    assert!(
        commit_res.is_err(),
        "Commit must fail when WAL flusher is closed"
    );

    // 3. Drop storage handle
    drop(storage);

    // 4. Re-open storage and replay WAL
    let reopened = LsmStorage::new(config).await;
    assert!(
        reopened.is_ok(),
        "Reopening store after failed commit must succeed without WAL corruption or sequence gap: {:?}",
        reopened.err()
    );

    let reopened_store = reopened.expect("reopened");
    assert_eq!(
        reopened_store.get(b"key1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val1"))
    );
    assert_eq!(reopened_store.get(b"fail_key").await.unwrap(), None);
}
