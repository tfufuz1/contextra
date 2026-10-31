//! # Hilfswerkzeuge fuer den Store (Ring 1)
//!
//! Stellt universelle Hilfsfunktionen bereit, insbesondere fuer deterministische
//! Hashberechnungen und Byte-Konvertierungen.

// FILE-CONTEXT: Utility functions for contextra-store (fsync helpers, etc.) (TS: 2026-08-29T17:17:31Z) (SESSION: 8f882f1f)
//! Utility functions for storage engine operations.

use contextra_types::{ContextraError, Result};
use std::path::Path;

#[cfg(not(loom))]
use crate::lsm::config::DurabilityMode;
#[cfg(not(loom))]
use std::fs::{File, TryLockError};

pub(crate) use crate::fsync::fsync_parent_dir;

/// Handle for exclusive database directory locking.
#[cfg(not(loom))]
#[derive(Debug)]
// ANCHOR[contextra-store:ROOT-UTIL-001]
/// Handle fuer exklusive Datenverzeichnissperre.
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
