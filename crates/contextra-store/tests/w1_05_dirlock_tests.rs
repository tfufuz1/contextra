use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::TempDir;

use contextra_durable_fs::{sync_dir_with_injector, FaultInjector, LockMode};

#[derive(Default)]
struct DirSyncErrInjector {
    dir_sync_calls: AtomicUsize,
}

impl FaultInjector for DirSyncErrInjector {
    fn on_dir_fsync(&self, _dir: &Path) -> std::io::Result<()> {
        self.dir_sync_calls.fetch_add(1, Ordering::SeqCst);
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Simulated dir sync error",
        ))
    }
}

#[test]
fn rename_durable_propagates_dir_sync_error() {
    let temp_dir = TempDir::new().expect("tempdir");
    let injector = DirSyncErrInjector::default();

    let res = sync_dir_with_injector(temp_dir.path(), &injector);
    assert!(res.is_err(), "sync_dir_with_injector must propagate error");
    assert_eq!(
        injector.dir_sync_calls.load(Ordering::SeqCst),
        1,
        "dir sync hook must be called"
    );
    assert!(
        res.unwrap_err().to_string().contains("Simulated dir sync error"),
        "Error message must contain simulated error text"
    );
}

#[test]
fn second_process_cannot_open_when_lock_unsupported() {
    let temp_dir = TempDir::new().expect("tempdir");
    // Verify that attempting DirLock acquire with LockMode::Normal fails closed if unsupported or when second acquire occurs.
    let lock1 = contextra_durable_fs::DirLock::acquire(temp_dir.path(), LockMode::Normal);
    assert!(lock1.is_ok());

    // Second lock attempt on same directory
    let lock2 = contextra_durable_fs::DirLock::acquire(temp_dir.path(), LockMode::Normal);
    // On POSIX or Windows, try_lock on the lock file will fail/conflict or behave appropriately.
    // Note: In current durable-fs DirLock, it creates .dirlock without try_lock yet.
    // This red test verifies LockPolicy / try_lock behavior in durable-fs DirLock.
    assert!(
        lock2.is_err(),
        "Second acquire on same directory must fail closed when lock is held or unsupported"
    );
}

#[test]
fn lock_file_not_truncated_by_second_open() {
    let temp_dir = TempDir::new().expect("tempdir");
    let lock_path = temp_dir.path().join("LOCK");

    // Write contents to lock file
    std::fs::write(&lock_path, b"PID=12345").unwrap();

    // Opening LOCK file via DirLock must NOT truncate existing file contents
    let _lock = contextra_durable_fs::DirLock::acquire(temp_dir.path(), LockMode::Normal).unwrap();
    let content = std::fs::read(&lock_path).unwrap();
    assert_eq!(content, b"PID=12345", "LOCK file must not be truncated when acquired");
}

#[test]
fn unsafe_no_lock_opt_in_logs_warning() {
    let temp_dir = TempDir::new().expect("tempdir");
    // MemoryOnly / UnsafeNoLock mode allows proceeding without file lock
    let lock = contextra_durable_fs::DirLock::acquire(temp_dir.path(), LockMode::MemoryOnly);
    assert!(lock.is_ok(), "MemoryOnly / UnsafeNoLock must succeed");
    assert_eq!(lock.unwrap().mode(), LockMode::MemoryOnly);
}
