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

/// Named execution step for fault injection in [`FailpointVfs`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailpointStep {
    /// File write operation.
    Write,
    /// File sync / fsync operation.
    FsyncFile,
    /// File rename operation.
    Rename,
    /// Directory sync / fsync operation.
    SyncDir,
    /// File remove operation.
    Remove,
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A VFS wrapper supporting step-specific fault injection and call counting.
#[derive(Debug)]
pub struct FailpointVfs<V: Vfs = RealFs> {
    inner: V,
    fail_steps: Arc<Mutex<HashMap<FailpointStep, bool>>>,
    fail_after_counts: Arc<Mutex<HashMap<FailpointStep, usize>>>,
    call_counts: Arc<Mutex<HashMap<FailpointStep, usize>>>,
}

impl<V: Vfs + Clone> Clone for FailpointVfs<V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            fail_steps: Arc::clone(&self.fail_steps),
            fail_after_counts: Arc::clone(&self.fail_after_counts),
            call_counts: Arc::clone(&self.call_counts),
        }
    }
}

impl Default for FailpointVfs<RealFs> {
    fn default() -> Self {
        Self::new()
    }
}

impl FailpointVfs<RealFs> {
    /// Creates a new `FailpointVfs` wrapping [`RealFs`].
    #[must_use]
    pub fn new() -> Self {
        Self::with_vfs(RealFs)
    }
}

impl<V: Vfs> FailpointVfs<V> {
    /// Creates a new `FailpointVfs` wrapping `inner`.
    pub fn with_vfs(inner: V) -> Self {
        Self {
            inner,
            fail_steps: Arc::new(Mutex::new(HashMap::new())),
            fail_after_counts: Arc::new(Mutex::new(HashMap::new())),
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Enables immediate fault injection for `step`.
    pub fn set_fail_step(&self, step: FailpointStep) {
        if let Ok(mut map) = self.fail_steps.lock() {
            map.insert(step, true);
        }
    }

    /// Enables fault injection for `step` starting after `count` calls.
    pub fn set_fail_after(&self, step: FailpointStep, count: usize) {
        if let Ok(mut map) = self.fail_after_counts.lock() {
            map.insert(step, count);
        }
    }

    /// Returns the total call count for `step`.
    #[must_use]
    pub fn call_count(&self, step: FailpointStep) -> usize {
        if let Ok(map) = self.call_counts.lock() {
            *map.get(&step).unwrap_or(&0)
        } else {
            0
        }
    }

    /// Resets all failpoint settings and call counters.
    pub fn reset(&self) {
        if let Ok(mut map) = self.fail_steps.lock() {
            map.clear();
        }
        if let Ok(mut map) = self.fail_after_counts.lock() {
            map.clear();
        }
        if let Ok(mut map) = self.call_counts.lock() {
            map.clear();
        }
    }

    fn check_and_count(&self, step: FailpointStep) -> std::io::Result<()> {
        let mut count = 0;
        if let Ok(mut counts) = self.call_counts.lock() {
            let entry = counts.entry(step).or_insert(0);
            *entry += 1;
            count = *entry;
        }

        if let Ok(steps) = self.fail_steps.lock() {
            if steps.get(&step).copied().unwrap_or(false) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Injected fault at step {:?}", step),
                ));
            }
        }

        if let Ok(afters) = self.fail_after_counts.lock() {
            if let Some(&threshold) = afters.get(&step) {
                if count >= threshold {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Injected fault at step {:?} after call count {}", step, count),
                    ));
                }
            }
        }

        Ok(())
    }
}

impl<V: Vfs + Clone> Vfs for FailpointVfs<V> {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> std::io::Result<Box<dyn VfsFile>> {
        let file = self.inner.open(path, opts)?;
        Ok(Box::new(FailpointVfsFile {
            inner: file,
            vfs: self.clone(),
        }))
    }

    fn create_dir_all(&self, path: &Path) -> std::io::Result<()> {
        self.inner.create_dir_all(path)
    }

    fn remove_file(&self, path: &Path) -> std::io::Result<()> {
        self.check_and_count(FailpointStep::Remove)?;
        self.inner.remove_file(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        self.check_and_count(FailpointStep::Rename)?;
        self.inner.rename(from, to)
    }

    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        self.inner.read_dir(path)
    }

    fn exists(&self, path: &Path) -> std::io::Result<bool> {
        self.inner.exists(path)
    }

    fn sync_dir(&self, path: &Path) -> std::io::Result<()> {
        self.check_and_count(FailpointStep::SyncDir)?;
        self.inner.sync_dir(path)
    }
}

struct FailpointVfsFile<V: Vfs + Clone> {
    inner: Box<dyn VfsFile>,
    vfs: FailpointVfs<V>,
}

impl<V: Vfs + Clone> VfsFile for FailpointVfsFile<V> {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        self.inner.read_at(buf, offset)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        self.vfs.check_and_count(FailpointStep::Write)?;
        self.inner.write_at(buf, offset)
    }

    fn set_len(&self, len: u64) -> std::io::Result<()> {
        self.inner.set_len(len)
    }

    fn len(&self) -> std::io::Result<u64> {
        self.inner.len()
    }

    fn sync_data(&self) -> std::io::Result<()> {
        self.vfs.check_and_count(FailpointStep::FsyncFile)?;
        self.inner.sync_data()
    }

    fn sync_all(&self) -> std::io::Result<()> {
        self.vfs.check_and_count(FailpointStep::FsyncFile)?;
        self.inner.sync_all()
    }
}
