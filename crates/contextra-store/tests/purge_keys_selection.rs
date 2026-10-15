mod purge_oracle_helper;

use bytes::Bytes;
use contextra_core::{StorageEngine, TxId};
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeScope};
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_store::sstable::witness_types::SstScopeSpec;
use purge_oracle_helper::verify_purge_oracle;
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
async fn test_purge_keys_selection_put_l0() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    storage.put(TxId(1), b"target_key", b"value1").await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(100 * 1024 * 1024),
    };
    let scope = PurgeScope::Keys(vec![b"target_key".to_vec()]);

    let report = storage.purge_keys(scope, 100, Some(&probe)).await.unwrap();
    assert!(report.records_purged >= 1);

    for file in &report.obsolete_files {
        let _ = contextra_durable_fs::durable_remove(file);
    }

    let scope_spec = SstScopeSpec {
        explicit: vec![b"target_key".to_vec()],
        prefixes: vec![],
    };
    verify_purge_oracle(dir.path(), &scope_spec, 100);

    Ok(())
}

#[tokio::test]
async fn test_purge_keys_prefix_isolation() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    storage.put(TxId(1), b"__col:a:123", b"val_a").await?;
    storage.put(TxId(2), b"__col:ab:123", b"val_ab").await?;
    storage.commit(TxId(1)).await?;
    storage.commit(TxId(2)).await?;
    StorageEngine::flush(&storage).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(100 * 1024 * 1024),
    };
    let scope = PurgeScope::Prefixes(vec![b"__col:a:".to_vec()]);

    let report = storage.purge_keys(scope, 100, Some(&probe)).await.unwrap();

    for file in &report.obsolete_files {
        let _ = contextra_durable_fs::durable_remove(file);
    }

    let scope_a = SstScopeSpec {
        explicit: vec![],
        prefixes: vec![b"__col:a:".to_vec()],
    };
    verify_purge_oracle(dir.path(), &scope_a, 100);

    let val_ab = storage.get_at_seq(b"__col:ab:123", 100).await?;
    assert_eq!(val_ab, Some(Bytes::from_static(b"val_ab")));

    Ok(())
}

#[tokio::test]
async fn test_purge_keys_preserves_newer_versions() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    storage.put(TxId(10), b"key1", b"old_val").await?;
    storage.commit(TxId(10)).await?;
    StorageEngine::flush(&storage).await?;

    storage.put(TxId(100), b"key1", b"new_val").await?;
    storage.commit(TxId(100)).await?;
    StorageEngine::flush(&storage).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(100 * 1024 * 1024),
    };
    let scope = PurgeScope::Keys(vec![b"key1".to_vec()]);

    let report = storage.purge_keys(scope, 50, Some(&probe)).await.unwrap();

    for file in &report.obsolete_files {
        let _ = contextra_durable_fs::durable_remove(file);
    }

    let val = storage.get_at_seq(b"key1", 100).await?;
    assert_eq!(val, Some(Bytes::from_static(b"new_val")));

    Ok(())
}

#[tokio::test]
async fn test_purge_keys_insufficient_space_leaves_data_untouched() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    storage.put(TxId(1), b"key1", b"val1").await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(10),
    };
    let scope = PurgeScope::Keys(vec![b"key1".to_vec()]);

    let err = storage
        .purge_keys(scope, 100, Some(&probe))
        .await
        .unwrap_err();
    assert!(matches!(err, PurgeBlocked::InsufficientDiskSpace { .. }));

    let val = storage.get_at_seq(b"key1", 100).await?;
    assert_eq!(val, Some(Bytes::from_static(b"val1")));

    Ok(())
}

#[tokio::test]
async fn test_purge_keys_active_snapshot_leaves_data_untouched() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    storage.put(TxId(10), b"key1", b"val1").await?;
    storage.commit(TxId(10)).await?;
    StorageEngine::flush(&storage).await?;

    let _snap = storage.snapshot_registry.acquire(|| 10);

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(100 * 1024 * 1024),
    };
    let scope = PurgeScope::Keys(vec![b"key1".to_vec()]);

    let err = storage
        .purge_keys(scope, 100, Some(&probe))
        .await
        .unwrap_err();
    assert_eq!(err, PurgeBlocked::ActiveSnapshot { min_seq: 10 });

    let val = storage.get_at_seq(b"key1", 100).await?;
    assert_eq!(val, Some(Bytes::from_static(b"val1")));

    Ok(())
}
