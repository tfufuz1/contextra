// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Virtual File System (VFS) Abstraktion fuer deterministische I/O-Injektion und Testbarkeit.
// INVARIANTEN: Zero-Panic, Fail-Closed, SCHNITTSTELLENVERTRAG exakt eingehalten.

use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::FileExt;

#[cfg(windows)]
use std::os::windows::fs::FileExt;

/// File opening options passed to [`Vfs::open`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VfsOpenOptions {
    /// Open for reading.
    pub read: bool,
    /// Open for writing.
    pub write: bool,
    /// Create new file if it does not exist.
    pub create: bool,
    /// Create new file, failing if file already exists.
    pub create_new: bool,
    /// Truncate existing file length to 0.
    pub truncate: bool,
    /// Append write mode.
    pub append: bool,
}

/// Abstract virtual filesystem interface for storage engines.
pub trait Vfs: Send + Sync + 'static {
    /// Opens a file according to specified options.
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> std::io::Result<Box<dyn VfsFile>>;
    /// Creates directory and all parent directories.
    fn create_dir_all(&self, path: &Path) -> std::io::Result<()>;
    /// Removes a file from filesystem.
    fn remove_file(&self, path: &Path) -> std::io::Result<()>;
    /// Renames a file atomically from `from` to `to`.
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    /// Lists all entries inside a directory path.
    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>>;
    /// Checks if a file or directory exists.
    fn exists(&self, path: &Path) -> std::io::Result<bool>;
    /// Synchronizes directory metadata to stable storage.
    fn sync_dir(&self, path: &Path) -> std::io::Result<()>;
}

/// Abstract positional file handle returned by [`Vfs::open`].
pub trait VfsFile: Send + Sync {
    /// Positional read into `buf` from `offset`.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;
    /// Positional write of `buf` at `offset`.
    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>;
    /// Truncates or extends file length.
    fn set_len(&self, len: u64) -> std::io::Result<()>;
    /// Queries physical file length in bytes.
    fn len(&self) -> std::io::Result<u64>;
    /// Returns `true` if file length is 0 bytes.
    fn is_empty(&self) -> std::io::Result<bool> {
        Ok(self.len()? == 0)
    }
    /// Synchronizes data bytes to disk.
    fn sync_data(&self) -> std::io::Result<()>;
    /// Synchronizes data bytes and metadata to disk.
    fn sync_all(&self) -> std::io::Result<()>;
}

/// Standard real filesystem implementation delegating to [`std::fs`].
#[derive(Debug, Clone, Copy, Default)]
pub struct RealFs;

impl Vfs for RealFs {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> std::io::Result<Box<dyn VfsFile>> {
        let mut std_opts = OpenOptions::new();
        std_opts
            .read(opts.read)
            .write(opts.write)
            .create(opts.create)
            .create_new(opts.create_new)
            .truncate(opts.truncate)
            .append(opts.append);

        let file = std_opts.open(path)?;
        Ok(Box::new(RealFsFile { file }))
    }

    fn create_dir_all(&self, path: &Path) -> std::io::Result<()> {
        fs::create_dir_all(path)
    }

    fn remove_file(&self, path: &Path) -> std::io::Result<()> {
        crate::durable_remove(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        contextra_sys::rename_durable(from, to)?;
        if let Some(parent) = to.parent() {
            crate::sync_dir(parent)?;
        }
        if let Some(from_parent) = from.parent() {
            if from.parent() != to.parent() {
                crate::sync_dir(from_parent)?;
            }
        }
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        let entries = fs::read_dir(path)?;
        let mut res = Vec::new();
        for entry in entries {
            res.push(entry?.path());
        }
        Ok(res)
    }

    fn exists(&self, path: &Path) -> std::io::Result<bool> {
        Ok(path.exists())
    }

    fn sync_dir(&self, path: &Path) -> std::io::Result<()> {
        crate::sync_dir(path)
    }
}

/// Positional [`std::fs::File`] wrapper implementing [`VfsFile`].
#[derive(Debug)]
pub struct RealFsFile {
    file: File,
}

impl VfsFile for RealFsFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            FileExt::read_at(&self.file, buf, offset)
        }
        #[cfg(windows)]
        {
            FileExt::seek_read(&self.file, buf, offset)
        }
        #[cfg(not(any(unix, windows)))]
        {
            compile_error!("Unsupported target OS for positional VfsFile");
        }
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            FileExt::write_at(&self.file, buf, offset)
        }
        #[cfg(windows)]
        {
            FileExt::seek_write(&self.file, buf, offset)
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
