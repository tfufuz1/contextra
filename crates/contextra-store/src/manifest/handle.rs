//! Manifest-Handle, Öffnen, Vergiftungsschutz und Lifecycle (Ring 1)
//!
//! Verwaltet die geöffnete Manifest-Datei mit atomarem Schreibpuffer, fsync-Semantik
//! und Vergiftungszustand bei fsync-Fehlern (INV-FSYNC-1).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::AsyncWriteExt;

use super::core::{CURRENT_MANIFEST_VERSION, MANIFEST_HEADER_MAGIC};
use crate::error::{StoreError, StoreResult};
use crate::util::DirLock;

/// Append-only Manifest file handle for recording active SSTable set transitions.
pub struct Manifest {
    pub(super) path: PathBuf,
    pub(super) file: tokio::sync::Mutex<tokio::fs::File>,
    pub(super) _dir_lock: Option<DirLock>,
    pub(super) is_poisoned: AtomicBool,
}

impl std::fmt::Debug for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manifest")
            .field("path", &self.path)
            .finish()
    }
}

impl Manifest {
    /// Opens or creates an append-only Manifest file at `path`.
    pub async fn open(path: impl AsRef<Path>) -> StoreResult<Self> {
        Self::open_with_dir_lock(path, None).await
    }

    /// Opens or creates an append-only Manifest file at `path`, attaching `dir_lock`.
    pub(crate) async fn open_with_dir_lock(
        path: impl AsRef<Path>,
        dir_lock: Option<DirLock>,
    ) -> StoreResult<Self> {
        let path = path.as_ref().to_path_buf();
        let (mut file, is_new) = match tokio::fs::OpenOptions::new()
            .create_new(true)
            .append(true)
            .read(true)
            .open(&path)
            .await
        {
            Ok(file) => (file, true),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let file = tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .read(true)
                    .open(&path)
                    .await
                    .map_err(|e| StoreError::Storage(format!("Failed to open MANIFEST: {}", e)))?;
                (file, false)
            }
            Err(e) => {
                return Err(StoreError::Storage(format!(
                    "Failed to create MANIFEST: {}",
                    e
                )));
            }
        };

        if is_new {
            let mut header = Vec::with_capacity(5);
            header.extend_from_slice(MANIFEST_HEADER_MAGIC);
            header.push(CURRENT_MANIFEST_VERSION);
            file.write_all(&header).await.map_err(|e| {
                StoreError::Storage(format!("Failed to write MANIFEST header: {}", e))
            })?;
            file.sync_all().await.map_err(|e| {
                StoreError::Storage(format!(
                    "MANIFEST file fsync failed for {}: {}",
                    path.display(),
                    e
                ))
            })?;
            crate::util::fsync_parent_dir(&path)
                .await
                .map_err(|e| StoreError::Storage(e.to_string()))?;
        }

        Ok(Self {
            path,
            file: tokio::sync::Mutex::new(file),
            _dir_lock: dir_lock,
            is_poisoned: AtomicBool::new(false),
        })
    }

    /// Checks if the manifest handle has been poisoned by a preceding fsync failure (INV-FSYNC-1).
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.is_poisoned.load(Ordering::SeqCst)
    }

    /// Explicitly poisons the manifest handle (INV-FSYNC-1).
    #[inline]
    pub fn poison(&self) {
        self.is_poisoned.store(true, Ordering::SeqCst);
    }

    /// Returns the filesystem path to the manifest file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}
