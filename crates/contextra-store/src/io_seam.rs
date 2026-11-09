//! # StoreIo / VFS Seam for SSTable, Compaction, and Recovery (Ring 1)
//!
//! Kapselt Datei- und Verzeichnisoperationen über [`contextra_ports::Vfs`].
//! Bietet synchronen und asynchronen Zugriff für SSTable-Reader, Builder,
//! Compaction und Recovery unter Vermeidung direkter std::fs / tokio::fs Aufrufe.

use contextra_ports::{ContextraError, Result, Vfs, VfsFile, VfsOpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Default POSIX/OS filesystem implementation of [`Vfs`] for contextra-store.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealFs;

impl Vfs for RealFs {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        let mut std_opts = std::fs::OpenOptions::new();
        std_opts
            .read(opts.read)
            .write(opts.write)
            .create(opts.create)
            .create_new(opts.create_new)
            .truncate(opts.truncate)
            .append(opts.append);

        let file = std_opts.open(path).map_err(ContextraError::Io)?;
        Ok(Box::new(RealFsFile { file }))
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        std::fs::read(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("tmp_file");
        let tmp_path = parent.join(format!("{}.tmp.{}", file_name, std::process::id()));

        // 1. Write to tmp file
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .map_err(ContextraError::Io)?;
        use std::io::Write;
        f.write_all(data).map_err(ContextraError::Io)?;
        // 2. fsync file
        f.sync_all().map_err(ContextraError::Io)?;
        drop(f);

        // 3. atomic rename
        if let Err(e) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(ContextraError::Io(e));
        }

        // 4. fsync parent dir
        self.sync_dir(parent)?;
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path).map_err(ContextraError::Io)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        std::fs::remove_file(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        std::fs::rename(from, to).map_err(ContextraError::Io)
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = std::fs::read_dir(path).map_err(ContextraError::Io)?;
        let mut res = Vec::new();
        for entry in entries {
            res.push(entry.map_err(ContextraError::Io)?.path());
        }
        Ok(res)
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        Ok(path.exists())
    }

    fn len(&self, path: &Path) -> Result<u64> {
        std::fs::metadata(path).map(|m| m.len()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        let entries = std::fs::read_dir(path).map_err(ContextraError::Io)?;
        let mut res = Vec::new();
        for entry in entries {
            let entry = entry.map_err(ContextraError::Io)?;
            if let Some(name) = entry.file_name().to_str() {
                res.push(name.to_string());
            }
        }
        res.sort();
        Ok(res)
    }

    fn sync_dir(&self, path: &Path) -> Result<()> {
        let f = std::fs::File::open(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })?;
        f.sync_all().map_err(ContextraError::Io)
    }
}

/// Standard [`VfsFile`] handle based on [`std::fs::File`].
#[derive(Debug)]
pub struct RealFsFile {
    file: std::fs::File,
}

impl RealFsFile {
    /// Creates a new `RealFsFile` from an open file handle.
    pub fn new(file: std::fs::File) -> Self {
        Self { file }
    }

    /// Returns a reference to the inner file.
    pub fn file(&self) -> &std::fs::File {
        &self.file
    }
}

impl VfsFile for RealFsFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            self.file.read_at(buf, offset)
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::FileExt;
            self.file.seek_read(buf, offset)
        }
        #[cfg(not(any(unix, windows)))]
        {
            compile_error!("Unsupported target OS for positional VfsFile");
        }
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            self.file.write_at(buf, offset)
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::FileExt;
            self.file.seek_write(buf, offset)
        }
        #[cfg(not(any(unix, windows)))]
        {
            compile_error!("Unsupported target OS for positional VfsFile");
        }
    }

    fn set_len(&self, len: u64) -> std::io::Result<()> {
        self.file.set_len(len)
    }

    fn len(&self) -> std::io::Result<u64> {
        self.file.metadata().map(|m| m.len())
    }

    fn sync_data(&self) -> std::io::Result<()> {
        self.file.sync_data()
    }

    fn sync_all(&self) -> std::io::Result<()> {
        self.file.sync_all()
    }
}

/// Central Store I/O abstraction (`StoreIo`) wrapping `Arc<dyn Vfs>`.
#[derive(Clone)]
pub struct StoreIo {
    vfs: Arc<dyn Vfs>,
}

impl std::fmt::Debug for StoreIo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreIo").finish()
    }
}

impl Default for StoreIo {
    fn default() -> Self {
        Self {
            vfs: Arc::new(RealFs),
        }
    }
}

impl StoreIo {
    /// Creates a new `StoreIo` with the specified `Vfs`.
    pub fn new(vfs: Arc<dyn Vfs>) -> Self {
        Self { vfs }
    }

    /// Returns a reference to the inner `Vfs`.
    pub fn vfs(&self) -> &Arc<dyn Vfs> {
        &self.vfs
    }

    /// Opens a file using positional VfsFile options.
    pub fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        self.vfs.open(path, opts)
    }

    /// Reads an entire file to a vector of bytes.
    pub async fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.read(&p))
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
    }

    /// Reads data positionally from `offset` into `buf`.
    pub async fn read_at(&self, path: &Path, offset: u64, len: usize) -> Result<Vec<u8>> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
            let mut buf = vec![0u8; len];
            let file = vfs.open(&p, &VfsOpenOptions::read_only())?;
            let mut read_bytes = 0;
            while read_bytes < len {
                let n = file
                    .read_at(
                        buf.get_mut(read_bytes..).ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "buffer underflow",
                            )
                        })?,
                        offset + read_bytes as u64,
                    )
                    .map_err(ContextraError::Io)?;
                if n == 0 {
                    return Err(ContextraError::Storage(
                        "Unexpected EOF reading block".into(),
                    ));
                }
                read_bytes += n;
            }
            Ok(buf)
        })
        .await
        .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
    }

    /// Atomically writes data to `path` with crash-consistency.
    pub async fn write_atomic(&self, path: &Path, data: Vec<u8>) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.write_atomic(&p, &data))
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
    }

    /// Creates directories along `path`.
    pub async fn create_dir_all(&self, path: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.create_dir_all(&p))
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
    }

    /// Removes a file at `path`.
    pub async fn remove_file(&self, path: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.remove_file(&p))
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
    }

    /// Atomically renames a file.
    pub async fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let f = from.to_path_buf();
        let t = to.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.rename(&f, &t))
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
    }

    /// Checks if a file exists.
    pub async fn exists(&self, path: &Path) -> Result<bool> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.exists(&p))
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
    }

    /// Returns the length in bytes of the file at `path`.
    pub async fn len(&self, path: &Path) -> Result<u64> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.len(&p))
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
    }

    /// Reads directory entries as PathBufs.
    pub async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.read_dir(&p))
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
    }

    /// Syncs directory metadata.
    pub async fn sync_dir(&self, path: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.sync_dir(&p))
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
    }
}
