use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::tempdir;

#[tokio::test]
async fn test_batch_rollback_does_not_truncate_valid_history() {
    let dir = tempdir().expect("tempdir creation failed");
    let path = dir.path().to_path_buf();

    let config = LsmConfig {
        path: path.clone(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = LsmStorage::new(config)
        .await
        .expect("LsmStorage creation failed");

    // Commit tx 1 & 2
    let tx1 = TxId::new(1);
    storage.put(tx1, b"k1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"k2", b"v2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    // Force failure on tx 3
    storage.simulate_wal_append_failure_for_test().await;

    let tx3 = TxId::new(3);
    storage.put(tx3, b"k3", b"v3").await.unwrap();
    let res = storage.commit(tx3).await;
    assert!(res.is_err(), "tx3 commit must fail");

    storage.restore_wal_file_handle_for_test().await;

    // Verify k1 and k2 are still readable
    assert_eq!(storage.get(b"k1").await.unwrap(), Some(bytes::Bytes::from_static(b"v1")));
    assert_eq!(storage.get(b"k2").await.unwrap(), Some(bytes::Bytes::from_static(b"v2")));
    assert_eq!(storage.get(b"k3").await.unwrap(), None);
}
