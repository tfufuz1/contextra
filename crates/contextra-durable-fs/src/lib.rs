#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

//! Durable filesystem operations and synchronous directory sync utilities for Contextra.
//!
//! Provides cross-platform durable file persistence contracts:
//! - [`sync_dir`]: Synchronizes directory metadata entries to stable storage.
//! - [`atomic_replace`]: Atomically replaces a target file with data and directory sync.
//! - [`durable_remove`]: Removes a target file and synchronizes the parent directory.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Synchronizes a directory and its metadata entries to stable storage.
///
/// Opens the directory and calls `sync_all`. All errors are propagated.
///
/// # Platform Behavior
/// - **Unix**: Opens directory via `File::open` and invokes `sync_all()`.
/// - **Windows**: Directory fsync is not supported by the underlying OS kernel.
///   Returns `Ok(())` with documented rationale.
pub fn sync_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(not(windows))]
    {
        let file = File::open(dir)?;
        file.sync_all()?;
        Ok(())
    }
    #[cfg(windows)]
    {
        // Windows: Verzeichnis-fsync nicht verfügbar, siehe Dokumentation
        // Feature-Flag-Hinweis: Dir-fsync wird unter Windows vom OS-Kernel nicht unterstützt.
        if !dir.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "directory not found",
            ));
        }
        Ok(())
    }
}

/// Helper struct to ensure temporary file cleanup on failure.
struct TmpFileGuard<'a>(&'a Path);

impl<'a> Drop for TmpFileGuard<'a> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0);
    }
}

/// Atomically replaces the file at `path` with `bytes`.
///
/// Sequence of operations:
/// 1. Determines parent directory.
/// 2. Creates a temporary file in the same directory with a unique name.
/// 3. Writes `bytes` via `write_all`.
/// 4. Flushes data to disk via `sync_all`.
/// 5. Renames the temporary file over `path`.
/// 6. Synchronizes parent directory via [`sync_dir`].
///
/// Every step propagates errors via `?`.
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
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let tmp_filename = format!(".{}.tmp.{}.{}.{}", file_stem, pid, nanos, count);
    let tmp_path = parent.join(tmp_filename);

    let mut tmp_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)?;

    let guard = TmpFileGuard(&tmp_path);

    tmp_file.write_all(bytes)?;
    tmp_file.sync_all()?;
    drop(tmp_file);

    std::fs::rename(&tmp_path, path)?;
    std::mem::forget(guard);

    sync_dir(parent)?;
    Ok(())
}

/// Durably removes a file at `path` and synchronizes the parent directory.
///
/// Sequence of operations:
/// 1. Removes the file via `remove_file`.
/// 2. Synchronizes parent directory via [`sync_dir`].
///
/// If the file does not exist, `std::io::ErrorKind::NotFound` is returned as an error and not ignored.
pub fn durable_remove(path: &Path) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target path has no parent directory",
        )
    })?;

    std::fs::remove_file(path)?;
    sync_dir(parent)?;
    Ok(())
}
