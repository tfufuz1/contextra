use contextra_core::{StorageEngine, TxId};
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeScope};
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
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
async fn test_purge_keys_insufficient_disk_space() -> contextra_core::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(0),
    };
    let scope = PurgeScope::Keys(vec![b"key1".to_vec()]);

    let res = storage.purge_keys(scope, 100, Some(&probe)).await;
    assert_eq!(
        res.unwrap_err(),
        PurgeBlocked::InsufficientDiskSpace {
            needed: 1024 * 1024,
            free: 0
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_purge_keys_success() -> contextra_core::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    // Put & commit key
    storage.put(TxId(1), b"purge_target", b"secret").await?;
    storage.commit(TxId(1)).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(100 * 1024 * 1024),
    };
    let scope = PurgeScope::Keys(vec![b"purge_target".to_vec()]);

    let res = storage.purge_keys(scope, 100, Some(&probe)).await;

    // WP-0.1: purge_keys must fail-closed with Unimplemented until real compaction filter is implemented
    assert_eq!(res.unwrap_err(), PurgeBlocked::Unimplemented);

    Ok(())
}
