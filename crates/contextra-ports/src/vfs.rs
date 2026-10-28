//! Virtual File System (VFS) port trait definition.
//!
//! # SPEC-REF & Rule 5 Compliance
//! In accordance with AGENTS.md Rule 5 ("I/O nur über `Vfs`"), all file system I/O operations
//! in storage and engine layers must interact exclusively through this abstract VFS interface.
//!
//! # Crash Consistency Contract
//! Concrete implementations of [`Vfs`] must satisfy strict crash-consistency guarantees:
//! - **Atomic Write (`write_atomic`)**:
//!   1. Data must first be written completely to a temporary file in the target directory (or sibling location on the same filesystem).
//!   2. The temporary file must be explicitly flushed/synced to disk (`fsync` / `FlushFileBuffers`).
//!   3. The temporary file must be atomically renamed to the target `path`.
//!   4. The parent directory must be synced (`sync_dir`) to ensure directory entry durability on filesystems like ext4/xfs.
//!   5. If any failure occurs prior to final rename, the original file at `path` must remain untouched and uncorrupted.
//! - **Directory Sync (`sync_dir`)**:
//!   Flushes pending directory metadata changes (file additions/deletions/renames) to persistent storage.
//! - **Fail-Closed Propagation**:
//!   I/O errors must be converted into [`ContextraError::Io`](crate::error::ContextraError::Io) or appropriate domain errors
//!   and propagated immediately. Swallowing I/O errors or ignoring sync returns is strictly prohibited.

use crate::Result;
use std::path::Path;
use std::sync::Arc;

/// Abstract object-safe, synchronous Virtual File System interface.
///
/// Implements minimal required operations for LSM storage engine, SSTables, WAL sidecars,
/// and vector index file persistence.
pub trait Vfs: Send + Sync + 'static {
    /// Reads the entire contents of a file at `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::FileNotFound`](crate::error::ContextraError::FileNotFound)
    /// if the file does not exist or cannot be read.
    fn read(&self, path: &Path) -> Result<Vec<u8>>;

    /// Atomically writes `data` to `path` with crash-consistency guarantees.
    ///
    /// # Crash-Consistency Contract
    /// Writes to a temporary file, calls `fsync` on the file handle, renames the temporary
    /// file over `path`, and syncs the parent directory (`sync_dir`).
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if write, sync, or rename fails.
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>;

    /// Creates a directory and all parent directories along `path` if missing.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if creation fails.
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Removes a file at `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::FileNotFound`](crate::error::ContextraError::FileNotFound)
    /// if removal fails or file does not exist.
    fn remove_file(&self, path: &Path) -> Result<()>;

    /// Returns `true` if a file or directory exists at `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if path state cannot be inspected.
    fn exists(&self, path: &Path) -> Result<bool>;

    /// Returns the length in bytes of the file at `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::FileNotFound`](crate::error::ContextraError::FileNotFound)
    /// if file metadata cannot be retrieved.
    fn len(&self, path: &Path) -> Result<u64>;

    /// Lists entry names (file and directory names, excluding `.` and `..`) inside directory `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::FileNotFound`](crate::error::ContextraError::FileNotFound)
    /// if path is not a directory or cannot be read.
    fn list_dir(&self, path: &Path) -> Result<Vec<String>>;

    /// Flushes directory metadata to persistent storage for durable file additions and renames.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if directory syncing fails.
    fn sync_dir(&self, path: &Path) -> Result<()>;
}

impl<T: Vfs + ?Sized> Vfs for Arc<T> {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        (**self).read(path)
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        (**self).write_atomic(path, data)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        (**self).create_dir_all(path)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        (**self).remove_file(path)
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        (**self).exists(path)
    }

    fn len(&self, path: &Path) -> Result<u64> {
        (**self).len(path)
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        (**self).list_dir(path)
    }

    fn sync_dir(&self, path: &Path) -> Result<()> {
        (**self).sync_dir(path)
    }
}
