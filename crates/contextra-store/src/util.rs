// FILE-CONTEXT: Utility functions for contextra-store (fsync helpers, etc.) (TS: 2026-08-29T17:17:31Z) (SESSION: 8f882f1f)
//! Utility functions for storage engine operations.

use contextra_core::{ContextraError, Result};
use std::path::Path;

#[cfg(not(loom))]
use crate::lsm::config::DurabilityMode;
#[cfg(not(loom))]
use std::fs::{File, TryLockError};

/// Handle for exclusive database directory locking.
#[cfg(not(loom))]
#[derive(Debug)]
pub(crate) struct DirLock {
    _file: Option<File>,
}

#[cfg(not(loom))]
impl DirLock {
    /// Acquires an exclusive lock on `dir/LOCK`.
    pub(crate) fn acquire(dir: &Path, durability_mode: DurabilityMode) -> Result<Self> {
        let lock_path = dir.join("LOCK");
        let file = File::create(&lock_path).map_err(|e| {
            ContextraError::Storage(format!(
                "Failed to open/create LOCK file at {}: {e}",
                lock_path.display()
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

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

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
