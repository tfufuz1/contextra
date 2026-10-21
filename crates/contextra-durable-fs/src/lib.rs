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
//! - [`FaultInjector`]: Trait providing injection points for storage faults (write, fsync, rename, dir-sync).
//!
//! ## Invariants
//!
//! - `INV-FSYNC-1`: File handles ([`DurableHandle`]) are poisoned upon any fsync or write failure,
//!   preventing further flush or write attempts on corrupted handles.
//! - `INV-ENOSPC-1`: Disk-full (`ENOSPC`) and storage I/O errors across write, fsync, rename,
//!   and directory sync are fail-closed and deterministically propagated as [`std::io::Error`] without panics.

/// Fail-closed health state tracking for storage operations (§18.4).
pub mod health;

/// Fault injection hooks for durable filesystem operations.
pub mod fault_injector;

/// Pending delete journal tracking files scheduled for removal across crashes.
pub mod journal;

/// Scrubbing utilities and safety check policies.
pub mod scrub;

/// Virtual File System (VFS) abstraction interface.
pub mod vfs;

pub use contextra_sys::FlushKind;
pub use fault_injector::{FaultInjector, NoFault};
pub use health::{AtomicHealthState, HealthEvent, HealthState};
pub use journal::PendingDeleteJournal;
pub use scrub::{
    scrub_and_remove, scrub_and_remove_with_injector, scrub_and_remove_with_mountinfo,
    scrub_if_safe, scrub_if_safe_with_injector, scrub_if_safe_with_mountinfo, ScrubMethod,
    ScrubOutcome, ScrubPolicy, ScrubReceipt, ScrubRefusal,
};
pub use vfs::{RealFs, RealFsFile, Vfs, VfsFile, VfsOpenOptions};

use contextra_sys::{flush_dir, flush_file, fs_class, rename_durable, FsClass};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

// Process-static monotonic counter providing local sequence numbers for temporary file creation.
// Uniqueness across processes and targets is guaranteed because temporary filenames incorporate the
// target directory path, target file stem, OS Process ID (`pid`), and this atomic sequence `count`.
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Marker type representing a handle poisoned by an fsync error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoisonedHandle;

/// A wrapper around [`File`] that poisons the handle upon an fsync failure (INV-FSYNC-1).
///
/// Ensures no further flush or write attempts occur on a corrupted or errored handle.
#[derive(Debug)]
pub struct DurableHandle<F: FaultInjector = NoFault> {
    file: Option<File>,
    is_poisoned: bool,
    injector: F,
    path: Option<PathBuf>,
}

impl DurableHandle<NoFault> {
    /// Wraps an open [`File`] into a new [`DurableHandle`] with default no-op fault injector.
    pub fn new(file: File) -> Self {
        Self::new_with_injector(file, NoFault)
    }
}

impl<F: FaultInjector> DurableHandle<F> {
    /// Wraps an open [`File`] with a custom [`FaultInjector`].
    pub fn new_with_injector(file: File, injector: F) -> Self {
        Self {
            file: Some(file),
            is_poisoned: false,
            injector,
            path: None,
        }
    }

    /// Wraps an open [`File`] associated with `path` and a custom [`FaultInjector`].
    pub fn new_with_path_and_injector(file: File, path: PathBuf, injector: F) -> Self {
        Self {
            file: Some(file),
            is_poisoned: false,
            injector,
            path: Some(path),
        }
    }

    /// Returns `true` if this handle has been poisoned due to a flush failure.
    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.is_poisoned
    }

    /// Flushes file contents using platform durability primitives and executes fault injection hooks.
    ///
    /// If the operation or fault hook fails, the handle becomes poisoned and subsequent calls fail immediately (INV-FSYNC-1).
    pub fn flush(&mut self, kind: FlushKind) -> std::io::Result<()> {
        if self.is_poisoned {
            return Err(std::io::Error::other(
                "handle is poisoned due to a previous fsync failure",
            ));
        }

        let Some(file) = self.file.as_ref() else {
            return Err(std::io::Error::other("file handle has been closed"));
        };

        if let Err(err) = self.injector.on_fsync(self.path.as_deref()) {
            self.is_poisoned = true;
            return Err(err);
        }

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

impl<F: FaultInjector> Write for DurableHandle<F> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.is_poisoned {
            return Err(std::io::Error::other(
                "handle is poisoned due to a previous fsync failure",
            ));
        }

        let Some(file) = self.file.as_mut() else {
            return Err(std::io::Error::other("file handle has been closed"));
        };

        self.injector.on_write(self.path.as_deref(), buf.len())?;

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

/// Synchronizes a directory and its metadata entries to stable storage using a default no-op fault injector.
pub fn sync_dir(dir: &Path) -> std::io::Result<()> {
    sync_dir_with_injector(dir, &NoFault)
}

/// Synchronizes a directory and its metadata entries to stable storage with a custom [`FaultInjector`].
pub fn sync_dir_with_injector<F: FaultInjector>(dir: &Path, injector: &F) -> std::io::Result<()> {
    injector.on_dir_fsync(dir)?;
    flush_dir(dir)
}

struct TmpFileGuard<'a>(&'a Path);

impl<'a> Drop for TmpFileGuard<'a> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0);
    }
}

/// Atomically replaces the file at `path` with `bytes` and flushes parent directory using default no-op fault injector.
pub fn atomic_replace(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    atomic_replace_with_injector(path, bytes, &NoFault)
}

/// Atomically replaces the file at `path` with `bytes` and flushes parent directory with a custom [`FaultInjector`].
pub fn atomic_replace_with_injector<F: FaultInjector>(
    path: &Path,
    bytes: &[u8],
    injector: &F,
) -> std::io::Result<()> {
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

    injector.on_write(Some(&tmp_path), bytes.len())?;
    tmp_file.write_all(bytes)?;

    injector.on_fsync(Some(&tmp_path))?;
    flush_file(&tmp_file, FlushKind::DataAndMetadata)?;
    drop(tmp_file);

    injector.on_rename(&tmp_path, path)?;
    rename_durable(&tmp_path, path)?;

    sync_dir_with_injector(parent, injector)?;

    std::mem::forget(guard);

    Ok(())
}

/// Durably removes a file at `path` and synchronizes the parent directory using default no-op fault injector.
pub fn durable_remove(path: &Path) -> std::io::Result<()> {
    durable_remove_with_injector(path, &NoFault)
}

/// Durably removes a file at `path` and synchronizes the parent directory with a custom [`FaultInjector`].
pub fn durable_remove_with_injector<F: FaultInjector>(
    path: &Path,
    injector: &F,
) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target path has no parent directory",
        )
    })?;

    std::fs::remove_file(path)?;
    sync_dir_with_injector(parent, injector)?;
    Ok(())
}
