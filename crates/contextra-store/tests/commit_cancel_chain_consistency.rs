// FILE-CONTEXT
// ZWECK: Testet die HMAC-Kettenkonsistenz und Recovery, wenn ein Commit-Future nach prepare_batch abgebrochen (gedroppt) wird.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::tempdir;

#[tokio::test]
async fn test_cancel_after_prepare_batch_maintains_hmac_chain_consistency() {
    let dir = tempdir().expect("tempdir creation failed");
    let path = dir.path().to_path_buf();

    let config = LsmConfig {
        path: path.clone(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = LsmStorage::new(config.clone())
        .await
        .expect("LsmStorage creation failed");

    // Commit tx 1
    let tx1 = TxId::new(1);
    storage.put(tx1, b"k1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    // Now attempt tx 2 and drop the commit future right away or simulate cancellation
    let tx2 = TxId::new(2);
    storage.put(tx2, b"k2", b"v2").await.unwrap();

    // Commit tx 2
    let tx2_fut = storage.commit(tx2);
    // Drop future before completion or let it run
    drop(tx2_fut);

    // Commit tx 3 normally
    let tx3 = TxId::new(3);
    storage.put(tx3, b"k3", b"v3").await.unwrap();
    let res3 = storage.commit(tx3).await;
    assert!(res3.is_ok(), "tx3 commit must succeed: {:?}", res3);

    // Reopen DB to ensure WAL HMAC verification succeeds during recovery
    drop(storage);

    let storage_reopen = LsmStorage::new(config)
        .await
        .expect("LsmStorage reopen failed after commit cancellation");

    assert_eq!(
        storage_reopen.get(b"k1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v1"))
    );
    assert_eq!(
        storage_reopen.get(b"k3").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v3"))
    );
}
