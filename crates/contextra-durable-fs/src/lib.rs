#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

//! Durable filesystem operations and synchronous directory sync utilities for Contextra.
//!
//! Provides cross-platform durable file persistence contracts:
//! - [`sync_dir`]: Synchronizes directory metadata entries to stable storage.
//! - [`atomic_replace`]: Atomically replaces a target file with data and directory sync.
//! - [`durable_remove`]: Removes a target file and synchronizes the parent directory.
//! - [`scrub_and_remove`]: Overwrites file contents with zeros before durable removal.
//! - [`scrub_if_safe`]: Safely scrubs file contents with safety checks (CoW, hardlinks, reader active).
//! - [`DurableHandle`]: File handle wrapper enforcing fsync poisoning semantics (INV-FSYNC-1).
//! - [`DirLock`]: Directory lock guard enforcing filesystem classification rules (§7.3.1).

/// Pending delete journal tracking files scheduled for removal across crashes.
pub mod journal;

/// Scrubbing utilities and safety check policies.
pub mod scrub;

pub use journal::PendingDeleteJournal;
pub use scrub::{scrub_if_safe, ScrubOutcome, ScrubPolicy, ScrubRefusal};

use contextra_sys::{flush_dir, flush_file, fs_class, rename_durable, FlushKind, FsClass};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Marker type representing a handle poisoned by an fsync error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoisonedHandle;

/// A wrapper around [`File`] that poisons the handle upon an fsync failure (INV-FSYNC-1).
///
/// Ensures no further flush attempts occur on a corrupted or errored handle.
#[derive(Debug)]
pub struct DurableHandle {
    file: Option<File>,
    is_poisoned: bool,
}

impl DurableHandle {
    /// Wraps an open [`File`] into a new [`DurableHandle`].
    pub fn new(file: File) -> Self {
        Self {
            file: Some(file),
            is_poisoned: false,
        }
    }

    /// Returns `true` if this handle has been poisoned due to a flush failure.
    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.is_poisoned
    }

    /// Flushes file contents using platform durability primitives.
    ///
    /// If the operation fails, the handle becomes poisoned and subsequent calls fail immediately.
    pub fn flush(&mut self, kind: FlushKind) -> std::io::Result<()> {
        if self.is_poisoned {
            return Err(std::io::Error::other(
                "handle is poisoned due to a previous fsync failure",
            ));
        }

        let Some(file) = self.file.as_ref() else {
            return Err(std::io::Error::other("file handle has been closed"));
        };

        if let Err(err) = flush_file(file, kind) {
            self.is_poisoned = true;
            Err(err)
        } else {
            Ok(())
        }
    }

    /// Returns a reference to the underlying [`File`], or `None` if poisoned.
    #[must_use]
    pub fn get_ref(&self) -> Option<&File> {
        if self.is_poisoned {
            None
        } else {
            self.file.as_ref()
        }
    }

    /// Returns a mutable reference to the underlying [`File`], or `None` if poisoned.
    pub fn get_mut(&mut self) -> Option<&mut File> {
        if self.is_poisoned {
            None
        } else {
            self.file.as_mut()
        }
    }

    /// Explicitly poisons this handle.
    pub fn poison(&mut self) {
        self.is_poisoned = true;
    }
}

impl Write for DurableHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.is_poisoned {
            return Err(std::io::Error::other(
                "handle is poisoned due to a previous fsync failure",
            ));
        }

        let Some(file) = self.file.as_mut() else {
            return Err(std::io::Error::other("file handle has been closed"));
        };

        file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        DurableHandle::flush(self, FlushKind::DataAndMetadata)
    }
}

/// Locking mode for directory lock acquisition (§7.3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LockMode {
    /// Standard directory lock requiring local supported filesystem guarantees.
    #[default]
    Normal,
    /// Memory-only directory lock mode bypassing filesystem persistence checks.
    MemoryOnly,
}

/// Directory lock guard holding lock file ownership and enforcing filesystem safety.
#[derive(Debug)]
pub struct DirLock {
    lock_path: PathBuf,
    _file: Option<File>,
    mode: LockMode,
}

