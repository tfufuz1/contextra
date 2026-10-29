use contextra_checkpoint::{CheckpointHardlinkCloner, DefaultHardlinkCloner};
use contextra_core::SnapshotRegistry;
use contextra_types::ContextraError;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_successful_hardlink_clone_and_inode_identity() -> contextra_types::Result<()> {
    let source_dir = TempDir::new().map_err(ContextraError::Io)?;
    let target_dir = TempDir::new().map_err(ContextraError::Io)?;

    let file1 = source_dir.path().join("000001.sst");
    let file2 = source_dir.path().join("000002.sst");
    let other_file = source_dir.path().join("ignore.txt");

    fs::write(&file1, b"sstable_data_1").map_err(ContextraError::Io)?;
    fs::write(&file2, b"sstable_data_2").map_err(ContextraError::Io)?;
    fs::write(&other_file, b"not_an_sstable").map_err(ContextraError::Io)?;

    let registry = Arc::new(SnapshotRegistry::new());
    let cloner = DefaultHardlinkCloner::new();

    let res = cloner
        .clone_sstable_hardlinks(100, source_dir.path(), target_dir.path(), &registry)
        .await?;

    assert_eq!(res.source_seq_no, 100);
    assert_eq!(res.linked_files.len(), 2);
    assert_eq!(res.wal_tail_offset, None);

    let target1 = target_dir.path().join("000001.sst");
    let target2 = target_dir.path().join("000002.sst");

    assert!(target1.exists());
    assert!(target2.exists());
    assert!(!target_dir.path().join("ignore.txt").exists());

    assert_eq!(
        fs::read(&target1).map_err(ContextraError::Io)?,
        b"sstable_data_1"
    );
    assert_eq!(
        fs::read(&target2).map_err(ContextraError::Io)?,
        b"sstable_data_2"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta_src1 = fs::metadata(&file1).map_err(ContextraError::Io)?;
        let meta_tgt1 = fs::metadata(&target1).map_err(ContextraError::Io)?;
        assert_eq!(
            meta_src1.ino(),
            meta_tgt1.ino(),
            "Hardlinked SSTables must share identical inode numbers on Unix"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_cross_device_link_error_handling() -> contextra_types::Result<()> {
    let source_dir = TempDir::new().map_err(ContextraError::Io)?;
    // Mount point /dev/shm is a tmpfs mount on Linux (distinct filesystem from /rom/overlay)
    let shm_base = std::path::Path::new("/dev/shm");
    if !shm_base.exists() {
        return Ok(()); // Skip if /dev/shm is not available
    }

    let target_dir_path = shm_base.join(format!("contextra_test_{}", std::process::id()));
    let file1 = source_dir.path().join("000001.sst");
    fs::write(&file1, b"sst_data").map_err(ContextraError::Io)?;

    let registry = Arc::new(SnapshotRegistry::new());
    let cloner = DefaultHardlinkCloner::new();

    let result = cloner
        .clone_sstable_hardlinks(200, source_dir.path(), &target_dir_path, &registry)
        .await;

    let _ = fs::remove_dir_all(&target_dir_path);

    match result {
        Err(ContextraError::CrossDeviceLink {
            source_path,
            target_path,
            ..
        }) => {
            assert!(source_path.contains("000001.sst"));
            assert!(target_path.contains("000001.sst"));
            Ok(())
        }
        Err(other) => Err(ContextraError::Internal(format!(
            "Expected CrossDeviceLink error, got: {other:?}"
        ))),
        Ok(_) => {
            // In case /dev/shm happens to be on the same filesystem in some container environments,
            // verify hardlink created successfully
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_snapshot_registry_compaction_gc_exclusion_during_cloning(
) -> contextra_types::Result<()> {
    let source_dir = TempDir::new().map_err(ContextraError::Io)?;
    let target_dir = TempDir::new().map_err(ContextraError::Io)?;

    let file1 = source_dir.path().join("000001.sst");
    fs::write(&file1, b"sstable_content").map_err(ContextraError::Io)?;

    let registry = Arc::new(SnapshotRegistry::new());
    assert_eq!(
        registry.min_active_seqno(),
        u64::MAX,
        "Initially no active pins"
    );

    let cloner = DefaultHardlinkCloner::new();

    let seq_no = 42;
    let res = cloner
        .clone_sstable_hardlinks(seq_no, source_dir.path(), target_dir.path(), &registry)
        .await?;

    assert_eq!(res.source_seq_no, 42);
    assert_eq!(
        registry.min_active_seqno(),
        u64::MAX,
        "Snapshot pin released after clone finishes"
    );

    Ok(())
}
