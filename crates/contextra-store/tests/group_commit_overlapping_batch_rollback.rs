// FILE-CONTEXT
// ZWECK: Testet, dass ein Fehlschlag eines nachfolgenden Batches B (Tx 3) nicht die bereits dauerhaften WAL-Einträge von Batch A (Tx 2) abschneidet.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg(feature = "fault-injection")]

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_overlapping_batch_rollback_does_not_truncate_durable_batch() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();

    let config = LsmConfig {
        path: path.clone(),
        group_commit_window_micros: 0, // Direct single commits (or distinct batches)
        ..Default::default()
    };

    let storage = Arc::new(
        LsmStorage::new(config)
            .await
            .expect("LsmStorage creation failed"),
    );

    // Batch 1 (Tx 1): Baseline commit
    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    // Batch A (Tx 2): Successful commit
    let tx2 = TxId::new(2);
    storage.put(tx2, b"key2", b"val2").await.unwrap();
    storage
        .commit(tx2)
        .await
        .expect("Tx 2 (Batch A) must succeed");

    // Batch B (Tx 3): Fails during WAL append
    let tx3 = TxId::new(3);
    storage.put(tx3, b"key3", b"val3").await.unwrap();

    // Simulate WAL append failure specifically for Tx 3
    storage.simulate_wal_append_failure_for_tx_for_test(3).await;

    let res3 = storage.commit(tx3).await;
    assert!(res3.is_err(), "Tx 3 (Batch B) must fail on WAL append");

    storage.restore_wal_file_handle_for_test().await;

    // Verify in-memory state
    assert_eq!(
        storage.get(b"key1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val1"))
    );
    assert_eq!(
        storage.get(b"key2").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val2"))
    );
    assert_eq!(storage.get(b"key3").await.unwrap(), None);

    // Drop and reopen storage to verify WAL durability and replay integrity
    drop(storage);

    let config_reopen = LsmConfig {
        path: path.clone(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage_reopen = LsmStorage::new(config_reopen)
        .await
        .expect("LsmStorage reopen failed; WAL replay must succeed without corruption");

    assert_eq!(
        storage_reopen.get(b"key1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val1"))
    );
    assert_eq!(
        storage_reopen.get(b"key2").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val2")),
        "Durable Tx 2 entry must exist after reopening store"
    );
    assert_eq!(storage_reopen.get(b"key3").await.unwrap(), None);
}