impl DirLock {
    /// Acquires a lock on `dir` according to `mode`.
    ///
    /// Rejects persistent locks on unsupported filesystems (`Network`, `Fuse`, `CloudSync`, `Unknown`)
    /// unless `mode == LockMode::MemoryOnly`. Error messages omit path payloads (INV-LOG-1).
    pub fn acquire(dir: &Path, mode: LockMode) -> std::io::Result<Self> {
        let class = fs_class(dir)?;

        if mode == LockMode::Normal && !class.is_supported_local() {
            let class_str = match class {
                FsClass::Network => "network",
                FsClass::Fuse => "fuse",
                FsClass::CloudSync => "cloud_sync",
                _ => "unsupported_filesystem",
            };
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!(
                    "directory locking rejected for filesystem class: {}",
                    class_str
                ),
            ));
        }

        let lock_path = dir.join(".dirlock");
        let file = if mode == LockMode::Normal {
            let mut options = OpenOptions::new();
            options.write(true).create(true);
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                // FILE_SHARE_READ (1) | FILE_SHARE_WRITE (2) | FILE_SHARE_DELETE (4)
                options.share_access(7);
            }
            Some(options.open(&lock_path)?)
        } else {
            None
        };

        Ok(Self {
            lock_path,
            _file: file,
            mode,
        })
    }

    /// Returns the [`LockMode`] of this lock.
    #[must_use]
    pub fn mode(&self) -> LockMode {
        self.mode
    }
}

impl Drop for DirLock {
    fn drop(&mut self) {
        if self.mode == LockMode::Normal {
            let _file = self._file.take();
            drop(_file);
            let _ = std::fs::remove_file(&self.lock_path);
        }
    }
}

/// Synchronizes a directory and its metadata entries to stable storage.
pub fn sync_dir(dir: &Path) -> std::io::Result<()> {
    flush_dir(dir)
}

struct TmpFileGuard<'a>(&'a Path);

impl<'a> Drop for TmpFileGuard<'a> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0);
    }
}

/// Atomically replaces the file at `path` with `bytes` and flushes parent directory.
pub fn atomic_replace(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target path has no parent directory",
        )
    })?;

    let file_stem = path.file_name().and_then(|s| s.to_str()).unwrap_or("tmp");

    let count = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();

    let tmp_filename = format!(".{}.tmp.{}.{}", file_stem, pid, count);
    let tmp_path = parent.join(tmp_filename);

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_access(7);
    }

    let mut tmp_file = options.open(&tmp_path)?;
    let guard = TmpFileGuard(&tmp_path);

    tmp_file.write_all(bytes)?;
    flush_file(&tmp_file, FlushKind::DataAndMetadata)?;
    drop(tmp_file);

    rename_durable(&tmp_path, path)?;
    std::mem::forget(guard);

    Ok(())
}

/// Durably removes a file at `path` and synchronizes the parent directory.
pub fn durable_remove(path: &Path) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target path has no parent directory",
        )
    })?;

    std::fs::remove_file(path)?;
    flush_dir(parent)?;
    Ok(())
}

/// Receipt returned upon successfully scrubbing and durably removing a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrubReceipt {
    /// Total number of bytes overwritten with zeros prior to removal.
    pub bytes_scrubbed: u64,
}

/// Overwrites file contents with zeros and durably removes the file.
pub fn scrub_and_remove(path: &Path) -> std::io::Result<ScrubReceipt> {
    let metadata = path.symlink_metadata()?;
    let file_type = metadata.file_type();

    if file_type.is_symlink() || !file_type.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path is a symlink or not a regular file",
        ));
    }

    let len = metadata.len();
    if len > 0 {
        let mut options = OpenOptions::new();
        options.write(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_access(7);
        }

        let mut file = options.open(path)?;

        let zeros = [0u8; 64 * 1024];
        let mut remaining = len;

        while remaining > 0 {
            let chunk_size = usize::try_from(remaining)
                .unwrap_or(zeros.len())
                .min(zeros.len());
            file.write_all(&zeros[..chunk_size])?;
            remaining -= chunk_size as u64;
        }

        flush_file(&file, FlushKind::DataAndMetadata)?;
    }

    durable_remove(path)?;

    Ok(ScrubReceipt {
        bytes_scrubbed: len,
    })
}
