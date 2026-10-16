// FILE-CONTEXT
// ZWECK: Testet Batch-scoped Rollback und Integrität von Immutable MemTables / alten WALs nach WAL-Append-Fehlern.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg(feature = "fault-injection")]

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::tempdir;

static TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn test_batch_scoped_rollback_preserves_prior_batch() {
    let _guard = TEST_LOCK.lock().await;
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

    // Tx 1: Put k1 -> v1 and commit successfully
    let tx1 = TxId::new(1);
    storage.put(tx1, b"k1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    // Inject fault for tx 2
    storage.simulate_wal_append_failure_for_test().await;

    let tx2 = TxId::new(2);
    storage.put(tx2, b"k2", b"v2").await.unwrap();
    let res = storage.commit(tx2).await;
    assert!(res.is_err(), "tx2 commit should fail on WAL append");

    storage.restore_wal_file_handle_for_test().await;

    // k1 must still be readable
    assert_eq!(
        storage.get(b"k1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v1"))
    );
    // k2 must be rolled back
    assert_eq!(storage.get(b"k2").await.unwrap(), None);

    // Reopen storage to verify recovery / durability of k1
    drop(storage);

    let config_reopen = LsmConfig {
        path: path.clone(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage_reopen = LsmStorage::new(config_reopen)
        .await
        .expect("LsmStorage reopen failed");

    assert_eq!(
        storage_reopen.get(b"k1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v1"))
    );
    assert_eq!(storage_reopen.get(b"k2").await.unwrap(), None);
}

#[tokio::test]
async fn test_rollback_preserves_immutable_memtables() {
    let _guard = TEST_LOCK.lock().await;
    let dir = tempdir().expect("tempdir creation failed");
    let path = dir.path().to_path_buf();

    let config = LsmConfig {
        path: path.clone(),
        memtable_size_limit: 1024 * 1024,
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = LsmStorage::new(config)
        .await
        .expect("LsmStorage creation failed");

    // Tx 1 & 2 committed
    let tx1 = TxId::new(1);
    storage.put(tx1, b"k1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"k2", b"v2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    // Request flush to push active memtable to immutable memtables without finishing SST write immediately
    storage.request_flush();

    // Inject fault for tx 3
    storage.simulate_wal_append_failure_for_test().await;

    let tx3 = TxId::new(3);
    storage.put(tx3, b"k3", b"v3").await.unwrap();
    let res = storage.commit(tx3).await;
    assert!(res.is_err(), "tx3 commit should fail on WAL append");

    storage.restore_wal_file_handle_for_test().await;

    // k1 and k2 must remain readable despite rollback of tx3
    assert_eq!(
        storage.get(b"k1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v1"))
    );
    assert_eq!(
        storage.get(b"k2").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v2"))
    );
    assert_eq!(storage.get(b"k3").await.unwrap(), None);
}
