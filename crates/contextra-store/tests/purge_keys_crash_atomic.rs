use bytes::Bytes;
use contextra_ports::StorageEngine;
use contextra_types::TxId;
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeScope};
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
async fn test_purge_keys_crash_atomic_recovery() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().to_path_buf();

    {
        let config = LsmConfig {
            path: db_path.clone(),
            ..Default::default()
        };
        let storage = LsmStorage::open(config).await?;

        storage.put(TxId(1), b"k1", b"v1").await?;
        storage.put(TxId(2), b"k2", b"v2").await?;
        storage.commit(TxId(1)).await?;
        storage.commit(TxId(2)).await?;
        StorageEngine::flush(&storage).await?;
    }

    // Re-open storage
    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(100 * 1024 * 1024),
    };
    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);

    let report = storage.purge_keys(scope, 100, Some(&probe)).await.unwrap();

    // Simulate Orchestrator disposing obsolete files
    for file in &report.obsolete_files {
        let _ = contextra_durable_fs::durable_remove(file);
    }

    drop(storage);

    // Re-open after purge
    let storage2 = LsmStorage::open(LsmConfig {
        path: db_path,
        ..Default::default()
    })
    .await?;

    assert_eq!(storage2.get_at_seq(b"k1", 100).await?, None);
    assert_eq!(
        storage2.get_at_seq(b"k2", 100).await?,
        Some(Bytes::from_static(b"v2"))
    );

    Ok(())
}

#[tokio::test]
async fn test_purge_keys_crash_before_manifest_commit_preserves_state() -> contextra_types::Result<()>
{
    let dir = tempdir()?;
    let db_path = dir.path().to_path_buf();

    {
        let config = LsmConfig {
            path: db_path.clone(),
            ..Default::default()
        };
        let storage = LsmStorage::open(config).await?;

        storage.put(TxId(1), b"k1", b"v1").await?;
        storage.put(TxId(2), b"k2", b"v2").await?;
        storage.commit(TxId(1)).await?;
        storage.commit(TxId(2)).await?;
        StorageEngine::flush(&storage).await?;

        // Simulate crash before manifest commit: write an unreferenced orphan SSTable output file to disk
        let orphan_path = db_path.join("sst-compact-00000000000000000099-0000.sst");
        tokio::fs::write(
            &orphan_path,
            b"partial_compacted_data_before_manifest_commit",
        )
        .await?;

        // Process drops before manifest commit occurs
        drop(storage);
    }

    // Re-open storage after simulated crash before manifest commit
    let storage_after_crash = LsmStorage::open(LsmConfig {
        path: db_path,
        ..Default::default()
    })
    .await?;

    // Unchanged state: k1 and k2 are both still present and intact in recovery
    assert_eq!(
        storage_after_crash.get_at_seq(b"k1", 100).await?,
        Some(Bytes::from_static(b"v1"))
    );
    assert_eq!(
        storage_after_crash.get_at_seq(b"k2", 100).await?,
        Some(Bytes::from_static(b"v2"))
    );

    Ok(())
}
