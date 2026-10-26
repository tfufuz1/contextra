// FILE-CONTEXT: Central fsync module for file and parent directory durability in contextra-store.
//! Central module for explicit file and parent directory synchronization.
//!
//! Enforces fail-closed durability requirements (REQ-FGT-4, INV-IO-1) and OS compatibility (E20).

use contextra_core::{ContextraError, Result};
use std::path::Path;

/// Syncs all in-memory metadata and data of a file to disk.
///
/// # Errors
/// Returns `ContextraError::Storage` or `ContextraError::Io` if syncing fails or if the OS is unsupported.
#[allow(dead_code)]
#[cfg(not(windows))]
pub(crate) async fn sync_file(file: &crate::wal::fs::File) -> Result<()> {
    file.sync_all().await.map_err(|e| {
        ContextraError::Storage(format!("File sync_all failed: {e}"))
    })
}

/// Syncs all in-memory metadata and data of a file to disk.
///
/// # Errors
/// On Windows, returns `ContextraError::Io` with `ErrorKind::Unsupported` per Plattformentscheidung E20.
#[allow(dead_code)]
#[cfg(windows)]
pub(crate) async fn sync_file(_file: &crate::wal::fs::File) -> Result<()> {
    Err(windows_unsupported_error("sync_file"))
}

/// Performs fsync on the parent directory of `path`.
///
/// Directory fsync is required on POSIX filesystems to guarantee that newly created files or
/// directory entries are durably persisted to disk.
///
/// # Errors
/// Returns `ContextraError::Storage` if opening or syncing the parent directory fails.
/// On Windows, returns `ContextraError::Io` with `ErrorKind::Unsupported` per Plattformentscheidung E20.
#[cfg(not(windows))]
pub(crate) async fn sync_parent_dir(path: &Path) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let dir_path = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };

    let dir = crate::wal::fs::File::open(dir_path).await.map_err(|e| {
        ContextraError::Storage(format!(
            "Directory open failed for fsync on {}: {e}",
            dir_path.display()
        ))
    })?;

    dir.sync_all().await.map_err(|e| {
        ContextraError::Storage(format!(
            "Directory fsync failed for {}: {e}",
            dir_path.display()
        ))
    })?;

    Ok(())
}

/// Performs fsync on the parent directory of `path`.
///
/// # Errors
/// On Windows, returns `ContextraError::Io` with `ErrorKind::Unsupported` per Plattformentscheidung E20.
#[cfg(windows)]
pub(crate) async fn sync_parent_dir(_path: &Path) -> Result<()> {
    Err(windows_unsupported_error("sync_parent_dir"))
}

/// Constructs a fail-closed unsupported error for Windows runtime attempts (Plattformentscheidung E20).
#[allow(dead_code)]
pub(crate) fn windows_unsupported_error(op_name: &str) -> ContextraError {
    let io_err = std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!(
            "{op_name} is not supported on Windows in Version 1 (Plattformentscheidung E20)"
        ),
    );
    ContextraError::Io(io_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(not(windows))]
    async fn test_sync_parent_dir_and_file_unix_success() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let file_path = temp_dir.path().join("test_file.txt");

        let file = crate::wal::fs::File::create(&file_path)
            .await
            .expect("create file");

        // Test sync_file
        sync_file(&file).await.expect("sync_file should succeed on Unix");

        // Test sync_parent_dir
        sync_parent_dir(&file_path)
            .await
            .expect("sync_parent_dir should succeed on Unix");
    }

    #[test]
    fn test_windows_unsupported_error_kind() {
        let err = windows_unsupported_error("sync_parent_dir");
        if let ContextraError::Io(io_err) = err {
            assert_eq!(io_err.kind(), std::io::ErrorKind::Unsupported);
            assert!(io_err.to_string().contains("Windows in Version 1"));
        } else {
            panic!("Expected ContextraError::Io, got {err:?}");
        }
    }
}
