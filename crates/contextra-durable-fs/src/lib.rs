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

/// Receipt returned upon successfully scrubbing and durably removing a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrubReceipt {
    /// Total number of bytes overwritten with zeros prior to removal.
    pub bytes_scrubbed: u64,
}

/// Overwrites file contents with zeros and durably removes the file.
///
/// Overwrites the file at `path` completely with zero bytes in 64-KiB chunks,
/// flushes data to disk via `sync_all()`, and removes the file via [`durable_remove`].
///
/// # Security & Physical Storage Limitations (Non-Guarantees)
///
/// **Important Notice:** This function reduces residual risk by overwriting visible disk blocks,
/// but **does NOT guarantee** complete physical data destruction or unrecoverability on modern hardware and storage systems.
/// Specifically, physical media remnants may persist due to:
/// - **SSD / Flash Wear-Leveling and Garbage Collection:** Flash controllers rewrite blocks to new physical locations.
/// - **Copy-on-Write (CoW) Filesystems:** (e.g., ZFS, Btrfs, APFS) Overwrites allocate new blocks rather than in-place mutation.
/// - **Journaling Filesystems & Metadata Logs:** Temporary or journaled copies may exist elsewhere on disk.
/// - **Storage Snapshots & Backups:** Historical block snapshots or volume mirrors remain unaffected.
///
/// Do not rely on this function as an absolute physical data destruction mechanism.
///
/// # Behavior & Symlinks
///
/// - Symlinks are **never** followed. If `path` is a symlink or is not a regular file,
///   an error of kind [`std::io::ErrorKind::InvalidInput`] is returned and no files are modified.
/// - If `path` does not exist, [`std::io::ErrorKind::NotFound`] is returned directly.
/// - Exact original byte length is determined prior to overwriting. Exactly `len` bytes are written
///   with no pre-truncation or file extension.
/// - An empty file (0 bytes) results in 0 bytes scrubbed and is durably removed.
///
/// # Concurrency & Async Callers
///
/// This operation is fully synchronous and performs blocking I/O (file overwriting, `sync_all`,
/// and parent directory fsync). Async callers (such as `tokio` runtimes) must wrap calls to
/// this function in blocking execution contexts (e.g., `spawn_blocking`).
///
/// # Errors
///
/// Returns an [`std::io::Error`] if metadata inspection, file opening, writing, flushing,
/// or directory synchronization fails.
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
        let mut file = OpenOptions::new().write(true).open(path)?;

        let zeros = [0u8; 64 * 1024];
        let mut remaining = len;

        while remaining > 0 {
            let chunk_size = usize::try_from(remaining).unwrap_or(zeros.len()).min(zeros.len());
            file.write_all(&zeros[..chunk_size])?;
            remaining -= chunk_size as u64;
        }

        file.sync_all()?;
    }

    durable_remove(path)?;

    Ok(ScrubReceipt { bytes_scrubbed: len })
}
