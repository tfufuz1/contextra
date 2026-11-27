//! Central module for file and directory synchronization operations.

use crate::io_seam::StoreIo;
use contextra_types::Result;
use std::path::Path;

/// Performs fsync on a single file at `path` through the real filesystem seam.
///
/// # Errors
/// Returns `ContextraError::Storage` on Unix if opening or syncing fails.
/// Returns `ContextraError::Io` with `ErrorKind::Unsupported` on Windows (E20).
pub async fn sync_file(path: &Path) -> Result<()> {
    StoreIo::default().sync_file(path).await
}

/// Performs fsync on the parent directory of `path` through the real filesystem seam.
///
/// Directory fsync is required on POSIX filesystems to guarantee that newly created files or
/// directory entries are durably persisted to disk.
///
/// # Errors
/// Returns `ContextraError::Storage` on Unix if opening or syncing fails.
/// Returns `ContextraError::Io` with `ErrorKind::Unsupported` on Windows (E20).
pub async fn sync_parent_dir(path: &Path) -> Result<()> {
    StoreIo::default().sync_parent_dir(path).await
}

/// Alias for `sync_parent_dir` for backward compatibility across storage modules.
pub async fn fsync_parent_dir(path: &Path) -> Result<()> {
    sync_parent_dir(path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    #[cfg(not(windows))]
    async fn test_sync_file_and_parent_dir_unix() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("test_file.txt");
        tokio::fs::write(&file_path, b"test payload")
            .await
            .expect("write");

        sync_file(&file_path).await.expect("sync_file failed");
        sync_parent_dir(&file_path)
            .await
            .expect("sync_parent_dir failed");
        fsync_parent_dir(&file_path)
            .await
            .expect("fsync_parent_dir failed");
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_sync_file_and_parent_dir_windows() {
        let path = std::path::Path::new("C:\\fake\\path");
        let err = sync_parent_dir(path).await.unwrap_err();
        if let ContextraError::Io(io_err) = err {
            assert_eq!(io_err.kind(), std::io::ErrorKind::Unsupported);
        } else {
            panic!("Expected ContextraError::Io with ErrorKind::Unsupported");
        }

        let err_file = sync_file(path).await.unwrap_err();
        if let ContextraError::Io(io_err) = err_file {
            assert_eq!(io_err.kind(), std::io::ErrorKind::Unsupported);
        } else {
            panic!("Expected ContextraError::Io with ErrorKind::Unsupported");
        }
    }
}
