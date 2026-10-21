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
pub use vfs::{FailpointStep, FailpointVfs, RealFs, RealFsFile, Vfs, VfsFile, VfsOpenOptions};

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

/// Locking policy for directory lock acquisition (§7.3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LockPolicy {
    /// Fail closed on lock acquisition errors or unsupported filesystems (Default).
    #[default]
    FailClosed,
    /// Proceed without lock on lock acquisition errors or unsupported filesystems (Opt-in with warning).
    UnsafeNoLock,
}

impl LockPolicy {
    /// Alias for `FailClosed` for backward compatibility with `LockMode::Normal`.
    #[allow(non_upper_case_globals)]
    pub const Normal: LockPolicy = LockPolicy::FailClosed;
    /// Alias for `UnsafeNoLock` for backward compatibility with `LockMode::MemoryOnly`.
    #[allow(non_upper_case_globals)]
    pub const MemoryOnly: LockPolicy = LockPolicy::UnsafeNoLock;
}

/// Deprecated alias for [`LockPolicy`].
pub type LockMode = LockPolicy;

/// Error type for directory lock acquisition.
#[derive(Debug)]
pub enum LockError {
    /// Data directory is already in use by another process/instance.
    Conflict,
    /// File locking is unsupported on this filesystem.
    Unsupported(String),
    /// Standard I/O error during lock file operation or directory sync.
    Io(std::io::Error),
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockError::Conflict => write!(f, "Datenverzeichnis bereits in Benutzung"),
            LockError::Unsupported(msg) => {
                write!(f, "Dateisperre auf diesem Dateisystem nicht unterstützt: {msg}")
            }
            LockError::Io(err) => write!(f, "I/O error during lock file operation: {err}"),
        }
    }
}

impl std::error::Error for LockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LockError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for LockError {
    fn from(err: std::io::Error) -> Self {
        LockError::Io(err)
    }
}

impl From<LockError> for std::io::Error {
    fn from(err: LockError) -> Self {
        match err {
            LockError::Conflict => std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "Datenverzeichnis bereits in Benutzung",
            ),
            LockError::Unsupported(msg) => std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("Dateisperre auf diesem Dateisystem nicht unterstützt: {msg}"),
            ),
            LockError::Io(e) => e,
        }
    }
}

/// Directory lock guard holding lock file ownership and enforcing filesystem safety.
#[derive(Debug)]
pub struct DirLock {
    lock_path: PathBuf,
    _file: Option<File>,
    policy: LockPolicy,
}

impl DirLock {
    /// Acquires an exclusive lock on `dir/LOCK`.
    pub fn acquire<P: Into<LockPolicy>>(dir: &Path, policy: P) -> Result<Self, LockError> {
        Self::acquire_with_injector(dir, policy.into(), &NoFault)
    }

    /// Acquires an exclusive lock on `dir/LOCK` with a custom [`FaultInjector`].
    pub fn acquire_with_injector<F: FaultInjector>(
        dir: &Path,
        policy: LockPolicy,
        injector: &F,
    ) -> Result<Self, LockError> {
        let class = fs_class(dir)?;

        if policy == LockPolicy::FailClosed
            && matches!(class, FsClass::Network | FsClass::Fuse | FsClass::CloudSync)
        {
            let class_str = match class {
                FsClass::Network => "network",
                FsClass::Fuse => "fuse",
                FsClass::CloudSync => "cloud_sync",
                _ => "unsupported_filesystem",
            };
            return Err(LockError::Unsupported(format!(
                "directory locking rejected for filesystem class: {class_str}"
            )));
        }

        let lock_path = dir.join("LOCK");
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        let open_res = options.open(&lock_path);

        let file = match open_res {
            Ok(f) => f,
            Err(e) => {
                if policy == LockPolicy::UnsafeNoLock {
                    tracing::warn!(
                        "Dateisperre auf diesem Dateisystem nicht unterstützt ({}); fahre ohne Sperre fort: {e}",
                        lock_path.display()
                    );
                    return Ok(Self {
                        lock_path,
                        _file: None,
                        policy,
                    });
                }
                return Err(LockError::Unsupported(format!(
                    "Failed to open/create LOCK file at {}: {e}",
                    lock_path.display()
                )));
            }
        };

        if let Err(e) = sync_dir_with_injector(dir, injector) {
            if policy == LockPolicy::UnsafeNoLock {
                tracing::warn!(
                    "Directory sync for LOCK file failed ({}); proceeding without lock in unsafe_no_lock mode: {e}",
                    dir.display()
                );
                return Ok(Self {
                    lock_path,
                    _file: None,
                    policy,
                });
            }
            return Err(LockError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to fsync directory for LOCK file at {}: {e}", dir.display()),
            )));
        }

        match file.try_lock() {
            Ok(()) => Ok(Self {
                lock_path,
                _file: Some(file),
                policy,
            }),
            Err(std::fs::TryLockError::WouldBlock) => Err(LockError::Conflict),
            Err(std::fs::TryLockError::Error(e)) => {
                if policy == LockPolicy::UnsafeNoLock {
                    tracing::warn!(
                        "Dateisperre auf diesem Dateisystem nicht unterstützt ({}); fahre ohne Sperre fort: {e}",
                        lock_path.display()
                    );
                    Ok(Self {
                        lock_path,
                        _file: None,
                        policy,
                    })
                } else {
                    Err(LockError::Unsupported(format!(
                        "{}: {e}",
                        lock_path.display()
                    )))
                }
            }
        }
    }

    /// Returns the lock file path.
    #[must_use]
    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }

    /// Returns the [`LockPolicy`] of this lock.
    #[must_use]
    pub fn policy(&self) -> LockPolicy {
        self.policy
    }

    /// Returns the [`LockMode`] of this lock.
    #[must_use]
    pub fn mode(&self) -> LockPolicy {
        self.policy
    }
}

