// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Mirror trait interface matching contextra_durable_fs::vfs contract.
// INVARIANTEN: Zero external dependencies beyond std.

use std::io;
use std::path::{Path, PathBuf};

/// Options for opening files in `SimVfs`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SimOpenOptions {
    /// Read access requested.
    pub read: bool,
    /// Write access requested.
    pub write: bool,
    /// Create new file if absent.
    pub create: bool,
    /// Create new file, failing if already exists.
    pub create_new: bool,
    /// Truncate file on open.
    pub truncate: bool,
    /// Append mode.
    pub append: bool,
}

impl SimOpenOptions {
    /// Creates a default `SimOpenOptions`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets read option.
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    /// Sets write option.
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    /// Sets create option.
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// Sets create_new option.
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    /// Sets truncate option.
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    /// Sets append option.
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }
}

/// Virtual File System interface for deterministic simulation.
pub trait SimVfs: Send + Sync + 'static {
    /// Opens a file with given options.
    fn open(&self, path: &Path, opts: &SimOpenOptions) -> io::Result<Box<dyn SimVfsFile>>;
    /// Creates directories recursively.
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    /// Removes a file.
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    /// Renames a file or directory.
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    /// Lists contents of a directory.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
    /// Checks if a path exists.
    fn exists(&self, path: &Path) -> io::Result<bool>;
    /// Syncs parent directory state to durable storage.
    fn sync_dir(&self, path: &Path) -> io::Result<()>;
}

/// Handle to an open file in `SimVfs`.
pub trait SimVfsFile: Send + Sync {
    /// Reads data into `buf` starting at `offset`.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize>;
    /// Writes `buf` starting at `offset`.
    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize>;
    /// Sets length of file.
    fn set_len(&self, len: u64) -> io::Result<()>;
    /// Gets length of file.
    fn len(&self) -> io::Result<u64>;
    /// Checks if file is empty.
    fn is_empty(&self) -> io::Result<bool> {
        Ok(self.len()? == 0)
    }
    /// Flushes file data pages to durable storage.
    fn sync_data(&self) -> io::Result<()>;
    /// Flushes file data and metadata pages to durable storage.
    fn sync_all(&self) -> io::Result<()>;
}
