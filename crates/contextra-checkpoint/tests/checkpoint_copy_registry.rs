// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Tests für CopyRegistry Verankerung und purge_checkpoint Hardlink-Entfernung mit nlink==1 Garantie.

use contextra_checkpoint::{purge_registered_checkpoint, DefaultHardlinkCloner};
use contextra_mvcc::SnapshotRegistry;
use contextra_store::copy_registry::CopyRegistry;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_purge_checkpoint_reduces_nlink_and_unlinks_target() {
    let src = tempdir().unwrap();
    let tgt = tempdir().unwrap();
    let reg_dir = tempdir().unwrap();

    let sst_path = src.path().join("000005.sst");
    let original_content = b"sstable_5_data_content";
    File::create(&sst_path)
        .unwrap()
        .write_all(original_content)
        .unwrap();

    let cloner = DefaultHardlinkCloner::new();
    let snap_registry = Arc::new(SnapshotRegistry::new());
    let mut copy_registry = CopyRegistry::open(reg_dir.path()).await.unwrap();

    let res = cloner
        .clone_sstable_hardlinks_registered(
            5,
            src.path(),
            tgt.path(),
            &snap_registry,
            &mut copy_registry,
            500,
        )
        .await
        .unwrap();

    let target_file = tgt.path().join("000005.sst");
    assert!(target_file.exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = std::fs::metadata(&sst_path).unwrap();
        assert_eq!(meta.nlink(), 2, "After hardlink clone, nlink must be 2");
    }

    // Execute purge_registered_checkpoint
    purge_registered_checkpoint(res.copy_id, &mut copy_registry, tgt.path())
        .await
        .unwrap();

    // Target hardlink should be removed
    assert!(!target_file.exists());

    // Registry should be empty
    assert_eq!(copy_registry.list().len(), 0);

    // Source file must remain intact with nlink == 1
    assert!(sst_path.exists());
    let read_back = std::fs::read(&sst_path).unwrap();
    assert_eq!(
        read_back, original_content,
        "Source file content must NOT be overwritten"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = std::fs::metadata(&sst_path).unwrap();
        assert_eq!(
            meta.nlink(),
            1,
            "After purge_checkpoint, source file nlink must be 1"
        );
    }

    // Idempotency test: second purge on already purged copy_id returns Ok(())
    assert!(
        purge_registered_checkpoint(res.copy_id, &mut copy_registry, tgt.path())
            .await
            .is_ok()
    );
}
