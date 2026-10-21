// FILE-CONTEXT: Utility functions for contextra-store (fsync helpers, etc.) (TS: 2026-08-29T17:17:31Z) (SESSION: 8f882f1f)
//! Utility functions for storage engine operations.

use contextra_types::{ContextraError, Result};
use std::path::Path;

#[cfg(not(loom))]
use crate::lsm::config::DurabilityMode;
#[cfg(not(loom))]
use std::fs::{File, TryLockError};

#[cfg(all(not(loom), any(test, feature = "test-utils")))]
pub type DirLockSyncHookFn = fn(&Path) -> std::io::Result<()>;

#[cfg(all(not(loom), any(test, feature = "test-utils")))]
std::thread_local! {
    static DIRLOCK_SYNC_HOOK: std::cell::RefCell<Option<DirLockSyncHookFn>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(all(not(loom), any(test, feature = "test-utils")))]
pub fn test_set_dirlock_sync_hook(hook: Option<DirLockSyncHookFn>) {
    DIRLOCK_SYNC_HOOK.with(|h| {
        *h.borrow_mut() = hook;
    });
}

#[cfg(not(loom))]
pub(crate) fn sync_dir_util(dir: &Path) -> std::io::Result<()> {
    #[cfg(any(test, feature = "test-utils"))]
    {
        let mut hook_res = None;
        DIRLOCK_SYNC_HOOK.with(|h| {
            if let Some(hook) = *h.borrow() {
                hook_res = Some(hook(dir));
            }
        });
        if let Some(res) = hook_res {
            return res;
        }
    }
    contextra_durable_fs::sync_dir(dir)
}

/// Handle for exclusive database directory locking.
#[cfg(not(loom))]
#[derive(Debug)]
pub struct DirLock {
    _file: Option<File>,
}

#[cfg(not(loom))]
impl DirLock {
    /// Acquires an exclusive lock on `dir/LOCK`.
    pub fn acquire(dir: &Path, durability_mode: DurabilityMode) -> Result<Self> {
        let lock_path = dir.join("LOCK");
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|e| {
                ContextraError::Storage(format!(
                    "Failed to open/create LOCK file at {}: {e}",
                    lock_path.display()
                ))
            })?;

        sync_dir_util(dir).map_err(|e| {
            ContextraError::Storage(format!(
                "Failed to fsync directory for LOCK file at {}: {e}",
                dir.display()
            ))
        })?;

        match file.try_lock() {
            Ok(()) => Ok(Self { _file: Some(file) }),
            Err(TryLockError::WouldBlock) => Err(ContextraError::Storage(
                "Datenverzeichnis bereits in Benutzung".into(),
            )),
            Err(TryLockError::Error(e)) => {
                decide_on_lock_error(durability_mode, &lock_path, &e).map(|_file| Self { _file })
            }
        }
    }
}

#[cfg(not(loom))]
fn decide_on_lock_error(
    durability_mode: DurabilityMode,
    lock_path: &Path,
    err: &std::io::Error,
) -> Result<Option<File>> {
    if durability_mode == DurabilityMode::Full || durability_mode == DurabilityMode::WalNoHmac {
        Err(ContextraError::Storage(format!(
            "Dateisperre auf diesem Dateisystem nicht unterstützt ({}): {err}",
            lock_path.display()
        )))
    } else {
        tracing::warn!(
            "Dateisperre auf diesem Dateisystem nicht unterstützt ({}); fahre ohne Sperre fort: {err}",
            lock_path.display()
        );
        Ok(None)
    }
}

#[cfg(all(not(loom), any(test, feature = "test-utils")))]
#[doc(hidden)]
#[allow(dead_code)]
pub fn test_decide_on_lock_error_wrapper(
    durability_mode: DurabilityMode,
    lock_path: &Path,
    err: &std::io::Error,
) -> Result<Option<File>> {
    decide_on_lock_error(durability_mode, lock_path, err)
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};
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

    #[test]
    fn test_decide_on_lock_error_semantics() {
        let dummy_path = Path::new("/tmp/test_LOCK");
        let mock_err = Error::new(ErrorKind::Unsupported, "Operation not supported");

        // DurabilityMode::Full -> Err
        let full_res = decide_on_lock_error(DurabilityMode::Full, dummy_path, &mock_err);
        assert!(full_res.is_err());
        assert!(full_res
            .unwrap_err()
            .to_string()
            .contains("Dateisperre auf diesem Dateisystem nicht unterstützt"));

        // DurabilityMode::WalNoHmac -> Err (Fail-closed fix per decision D1!)
        let wal_res = decide_on_lock_error(DurabilityMode::WalNoHmac, dummy_path, &mock_err);
        assert!(
            wal_res.is_err(),
            "WalNoHmac must fail closed on lock failure"
        );
        assert!(wal_res
            .unwrap_err()
            .to_string()
            .contains("Dateisperre auf diesem Dateisystem nicht unterstützt"));

        // DurabilityMode::MemoryOnly -> Ok(None)
        let mem_res = decide_on_lock_error(DurabilityMode::MemoryOnly, dummy_path, &mock_err);
        assert!(mem_res.is_ok());
        assert!(mem_res.unwrap().is_none());
    }
}

/// Performs fsync on the parent directory of `path`.
///
/// Directory fsync is required on POSIX filesystems to guarantee that newly created files or
/// directory entries are durably persisted to disk.
///
/// # Errors
/// Returns `ContextraError::Storage` if opening or syncing the parent directory fails.
pub(crate) fn fsync_parent_dir_sync(path: &Path) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let dir_path = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };

    let dir = std::fs::File::open(dir_path).map_err(|e| {
        ContextraError::Storage(format!(
            "Directory open failed for fsync on {}: {e}",
            dir_path.display()
        ))
    })?;

    dir.sync_all().map_err(|e| {
        ContextraError::Storage(format!(
            "Directory fsync failed for {}: {e}",
            dir_path.display()
        ))
    })?;

    Ok(())
}

pub(crate) async fn fsync_parent_dir(path: &Path) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let dir_path = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };

    let dir = crate::wal::fs::File::open(dir_path).await.map_err(|e| {
        ContextraError::Storage(format!(
            "Directory open failed for fsync on {}: {e}",
            dir_path.display()
        ))
    })?;

    dir.sync_all().await.map_err(|e| {
        ContextraError::Storage(format!(
            "Directory fsync failed for {}: {e}",
            dir_path.display()
        ))
    })?;

    Ok(())
}
