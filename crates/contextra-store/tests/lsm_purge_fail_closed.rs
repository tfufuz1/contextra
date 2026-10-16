use contextra_ports::StorageEngine;
use contextra_types::TxId;
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeScope};
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use std::fs;
use tempfile::tempdir;

struct TestFreeSpaceProbe {
    free_bytes: Option<u64>,
}

impl FreeSpaceProbe for TestFreeSpaceProbe {
    fn available_bytes(&self) -> std::result::Result<Option<u64>, contextra_ports::ProbeError> {
        Ok(self.free_bytes)
    }

    fn free_space_bytes(&self) -> Option<u64> {
        self.free_bytes
    }
}

/// Helper to count `.sst` files in the store directory.
fn count_sst_files(dir_path: &std::path::Path) -> usize {
    fs::read_dir(dir_path)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map_or(false, |ext| ext == "sst" || ext == "sstable")
        })
        .count()
}

#[tokio::test]
async fn test_purge_keys_returns_unimplemented_fail_closed() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    // Put & commit multiple keys
    storage.put(TxId(10), b"key1", b"val1").await?;
    storage.put(TxId(11), b"key2", b"val2").await?;
    storage.commit(TxId(10)).await?;
    storage.commit(TxId(11)).await?;

    // Flush to create SSTable file on disk
    StorageEngine::flush(&storage).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(100 * 1024 * 1024),
    };
    let scope = PurgeScope::Keys(vec![b"key1".to_vec(), b"key2".to_vec()]);

    let res = storage.purge_keys(scope, 100, Some(&probe)).await;
    assert!(res.is_err());

    Ok(())
}

#[tokio::test]
async fn test_purge_keys_active_snapshot_precedence() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    // Acquire active snapshot at sequence 50
    let _snapshot_guard = storage.snapshot_registry.acquire(|| 50);

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(100 * 1024 * 1024),
    };
    let scope = PurgeScope::Keys(vec![b"key1".to_vec()]);

    // Call purge_keys as_of_tx = 100 (min_active_seq 50 <= as_of_tx 100)
    let res = storage.purge_keys(scope, 100, Some(&probe)).await;
    assert!(res.is_err());

    Ok(())
}

#[tokio::test]
async fn test_purge_keys_leaves_sstable_files_untouched() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    // Write data and flush to generate an SSTable on disk
    storage.put(TxId(1), b"target_key", b"target_value").await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    let sst_count_before = count_sst_files(dir.path());
    assert!(
        sst_count_before >= 1,
        "Expected at least one SSTable file after flush"
    );

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(100 * 1024 * 1024),
    };
    let scope = PurgeScope::Keys(vec![b"target_key".to_vec()]);

    let res = storage.purge_keys(scope, 100, Some(&probe)).await;
    assert!(res.is_err());

    let sst_count_after = count_sst_files(dir.path());
    assert_eq!(
        sst_count_before, sst_count_after,
        "SSTable file count must remain completely unchanged"
    );

    Ok(())
}
