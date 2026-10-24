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
                if durability_mode == DurabilityMode::Full {
                    Err(ContextraError::Storage(format!(
                        "Dateisperre auf diesem Dateisystem nicht unterstützt: {e}"
                    )))
                } else {
                    tracing::warn!(
                        "Dateisperre auf diesem Dateisystem nicht unterstützt ({}); fahre ohne Sperre fort: {e}",
                        lock_path.display()
                    );
                    Ok(Self { _file: None })
                }
            }
        }
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
