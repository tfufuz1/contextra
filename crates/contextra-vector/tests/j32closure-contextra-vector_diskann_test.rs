#[cfg(feature = "experimental-diskann")]
use contextra_vector::diskann::{DiskAnnConfig, DiskAnnIndex};

#[cfg(feature = "experimental-diskann")]
#[test]
fn test_diskann_recover_pending_delta_sync() {
    let temp_dir = tempfile::tempdir().expect("tempdir failed");
    let index_path = temp_dir.path().join("test.diskann");
    let config = DiskAnnConfig {
        index_path,
        dimension: 4,
        ..Default::default()
    };
    let index = DiskAnnIndex::try_new(config).expect("DiskAnnIndex creation failed");

    // Without pending.wal, recover_pending_delta_sync should return Ok(0)
    let count = index.recover_pending_delta_sync().expect("recover failed");
    assert_eq!(count, 0);
}
