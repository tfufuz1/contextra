//! Central module for file and directory synchronization operations.

use contextra_types::{ContextraError, Result};
use std::path::Path;

/// Performs fsync on a single file at `path`.
///
/// # Errors
/// Returns `ContextraError::Storage` on Unix if opening or syncing fails.
/// Returns `ContextraError::Io` with `ErrorKind::Unsupported` on Windows (E20).
pub async fn sync_file(path: &Path) -> Result<()> {
    #[cfg(not(windows))]
    {
        let file = crate::wal::fs::File::open(path).await.map_err(|e| {
            ContextraError::Storage(format!(
                "File open failed for fsync on {}: {e}",
                path.display()
            ))
        })?;
        file.sync_all().await.map_err(|e| {
            ContextraError::Storage(format!("File fsync failed for {}: {e}", path.display()))
        })?;
        Ok(())
    }
    #[cfg(windows)]
    {
        let _ = path;
        Err(ContextraError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows file sync is unsupported under E20",
        )))
    }
}

/// Performs fsync on the parent directory of `path`.
///
/// Directory fsync is required on POSIX filesystems to guarantee that newly created files or
/// directory entries are durably persisted to disk.
///
/// # Errors
/// Returns `ContextraError::Storage` on Unix if opening or syncing fails.
/// Returns `ContextraError::Io` with `ErrorKind::Unsupported` on Windows (E20).
pub async fn sync_parent_dir(path: &Path) -> Result<()> {
    #[cfg(not(windows))]
    {
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
    #[cfg(windows)]
    {
        let _ = path;
        Err(ContextraError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows directory sync is unsupported under E20",
        )))
    }
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
