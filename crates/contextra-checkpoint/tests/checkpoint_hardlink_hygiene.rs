// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Tests für Hardlink-Cloner Hygiene — SSTable-Filterung (.sst vs .tmp), Target-Removal Fehler und Registration.

use contextra_checkpoint::{CheckpointHardlinkCloner, DefaultHardlinkCloner};
use contextra_mvcc::SnapshotRegistry;
use contextra_store::copy_registry::CopyRegistry;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_hardlink_cloner_filters_tmp_files() {
    let src = tempdir().unwrap();
    let tgt = tempdir().unwrap();

    // Create a valid .sst file and a .tmp file
    let sst_path = src.path().join("000001.sst");
    let tmp_path = src.path().join("000002.tmp");
    let partial_sst_tmp = src.path().join("000003.sst.tmp");

    File::create(&sst_path)
        .unwrap()
        .write_all(b"sst_data")
        .unwrap();
    File::create(&tmp_path)
        .unwrap()
        .write_all(b"tmp_data")
        .unwrap();
    File::create(&partial_sst_tmp)
        .unwrap()
        .write_all(b"partial_data")
        .unwrap();

    let cloner = DefaultHardlinkCloner::new();
    let registry = Arc::new(SnapshotRegistry::new());

    let res = cloner
        .clone_sstable_hardlinks(1, src.path(), tgt.path(), &registry)
        .await
        .unwrap();

    // Only 000001.sst should be cloned
    assert_eq!(res.linked_files.len(), 1);
    assert_eq!(res.linked_files[0].file_name().unwrap(), "000001.sst");
    assert!(!tgt.path().join("000002.tmp").exists());
    assert!(!tgt.path().join("000003.sst.tmp").exists());
}

#[tokio::test]
async fn test_hardlink_cloner_registered_and_target_overwrite() {
    let src = tempdir().unwrap();
    let tgt = tempdir().unwrap();
    let reg_dir = tempdir().unwrap();

    let sst_path = src.path().join("000010.sst");
    File::create(&sst_path)
        .unwrap()
        .write_all(b"data10")
        .unwrap();

    let cloner = DefaultHardlinkCloner::new();
    let snap_registry = Arc::new(SnapshotRegistry::new());
    let mut copy_registry = CopyRegistry::open(reg_dir.path()).await.unwrap();

    let res = cloner
        .clone_sstable_hardlinks_registered(
            10,
            src.path(),
            tgt.path(),
            &snap_registry,
            &mut copy_registry,
            100,
        )
        .await
        .unwrap();

    assert_eq!(res.linked_files.len(), 1);
    assert_ne!(res.copy_id, [0u8; 16]);
    assert_eq!(res.file_ids.len(), 1);

    // Verify copy is registered in copy_registry
    assert_eq!(copy_registry.list().len(), 1);
    assert_eq!(copy_registry.list()[0].copy_id, res.copy_id);

    // Cloning again to same target should successfully overwrite target link
    let res2 = cloner
        .clone_sstable_hardlinks_registered(
            10,
            src.path(),
            tgt.path(),
            &snap_registry,
            &mut copy_registry,
            101,
        )
        .await
        .unwrap();

    assert_eq!(res2.linked_files.len(), 1);
    assert_eq!(copy_registry.list().len(), 2);
}
