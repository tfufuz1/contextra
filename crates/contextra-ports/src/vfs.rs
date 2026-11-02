//! Virtual File System (VFS) port trait definition.
//!
//! # SPEC-REF & Rule 5 Compliance
//! In accordance with AGENTS.md Rule 5 ("I/O nur über `Vfs`") and contract V-VFS,
//! all file system I/O operations in storage, engine, and index layers must interact
//! exclusively through this abstract VFS interface.
//!
//! # Crash Consistency & Durability Contract
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
//! - **Cancel-Safety**:
//!   Synchronous VFS operations are inherently non-preemptible by async cancellations once executing within OS boundaries.

use crate::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Options configuring how a file is opened via [`Vfs::open`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VfsOpenOptions {
    /// Opens the file with read access.
    pub read: bool,
    /// Opens the file with write access.
    pub write: bool,
    /// Creates a new file if it does not exist.
    pub create: bool,
    /// Creates a new file, failing if it already exists (`create_new`).
    pub create_new: bool,
    /// Truncates the file length to 0 upon opening.
    pub truncate: bool,
    /// Sets append mode for write operations.
    pub append: bool,
}

impl VfsOpenOptions {
    /// Creates options configured for read-only access.
    pub fn read_only() -> Self {
        Self {
            read: true,
            ..Default::default()
        }
    }

    /// Creates options configured for write-only access, creating the file if missing.
    pub fn write_create() -> Self {
        Self {
            write: true,
            create: true,
            ..Default::default()
        }
    }
}

/// Abstract file handle providing positional I/O without modifying internal file offsets.
pub trait VfsFile: Send + Sync {
    /// Reads data from byte offset `offset` into `buf` without moving the file pointer.
    ///
    /// # Errors
    /// Returns `std::io::Error` on read failure.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;

    /// Writes data from `buf` to byte offset `offset` without moving the file pointer.
    ///
    /// # Errors
    /// Returns `std::io::Error` on write failure.
    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>;

    /// Truncates or extends the underlying file length to `len` bytes.
    ///
    /// # Errors
    /// Returns `std::io::Error` on truncate or extension failure.
    fn set_len(&self, len: u64) -> std::io::Result<()>;

    /// Queries the current physical file length in bytes.
    ///
    /// # Errors
    /// Returns `std::io::Error` if file metadata cannot be queried.
    fn len(&self) -> std::io::Result<u64>;

    /// Returns `true` if the file length is zero.
    ///
    /// # Errors
    /// Returns `std::io::Error` if querying file length fails.
    fn is_empty(&self) -> std::io::Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Flushes data blocks to disk (`fdatasync`).
    ///
    /// # Errors
    /// Returns `std::io::Error` if data synchronization fails.
    fn sync_data(&self) -> std::io::Result<()>;

    /// Flushes data blocks and metadata to disk (`fsync`).
    ///
    /// # Errors
    /// Returns `std::io::Error` if file metadata or data synchronization fails.
    fn sync_all(&self) -> std::io::Result<()>;
}

/// Abstract object-safe Virtual File System interface.
///
/// Implements required file operations for LSM storage engine, SSTables, WAL sidecars,
/// and vector index file persistence.
pub trait Vfs: Send + Sync + 'static {
    /// Opens a file according to the specified options.
    ///
    /// # Contract V-VFS
    /// Default implementation delegates to `std::io::ErrorKind::Unsupported` unless overridden.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if opening fails.
    fn open(&self, _path: &Path, _opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        Err(crate::ContextraError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "positional VfsFile open is not supported by default in this Vfs implementation",
        )))
    }

    /// Reads the entire contents of a file at `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
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
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// if removal fails or file does not exist.
    fn remove_file(&self, path: &Path) -> Result<()>;

    /// Atomically renames a file from `from` to `to`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if rename fails.
    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let _ = (from, to);
        Err(crate::ContextraError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "atomic rename is not supported by default in this Vfs implementation",
        )))
    }

    /// Reads all entry paths within directory `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if reading directory entries fails.
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = self.list_dir(path)?;
        Ok(entries.into_iter().map(|name| path.join(name)).collect())
    }

    /// Returns `true` if a file or directory exists at `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if path state cannot be inspected.
    fn exists(&self, path: &Path) -> Result<bool>;

    /// Returns the length in bytes of the file at `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// if file metadata cannot be retrieved.
    fn len(&self, path: &Path) -> Result<u64>;

    /// Lists entry names (file and directory names, excluding `.` and `..`) inside directory `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// if path is not a directory or cannot be read.
    fn list_dir(&self, path: &Path) -> Result<Vec<String>>;

    /// Flushes directory metadata to persistent storage for durable file additions and renames.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if directory syncing fails.
    fn sync_dir(&self, path: &Path) -> Result<()>;
}

impl<T: Vfs + ?Sized> Vfs for Arc<T> {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        (**self).open(path, opts)
    }

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

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        (**self).rename(from, to)
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        (**self).read_dir(path)
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
