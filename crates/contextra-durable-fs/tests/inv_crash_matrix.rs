use contextra_durable_fs::fault_injector::FaultInjectingVfs;
use contextra_durable_fs::vfs::{StdVfs, Vfs};
use tempfile::tempdir;

#[test]
fn test_inv_vfs_atomic_1_crash_matrix() {
    let dir = tempdir().expect("tempdir");
    let target = dir.path().join("atomic_target.txt");
    let initial_data = b"initial-data-version-1";
    let new_data = b"new-data-version-2";

    let vfs = FaultInjectingVfs::new(StdVfs);

    // Initialen Zustand herstellen
    vfs.write_atomic(&target, initial_data)
        .expect("write initial");
    assert_eq!(vfs.read(&target).expect("read initial"), initial_data);

    // 1. Crash vor dem Schreiben (DiskFull)
    vfs.inject_write_disk_full();
    assert!(vfs.write_atomic(&target, new_data).is_err());
    vfs.simulate_crash().expect("crash");
    assert_eq!(vfs.read(&target).expect("read"), initial_data);

    // 2. Crash beim fsync
    vfs.inject_fsync_failure();
    assert!(vfs.write_atomic(&target, new_data).is_err());
    vfs.simulate_crash().expect("crash");
    assert_eq!(vfs.read(&target).expect("read"), initial_data);

    // 3. Crash beim Rename
    vfs.inject_rename_failure();
    assert!(vfs.write_atomic(&target, new_data).is_err());
    vfs.simulate_crash().expect("crash");
    assert_eq!(vfs.read(&target).expect("read"), initial_data);

    // 4. Erfolgreiches Überschreiben
    vfs.write_atomic(&target, new_data).expect("write new");
    vfs.simulate_crash().expect("crash");
    assert_eq!(vfs.read(&target).expect("read"), new_data);
}

#[test]
fn test_inv_fsync_1_handle_poisoning() {
    use contextra_durable_fs::fault_injector::FaultInjector;
    use contextra_durable_fs::{DurableHandle, FlushKind};
    use std::fs::File;
    use std::io::{self, Write};
    use std::path::Path;

    struct FailFsync;
    impl FaultInjector for FailFsync {
        fn on_fsync(&self, _path: Option<&Path>) -> io::Result<()> {
            Err(io::Error::other("simulated fsync failure"))
        }
    }

    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("handle_test.bin");
    let file = File::create(&file_path).expect("create file");

    let mut handle = DurableHandle::new_with_injector(file, FailFsync);
    assert!(!handle.is_poisoned());

    // Flush schlägt fehl -> Handle muss sofort vergiftet sein
    assert!(handle.flush(FlushKind::DataAndMetadata).is_err());
    assert!(handle.is_poisoned());

    // Folgeaufruf von write MUSS sofort abgewiesen werden
    assert!(handle.write(b"data").is_err());
    assert!(handle.flush(FlushKind::DataAndMetadata).is_err());
    assert!(handle.get_mut().is_none());
}
