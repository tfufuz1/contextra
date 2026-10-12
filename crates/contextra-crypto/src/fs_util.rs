#![forbid(unsafe_code)]

//! Private filesystem utilities for durable operations in contextra-crypto.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Synchronizes a directory and its metadata entries to stable storage.
pub fn sync_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(not(windows))]
    {
        let file = File::open(dir)?;
        file.sync_all()?;
        Ok(())
    }
    #[cfg(windows)]
    {
        if !dir.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "directory not found",
            ));
        }
        Ok(())
    }
}

struct TmpFileGuard<'a>(&'a Path);

impl<'a> Drop for TmpFileGuard<'a> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0);
    }
}

/// Atomically replaces the file at `path` with `bytes`.
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
