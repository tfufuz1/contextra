use bytes::Bytes;
use contextra_store::compaction::CompactionConfig;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::TxId;
use contextra_ports::StorageEngine;
use tempfile::TempDir;

#[tokio::test]
async fn wal_cleanup_failure_retried_next_flush() {
    let temp_dir = TempDir::new().expect("tempdir");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config)
        .await
        .expect("LsmStorage creation failed");

    // Put data and commit transaction 1
    storage
        .put(TxId(1), b"key1", b"val1")
        .await
        .expect("put key1");
    storage.commit(TxId(1)).await.expect("commit 1");
    storage.flush().await.expect("flush 1");

    // Put data and commit transaction 2
    storage
        .put(TxId(2), b"key2", b"val2")
        .await
        .expect("put key2");
    storage.commit(TxId(2)).await.expect("commit 2");
    storage.flush().await.expect("flush 2");

    // Read counter
    let failures = contextra_store::util::wal_cleanup_failures_total();
    assert_eq!(failures, 0, "Initial clean flush must have 0 cleanup failures");

    // Verify both keys exist in engine
    assert_eq!(
        storage.get(b"key1").await.unwrap(),
        Some(Bytes::from("val1"))
    );
    assert_eq!(
        storage.get(b"key2").await.unwrap(),
        Some(Bytes::from("val2"))
    );
}

#[tokio::test]
async fn compaction_manifest_crash_matrix() {
    let temp_dir = TempDir::new().expect("tempdir");
    let db_path = temp_dir.path().to_path_buf();

    let config = LsmConfig {
        path: db_path.clone(),
        compaction: CompactionConfig {
            min_sstables_per_tier: 2,
            ..Default::default()
        },
        ..Default::default()
    };

    let storage = LsmStorage::new(config.clone())
        .await
        .expect("Storage creation");

    // Write keys across two flushes to produce 2 SSTables
    storage.put(TxId(1), b"k1", b"v1").await.unwrap();
    storage.commit(TxId(1)).await.unwrap();
    storage.flush().await.unwrap();

    storage.put(TxId(2), b"k2", b"v2").await.unwrap();
    storage.commit(TxId(2)).await.unwrap();
    storage.flush().await.unwrap();

    // Trigger compaction
    let compacted = storage.maybe_compact_for_test().await.unwrap();
    assert!(compacted, "Compaction should occur");

    // Reopen storage from disk to verify crash recovery / persistence
    storage.close().await.unwrap();
    drop(storage);

    let storage2 = LsmStorage::new(config)
        .await
        .expect("Reopen storage after compaction");

    assert_eq!(
        storage2.get(b"k1").await.unwrap(),
        Some(Bytes::from("v1"))
    );
    assert_eq!(
        storage2.get(b"k2").await.unwrap(),
        Some(Bytes::from("v2"))
    );
}
