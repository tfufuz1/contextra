//! VFS Abstraction Layer for `contextra-store` (SPEC §5.8, D-04, T-LINT-2).
//!
//! Provides filesystem access exclusively through `contextra_ports::vfs` abstractions.

use contextra_ports::vfs::{Vfs, VfsFile, VfsOpenOptions};
use contextra_types::{ContextraError, Result};
use std::fs as std_file_type;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// In-crate VfsFile implementation backed by OS file handle for positional reads.
pub struct StorePosixFile {
    inner: Mutex<std_file_type::File>,
}

impl StorePosixFile {
    pub fn new(file: std_file_type::File) -> Self {
        Self {
            inner: Mutex::new(file),
        }
    }
}

impl VfsFile for StorePosixFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            let guard = self
                .inner
                .lock()
                .map_err(|_| std::io::Error::other("mutex poisoned"))?;
            guard.read_at(buf, offset)
        }
        #[cfg(not(unix))]
        {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| std::io::Error::other("mutex poisoned"))?;
            guard.seek(SeekFrom::Start(offset))?;
            use std::io::Read;
            guard.read(buf)
        }
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            let guard = self
                .inner
                .lock()
                .map_err(|_| std::io::Error::other("mutex poisoned"))?;
            guard.write_at(buf, offset)
        }
        #[cfg(not(unix))]
        {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| std::io::Error::other("mutex poisoned"))?;
            guard.seek(SeekFrom::Start(offset))?;
            use std::io::Write;
            guard.write(buf)
        }
    }

    fn set_len(&self, len: u64) -> std::io::Result<()> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("mutex poisoned"))?;
        guard.set_len(len)
    }

    fn len(&self) -> std::io::Result<u64> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("mutex poisoned"))?;
        guard.metadata().map(|m| m.len())
    }

    fn sync_data(&self) -> std::io::Result<()> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("mutex poisoned"))?;
        guard.sync_data()
    }

    fn sync_all(&self) -> std::io::Result<()> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("mutex poisoned"))?;
        guard.sync_all()
    }
}

pub async fn open_vfs_file(path: &Path) -> Result<(Arc<dyn VfsFile>, u64)> {
    let file = tokio::fs::File::open(path)
        .await
        .map_err(|e| ContextraError::Storage(format!("File open failed: {}", e)))?;
    let metadata = file
        .metadata()
        .await
        .map_err(|e| ContextraError::Storage(format!("Metadata failed: {}", e)))?;
    let file_size = metadata.len();
    let std_file = file.into_std().await;
    Ok((Arc::new(StorePosixFile::new(std_file)), file_size))
}

/// Default filesystem wrapper implementing POSIX durable operations through ports.
pub struct StorageVfs;

impl StorageVfs {
    #[inline]
    pub fn read_dir(vfs: &dyn Vfs, path: &Path) -> Result<Vec<PathBuf>> {
        vfs.read_dir(path)
    }

    #[inline]
    pub fn read(vfs: &dyn Vfs, path: &Path) -> Result<Vec<u8>> {
        vfs.read(path)
    }

    #[inline]
    pub fn write_atomic(vfs: &dyn Vfs, path: &Path, data: &[u8]) -> Result<()> {
        vfs.write_atomic(path, data)
    }

    #[inline]
    pub fn remove_file(vfs: &dyn Vfs, path: &Path) -> Result<()> {
        vfs.remove_file(path)
    }

    #[inline]
    pub fn rename(vfs: &dyn Vfs, from: &Path, to: &Path) -> Result<()> {
        vfs.rename(from, to)
    }

    #[inline]
    pub fn create_dir_all(vfs: &dyn Vfs, path: &Path) -> Result<()> {
        vfs.create_dir_all(path)
    }

    #[inline]
    pub fn exists(vfs: &dyn Vfs, path: &Path) -> Result<bool> {
        vfs.exists(path)
    }

    #[inline]
    pub fn len(vfs: &dyn Vfs, path: &Path) -> Result<u64> {
        vfs.len(path)
    }

    #[inline]
    pub fn sync_dir(vfs: &dyn Vfs, path: &Path) -> Result<()> {
        vfs.sync_dir(path)
    }

    #[inline]
    pub fn open(vfs: &dyn Vfs, path: &Path, options: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        vfs.open(path, options)
    }
}