impl Drop for DirLock {
    fn drop(&mut self) {
        let _file = self._file.take();
        // INTENTIONAL-DROP: Dropping File handle explicitly releases POSIX flock / Win32 file lock.
        // We deliberately do not remove the LOCK file on disk to avoid TOCTOU races between processes.
        drop(_file);
    }
}

/// Synchronizes a directory and its metadata entries to stable storage using a default no-op fault injector.
pub fn sync_dir(dir: &Path) -> std::io::Result<()> {
    sync_dir_with_injector(dir, &NoFault)
}

#[cfg(any(test, feature = "test-utils"))]
/// Function pointer signature for thread-local directory sync hooks used in tests.
pub type DirLockSyncHookFn = fn(&Path) -> std::io::Result<()>;

#[cfg(any(test, feature = "test-utils"))]
std::thread_local! {
    static DIRLOCK_SYNC_HOOK: std::cell::RefCell<Option<DirLockSyncHookFn>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(any(test, feature = "test-utils"))]
/// Sets or clears a thread-local directory sync hook for testing.
pub fn test_set_dirlock_sync_hook(hook: Option<DirLockSyncHookFn>) {
    DIRLOCK_SYNC_HOOK.with(|h| {
        *h.borrow_mut() = hook;
    });
}

/// Synchronizes a directory and its metadata entries to stable storage with a custom [`FaultInjector`].
pub fn sync_dir_with_injector<F: FaultInjector>(dir: &Path, injector: &F) -> std::io::Result<()> {
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
    injector.on_dir_fsync(dir)?;
    flush_dir(dir)
}

struct TmpFileGuard<'a>(&'a Path);

impl<'a> Drop for TmpFileGuard<'a> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0); // INTENTIONAL-DROP: Best-effort cleanup of temporary file if atomic_replace unwinds or errors before std::mem::forget(guard).
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

/// Atomically writes `bytes` to `path` using `vfs` with temp file creation, fsync, rename, and directory sync.
pub async fn durable_write_atomic(
    vfs: &dyn Vfs,
    path: &Path,
    bytes: &[u8],
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
    let tmp_filename = format!(".{}.atomic_tmp.{}.{}", file_stem, pid, count);
    let tmp_path = parent.join(tmp_filename);

    let opts = VfsOpenOptions {
        write: true,
        create_new: true,
        ..Default::default()
    };

    let file = vfs.open(&tmp_path, &opts)?;

    let write_res = (|| -> std::io::Result<()> {
        let mut written = 0;
        while written < bytes.len() {
            let n = file.write_at(&bytes[written..], written as u64)?;
            if n == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "failed to write all bytes to temp file",
                ));
            }
            written += n;
        }
        file.sync_all()?;
        Ok(())
    })();

    if let Err(err) = write_res {
        let _ = vfs.remove_file(&tmp_path); // INTENTIONAL-DROP: Best-effort cleanup of partial temp file on failure.
        return Err(err);
    }

    if let Err(err) = vfs.rename(&tmp_path, path) {
        let _ = vfs.remove_file(&tmp_path); // INTENTIONAL-DROP: Best-effort cleanup of temp file when rename fails.
        return Err(err);
    }

    if let Err(err) = vfs.sync_dir(parent) {
        return Err(err);
    }

    Ok(())
}

/// Removes a file if it exists, ignoring `NotFound` errors and propagating all other I/O errors.
pub async fn remove_if_exists(vfs: &dyn Vfs, path: &Path) -> std::io::Result<()> {
    match vfs.remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}
