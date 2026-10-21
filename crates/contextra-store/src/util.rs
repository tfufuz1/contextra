// FILE-CONTEXT: Utility functions for contextra-store (fsync helpers, etc.) (TS: 2026-08-29T17:17:31Z) (SESSION: 8f882f1f)
//! Utility functions for storage engine operations.

use contextra_types::{ContextraError, Result};
use std::path::Path;

use std::sync::atomic::{AtomicU64, Ordering};

pub use contextra_durable_fs::{LockError, LockPolicy};

pub static WAL_CLEANUP_FAILURES_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Increments the global counter tracking failed WAL cleanup attempts.
pub fn increment_wal_cleanup_failures() {
    WAL_CLEANUP_FAILURES_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Returns the current count of failed WAL cleanup attempts.
pub fn wal_cleanup_failures_total() -> u64 {
    WAL_CLEANUP_FAILURES_TOTAL.load(Ordering::Relaxed)
}

#[cfg(all(not(loom), any(test, feature = "test-utils")))]
pub use contextra_durable_fs::test_set_dirlock_sync_hook;

#[cfg(not(loom))]
impl From<crate::lsm::config::DurabilityMode> for contextra_durable_fs::LockPolicy {
    fn from(mode: crate::lsm::config::DurabilityMode) -> Self {
        match mode {
            crate::lsm::config::DurabilityMode::Full
            | crate::lsm::config::DurabilityMode::WalNoHmac => {
                contextra_durable_fs::LockPolicy::FailClosed
            }
            crate::lsm::config::DurabilityMode::MemoryOnly => {
                contextra_durable_fs::LockPolicy::UnsafeNoLock
            }
        }
    }
}

#[cfg(all(not(loom), any(test, feature = "test-utils")))]
#[doc(hidden)]
pub fn test_decide_on_lock_error_wrapper(
    durability_mode: crate::lsm::config::DurabilityMode,
    lock_path: &Path,
    err: &std::io::Error,
) -> Result<Option<std::fs::File>> {
    let policy = LockPolicy::from(durability_mode);
    if policy == LockPolicy::FailClosed {
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

/// Handle for exclusive database directory locking.
#[cfg(not(loom))]
#[derive(Debug)]
pub struct DirLock(
    #[allow(dead_code, reason = "Held for lifetime of database instance to maintain flock")]
    pub contextra_durable_fs::DirLock,
);

#[cfg(not(loom))]
impl DirLock {
    /// Acquires an exclusive lock on `dir/LOCK`.
    pub fn acquire<P: Into<LockPolicy>>(dir: &Path, policy: P) -> Result<Self> {
        let lock = contextra_durable_fs::DirLock::acquire(dir, policy.into())
            .map_err(|e| match e {
                LockError::Conflict => {
                    ContextraError::Storage("Datenverzeichnis bereits in Benutzung".into())
                }
                LockError::Unsupported(msg) => ContextraError::Storage(format!(
                    "Dateisperre auf diesem Dateisystem nicht unterstützt ({msg})"
                )),
                LockError::Io(err) => ContextraError::Storage(err.to_string()),
            })?;
        Ok(Self(lock))
    }
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
    fn test_decide_on_lock_policy_mapping() {
        use crate::lsm::config::DurabilityMode;
        assert_eq!(LockPolicy::from(DurabilityMode::Full), LockPolicy::FailClosed);
        assert_eq!(LockPolicy::from(DurabilityMode::WalNoHmac), LockPolicy::FailClosed);
        assert_eq!(LockPolicy::from(DurabilityMode::MemoryOnly), LockPolicy::UnsafeNoLock);
    }
}

/// Performs fsync on the parent directory of `path`.
///
/// Directory fsync is required on POSIX filesystems to guarantee that newly created files or
/// directory entries are durably persisted to disk.
///
/// # Errors
/// Returns `ContextraError::Storage` if opening or syncing the parent directory fails.
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
