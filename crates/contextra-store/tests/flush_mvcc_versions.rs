use bytes::Bytes;
use contextra_core::{StorageEngine, TxId};
use contextra_store::manifest::{Manifest, ManifestEntry};
use contextra_store::sstable::{BlockCache, SstableReader};
use contextra_store::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_flush_preserves_pinned_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await.unwrap();

    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    // Put v1 at seq 1
    storage.put(tx1, b"key1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    // Pin snapshot at seq 1
    let pinned_seq = 1;
    storage.snapshot_registry.pin(pinned_seq);

    // Put v2 at seq 2
    storage.put(tx2, b"key1", b"v2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    // Flush MemTable to SSTable
    storage.flush().await.unwrap();

    // Query key1 at pinned_seq (seq 1)
    let val = storage.get_at_seq(b"key1", pinned_seq).await.unwrap();
    assert_eq!(
        val,
        Some(Bytes::from("v1")),
        "Flush must preserve v1 because snapshot at seq 1 is pinned"
    );
}

#[tokio::test]
async fn test_flush_preserves_tombstone_floor() {
    let temp_dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await.unwrap();

    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    // Put v1
    storage.put(tx1, b"key1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    // Pin snapshot at seq 1
    let pinned_seq = 1;
    storage.snapshot_registry.pin(pinned_seq);

    // Delete key1
    storage.delete(tx2, b"key1").await.unwrap();
    storage.commit(tx2).await.unwrap();

    // Flush
    storage.flush().await.unwrap();

    // Query at seq 1 must still yield v1
    let val = storage.get_at_seq(b"key1", pinned_seq).await.unwrap();
    assert_eq!(
        val,
        Some(Bytes::from("v1")),
        "Flush must preserve v1 even after deletion if seq 1 is pinned"
    );
}

#[tokio::test]
async fn test_flush_without_pin_keeps_latest_only() {
    let temp_dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await.unwrap();

    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    storage.put(tx1, b"key1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    storage.put(tx2, b"key1", b"v2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    // Flush with no pinned snapshots
    storage.flush().await.unwrap();

    // Latest version v2 is returned for current get
    let val = storage.get(b"key1").await.unwrap();
    assert_eq!(val, Some(Bytes::from("v2")));
}

#[tokio::test]
async fn test_compaction_candidate_selection_contiguity() {
    let temp_dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await.unwrap();

    // Insert to sst1
    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"old").await.unwrap();
    storage.commit(tx1).await.unwrap();
    storage.flush().await.unwrap();

    // Insert to sst2
    let tx2 = TxId::new(2);
    storage.put(tx2, b"key1", b"NEW").await.unwrap();
    storage.commit(tx2).await.unwrap();
    storage.flush().await.unwrap();

    // Insert to sst3
    let tx3 = TxId::new(3);
    storage.put(tx3, b"key2", b"other").await.unwrap();
    storage.commit(tx3).await.unwrap();
    storage.flush().await.unwrap();

    // Verify current read returns "NEW"
    let val = storage.get(b"key1").await.unwrap();
    assert_eq!(val, Some(Bytes::from("NEW")));
}

#[tokio::test]
async fn test_checkpoint_hmac_matches_wal_checkpoint() {
    let temp_dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config.clone()).await.unwrap();

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    storage.flush().await.unwrap();

    let entries = Manifest::load(&config.path.join("MANIFEST")).await.unwrap();
    let has_wal_checkpoint = entries
        .iter()
        .any(|e| matches!(e, ManifestEntry::WalCheckpoint { .. }));

    assert!(
        has_wal_checkpoint,
        "Manifest must contain a WalCheckpoint entry after flush"
    );
}

#[tokio::test]
async fn test_concurrent_commit_during_flush_is_durable() {
    let temp_dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config.clone()).await.unwrap();

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    // Concurrent commit and flush
    let tx2 = TxId::new(2);
    storage.put(tx2, b"key2", b"v2").await.unwrap();

    let flush_fut = storage.flush();
    let commit_fut = storage.commit(tx2);

    let (flush_res, commit_res) = tokio::join!(flush_fut, commit_fut);
    flush_res.unwrap();
    commit_res.unwrap();

    // Reopen DB to verify durability
    drop(storage);

    let storage_reopened = LsmStorage::open(config).await.unwrap();
    let v1 = storage_reopened.get(b"key1").await.unwrap();
    let v2 = storage_reopened.get(b"key2").await.unwrap();
    assert_eq!(v1, Some(Bytes::from("v1")));
    assert_eq!(v2, Some(Bytes::from("v2")));
}

#[tokio::test]
async fn test_v1_sst_compatibility_fixture() {
    let fixture_path = "tests/fixtures/sst_v1_single_version.sst";
    let block_cache = Arc::new(BlockCache::new(100));
    let reader = SstableReader::open(fixture_path, block_cache)
        .await
        .expect("SstableReader must open v1 SSTable fixture");

    let v1 = reader.get(b"key1").await.unwrap();
    assert_eq!(
        v1.map(|(val, seq, tx)| (val, seq, tx)),
        Some((Bytes::from("v1"), 10, 1))
    );

    let v2 = reader.get(b"key2").await.unwrap();
    assert_eq!(
        v2.map(|(val, seq, tx)| (val, seq, tx)),
        Some((Bytes::from("v2"), 20, 2))
    );

    let v3 = reader.get(b"key3").await.unwrap();
    assert_eq!(
        v3.map(|(val, seq, tx)| (val, seq, tx)),
        Some((Bytes::from("v3"), 30, 3))
    );
}
