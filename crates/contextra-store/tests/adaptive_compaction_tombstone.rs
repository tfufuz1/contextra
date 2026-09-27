use contextra_core::{StorageEngine, TxId};
use contextra_store::{LsmConfig, LsmStorage};
use tempfile::TempDir;

#[tokio::test]
async fn test_adaptive_compaction_tombstone_protection() {
    let temp_dir = TempDir::new().unwrap();
    let data_path = temp_dir.path().to_path_buf();

    let mut lsm_config = LsmConfig::default();
    lsm_config.path = data_path.clone();
    lsm_config.compaction.enable_adaptive_compaction = true;
    lsm_config.compaction.adaptive_read_ratio_threshold = 0.50;
    lsm_config.compaction.min_sstables_per_tier = 2;

    let storage = LsmStorage::new(lsm_config).await.unwrap();

    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    // Put key "k1" in tx1
    storage.put(tx1, b"k1", b"val1").await.unwrap();
    storage.commit(tx1).await.unwrap();
    storage.flush().await.unwrap();

    // Pin checkpoint / snapshot at seq 1 (or tx1)
    let last_seq = storage.last_seq_no().await.unwrap();
    storage.pin_checkpoint(last_seq).await.unwrap();

    // Delete key "k1" in tx2
    storage.delete(tx2, b"k1").await.unwrap();
    storage.commit(tx2).await.unwrap();
    storage.flush().await.unwrap();

    // Now min_active_seqno is pinned to last_seq (1)
    let min_active = storage.snapshot_registry.min_active_seqno();
    assert_eq!(min_active, last_seq);

    // Trigger compaction via storage engine
    let compacted = storage.maybe_compact().await.unwrap();

    assert!(compacted, "Compaction should be performed");

    // Verify key "k1" at seq 1 is STILL readable via scan_prefix_at
    let scanned = storage.scan_prefix_at(b"k1", last_seq).await.unwrap();
    assert_eq!(scanned.len(), 1);
    assert_eq!(scanned[0].0, b"k1");
    assert_eq!(scanned[0].1, b"val1");

    storage.close().await.unwrap();
}
