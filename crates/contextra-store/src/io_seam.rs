//! Kapselt Datei- und Verzeichnisoperationen für contextra-store.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::error::{StoreError, StoreResult as Result};

/// Method via which medium verification was performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerifiedVia {
    Direct,
    CacheDropped,
    Cached,
}

/// Options configuring how a file is opened via [].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VfsOpenOptions {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub create_new: bool,
    pub truncate: bool,
}

impl VfsOpenOptions {
    pub fn read_only() -> Self {
        Self { read: true, ..Default::default() }
    }
}

/// Abstract positional file handle.
pub trait VfsFile: Send + Sync + 'static {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;
    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>;
    fn set_len(&self, len: u64) -> std::io::Result<()>;
    fn len(&self) -> std::io::Result<u64>;
    fn sync_data(&self) -> std::io::Result<()>;
    fn sync_all(&self) -> std::io::Result<()>;
}

/// Abstract virtual file system for contextra-store.
pub trait Vfs: Send + Sync + 'static {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>>;
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn remove_file(&self, path: &Path) -> Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    fn exists(&self, path: &Path) -> Result<bool>;
    fn len(&self, path: &Path) -> Result<u64>;
    fn list_dir(&self, path: &Path) -> Result<Vec<String>>;
    fn sync_dir(&self, path: &Path) -> Result<()>;
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        std::fs::read(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StoreError::NotFound(path.display().to_string())
            } else {
                StoreError::from(e)
            }
        })
    }
}

/// Default POSIX/OS filesystem implementation of [] for contextra-store.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealFs;

impl Vfs for RealFs {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        let mut std_opts = std::fs::OpenOptions::new();
        std_opts.read(opts.read);
        std_opts.write(opts.write);
        std_opts.create(opts.create);
        std_opts.create_new(opts.create_new);
        std_opts.truncate(opts.truncate);

        let file = std_opts.open(path).map_err(StoreError::Io)?;
        Ok(Box::new(RealFsFile::new(file)))
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        let parent = path.parent().ok_or_else(|| {
            StoreError::InvalidInput("Path has no parent directory".into())
        })?;

        let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));

        // 1. Write to tmp file
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .map_err(StoreError::Io)?;
        use std::io::Write;
        f.write_all(data).map_err(StoreError::Io)?;
        // 2. fsync file
        f.sync_all().map_err(StoreError::Io)?;
        drop(f);

        // 3. atomic rename
        if let Err(e) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(StoreError::Io(e));
        }

        // 4. fsync parent dir
        self.sync_dir(parent)?;
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path).map_err(StoreError::Io)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        std::fs::remove_file(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StoreError::NotFound(path.display().to_string())
            } else {
                StoreError::Io(e)
            }
        })
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        std::fs::rename(from, to).map_err(StoreError::Io)
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = std::fs::read_dir(path).map_err(StoreError::Io)?;
        let mut res = Vec::new();
        for entry in entries {
            res.push(entry.map_err(StoreError::Io)?.path());
        }
        Ok(res)
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        Ok(path.exists())
    }

    fn len(&self, path: &Path) -> Result<u64> {
        std::fs::metadata(path).map(|m| m.len()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StoreError::NotFound(path.display().to_string())
            } else {
                StoreError::Io(e)
            }
        })
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        let entries = std::fs::read_dir(path).map_err(StoreError::Io)?;
        let mut res = Vec::new();
        for entry in entries {
            let entry = entry.map_err(StoreError::Io)?;
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
                StoreError::NotFound(path.display().to_string())
            } else {
                StoreError::Io(e)
            }
        })?;
        f.sync_all().map_err(StoreError::Io)
    }
}

/// Standard [] handle based on [].
#[derive(Debug)]
pub struct RealFsFile {
    file: std::fs::File,
}

impl RealFsFile {
    /// Creates a new  from an open file handle.
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

/// Central Store I/O abstraction () wrapping .
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
    /// Creates a new  with the specified .
    pub fn new(vfs: Arc<dyn Vfs>) -> Self {
        Self { vfs }
    }

    /// Returns a reference to the inner .
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
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Reads data positionally from  into .
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
                    .map_err(StoreError::Io)?;
                if n == 0 {
                    return Err(StoreError::Storage(
                        "Unexpected EOF reading block".into(),
                    ));
                }
                read_bytes += n;
            }
            Ok(buf)
        })
        .await
        .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Atomically writes data to  with crash-consistency.
    pub async fn write_atomic(&self, path: &Path, data: Vec<u8>) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.write_atomic(&p, &data))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Creates directories along .
    pub async fn create_dir_all(&self, path: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.create_dir_all(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Removes a file at .
    pub async fn remove_file(&self, path: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.remove_file(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Atomically renames a file.
    pub async fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let f = from.to_path_buf();
        let t = to.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.rename(&f, &t))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Checks if a file exists.
    pub async fn exists(&self, path: &Path) -> Result<bool> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.exists(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Returns the length in bytes of the file at .
    pub async fn len(&self, path: &Path) -> Result<u64> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.len(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Reads directory entries as PathBufs.
    pub async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.read_dir(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Syncs directory metadata.
    pub async fn sync_dir(&self, path: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.sync_dir(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }
}
