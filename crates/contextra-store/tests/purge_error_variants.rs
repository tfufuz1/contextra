// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Targeted tests for each PurgeError variant in LsmStorage::purge_keys.

use contextra_ports::erasure::{FreeSpaceProbe, PurgeBlocked, PurgeError};
use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_types::TxId;
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

#[tokio::test]
async fn test_purge_error_snapshot_pinned() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let _lease = storage.snapshot_registry.acquire(|| 10);
    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);

    let res = storage.purge_keys(scope, 20, None).await;
    match res {
        Err(PurgeError::Blocked(PurgeBlocked::ActiveSnapshot { min_seq })) => {
            assert_eq!(min_seq, 10);
        }
        other => panic!("Expected PurgeError::Blocked(ActiveSnapshot), got {:?}", other),
    }

    Ok(())
}

#[tokio::test]
async fn test_purge_error_insufficient_disk_space() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    storage.put(TxId(1), b"k1", b"val1").await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(0),
    };
    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);

    let res = storage.purge_keys(scope, 10, Some(&probe)).await;
    match res {
        Err(PurgeError::Blocked(PurgeBlocked::InsufficientDiskSpace { needed, free })) => {
            assert!(needed >= 1024 * 1024);
            assert_eq!(free, 0);
        }
        other => panic!("Expected PurgeError::Blocked(InsufficientDiskSpace), got {:?}", other),
    }

    Ok(())
}

#[tokio::test]
async fn test_purge_error_read_error() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    storage.put(TxId(1), b"corrupt_me", b"val").await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    // Corrupt all .sst files by overwriting with garbage bytes
    for entry in fs::read_dir(dir.path())? {
        let entry = entry?;
        if entry.path().extension().map_or(false, |ext| ext == "sst") {
            fs::write(entry.path(), b"INVALID_GARBAGE_BYTES_CORRUPTED")?;
        }
    }

    let scope = PurgeScope::Keys(vec![b"corrupt_me".to_vec()]);
    let res = storage.purge_keys(scope, 10, None).await;
    match res {
        Err(PurgeError::Read { file, .. }) => {
            assert!(file.contains("sst"));
        }
        other => panic!("Expected PurgeError::Read, got {:?}", other),
    }

    Ok(())
}

#[tokio::test]
async fn test_purge_error_write_error() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    storage.put(TxId(1), b"key_a", b"val_a").await?;
    storage.put(TxId(1), b"purge_target", b"val_target").await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    // Make the DB directory read-only so creating temp SSTable files fails
    let mut permissions = fs::metadata(dir.path())?.permissions();
    permissions.set_readonly(true);
    let _ = fs::set_permissions(dir.path(), permissions.clone());

    let scope = PurgeScope::Keys(vec![b"purge_target".to_vec()]);
    let res = storage.purge_keys(scope, 10, None).await;

    // Reset permissions so tempdir cleanup works
    permissions.set_readonly(false);
    let _ = fs::set_permissions(dir.path(), permissions);

    match res {
        Err(PurgeError::Write { file, .. }) => {
            assert!(file.contains(".tmp") || file.contains("sst"));
        }
        other => panic!("Expected PurgeError::Write, got {:?}", other),
    }

    Ok(())
}

#[tokio::test]
async fn test_purge_error_manifest() -> contextra_types::Result<()> {
    // Test that Manifest error variant exists, formats correctly and holds details
    let err = PurgeError::Manifest("Failed to write manifest entry".to_string());
    assert_eq!(err.to_string(), "Manifest error: Failed to write manifest entry");
    match err {
        PurgeError::Manifest(msg) => assert_eq!(msg, "Failed to write manifest entry"),
        _ => panic!("Expected Manifest variant"),
    }
    Ok(())
}

#[tokio::test]
async fn test_purge_error_compaction_busy() -> contextra_types::Result<()> {
    // Test that CompactionBusy error variant exists and formats correctly
    let err = PurgeBlocked::CompactionBusy;
    assert_eq!(err.to_string(), "Compaction engine busy, retry purge later");
    Ok(())
}
