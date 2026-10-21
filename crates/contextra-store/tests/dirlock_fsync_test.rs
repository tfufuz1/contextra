// FILE-CONTEXT: Test that DirLock::acquire performs parent directory fsync and handles fsync errors.

#![forbid(unsafe_code)]

use contextra_store::util::{test_set_dirlock_sync_hook, DirLock};
use contextra_store::lsm::config::DurabilityMode;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::TempDir;

static DIR_SYNC_COUNT: AtomicUsize = AtomicUsize::new(0);

fn mock_dir_sync_hook_ok(_path: &Path) -> std::io::Result<()> {
    DIR_SYNC_COUNT.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

fn mock_dir_sync_hook_err(_path: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "simulated dirlock dir fsync failure",
    ))
}

#[test]
fn test_dirlock_acquire_invokes_dir_fsync() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path();

    DIR_SYNC_COUNT.store(0, Ordering::SeqCst);
    test_set_dirlock_sync_hook(Some(mock_dir_sync_hook_ok));

    let lock_res = DirLock::acquire(db_path, DurabilityMode::Full);
    test_set_dirlock_sync_hook(None);

    assert!(lock_res.is_ok(), "DirLock::acquire should succeed");
    assert!(
        DIR_SYNC_COUNT.load(Ordering::SeqCst) > 0,
        "DirLock::acquire MUST invoke parent directory fsync after creating LOCK file"
    );
}

#[test]
fn test_dirlock_acquire_propagates_dir_fsync_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path();

    test_set_dirlock_sync_hook(Some(mock_dir_sync_hook_err));

    let lock_res = DirLock::acquire(db_path, DurabilityMode::Full);
    test_set_dirlock_sync_hook(None);

    assert!(
        lock_res.is_err(),
        "DirLock::acquire MUST return Err when directory fsync fails"
    );
    assert!(
        lock_res.unwrap_err().to_string().contains("simulated dirlock dir fsync failure"),
        "Error message must convey directory fsync failure"
    );
}
