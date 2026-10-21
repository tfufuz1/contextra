use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeScope};
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_types::{Result, TxId};
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

#[tokio::test]
async fn test_purge_keys_insufficient_disk_space() -> Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let tx1 = TxId::try_new(1).unwrap();
    storage.put(tx1, b"key1", b"val1").await?;
    storage.commit(tx1).await?;
    StorageEngine::flush(&storage).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(0),
    };
    let scope = PurgeScope::Keys(vec![b"key1".to_vec()]);

    let res = storage.purge_keys(scope, 100, Some(&probe)).await;
    match res.unwrap_err() {
        contextra_ports::erasure::PurgeError::Blocked(PurgeBlocked::InsufficientDiskSpace { needed, free }) => {
            assert!(needed > 0);
            assert_eq!(free, 0);
        }
        other => panic!("expected InsufficientDiskSpace, got {:?}", other),
    }

    Ok(())
}

#[tokio::test]
async fn test_purge_keys_success() -> Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let tx1 = TxId::try_new(1).unwrap();
    // Put & commit key
    storage.put(tx1, b"purge_target", b"secret").await?;
    storage.commit(tx1).await?;
    StorageEngine::flush(&storage).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(100 * 1024 * 1024),
    };
    let scope = PurgeScope::Keys(vec![b"purge_target".to_vec()]);

    let res = storage.purge_keys(scope, 100, Some(&probe)).await;
    assert!(
        res.is_ok(),
        "purge_keys must succeed when disk space is available: {:?}",
        res
    );

    Ok(())
}
