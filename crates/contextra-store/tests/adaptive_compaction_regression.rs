use contextra_core::{StorageEngine, TxId};
use contextra_store::{LsmConfig, LsmStorage};
use tempfile::TempDir;

#[tokio::test]
async fn test_adaptive_compaction_regression_default_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let data_path = temp_dir.path().to_path_buf();

    // Default LsmConfig has enable_adaptive_compaction = false
    let mut lsm_config = LsmConfig::default();
    lsm_config.path = data_path.clone();
    assert!(!lsm_config.compaction.enable_adaptive_compaction);
    lsm_config.compaction.min_sstables_per_tier = 2;

    let storage = LsmStorage::new(lsm_config).await.unwrap();

    // Perform standard ops
    for i in 1..=4 {
        let tx = TxId(i);
        let key = format!("k{}", i).into_bytes();
        let val = format!("v{}", i).into_bytes();
        storage.put(tx, &key, &val).await.unwrap();
        storage.commit(tx).await.unwrap();
        storage.flush().await.unwrap();
    }

    // Standard STCS compaction
    let compacted = storage.maybe_compact().await.unwrap();

    assert!(compacted, "Standard STCS compaction should run as expected");

    // All key-values readable
    for i in 1..=4 {
        let key = format!("k{}", i).into_bytes();
        let val = format!("v{}", i).into_bytes();
        let res = storage.get(&key).await.unwrap();
        assert_eq!(res.as_deref(), Some(val.as_slice()));
    }

    storage.close().await.unwrap();
}
