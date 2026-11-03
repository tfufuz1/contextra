//! Test Virtual File System (VFS) implementation for DiskANN tests and internal fallback.

use contextra_ports::{ContextraError, Result, Vfs, VfsFile, VfsOpenOptions};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Default)]
struct InMemoryFs {
    files: BTreeMap<PathBuf, Vec<u8>>,
    dirs: BTreeMap<PathBuf, ()>,
}

/// A test VFS implementation that supports in-memory files and local OS test files.
#[derive(Clone, Debug, Default)]
pub struct TestVfs {
    state: Arc<RwLock<InMemoryFs>>,
}

impl TestVfs {
    pub fn new() -> Self {
        Self::default()
    }

    fn is_on_disk(path: &Path) -> bool {
        path.exists()
            || path
                .parent()
                .map(|p| p.exists() && p.is_dir())
                .unwrap_or(false)
    }
}

pub type InMemoryVfs = TestVfs;

struct InMemoryFileHandle {
    path: PathBuf,
    state: Arc<RwLock<InMemoryFs>>,
}

impl VfsFile for InMemoryFileHandle {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        let guard = self.state.read();
        let file_bytes = guard.files.get(&self.path).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!("file not found: {:?}", self.path),
            )
        })?;

        let offset = offset as usize;
        if offset >= file_bytes.len() {
            return Ok(0);
        }

        let available = &file_bytes[offset..];
        let to_read = buf.len().min(available.len());
        buf[..to_read].copy_from_slice(&available[..to_read]);
        Ok(to_read)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        let mut guard = self.state.write();
        let file_bytes = guard.files.entry(self.path.clone()).or_default();
        let offset = offset as usize;

        if offset + buf.len() > file_bytes.len() {
            file_bytes.resize(offset + buf.len(), 0);
        }
        file_bytes[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn set_len(&self, len: u64) -> io::Result<()> {
        let mut guard = self.state.write();
        let file_bytes = guard.files.entry(self.path.clone()).or_default();
        file_bytes.resize(len as usize, 0);
        Ok(())
    }

    fn len(&self) -> io::Result<u64> {
        let guard = self.state.read();
        let file_bytes = guard.files.get(&self.path).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!("file not found: {:?}", self.path),
            )
        })?;
        Ok(file_bytes.len() as u64)
    }

    fn sync_data(&self) -> io::Result<()> {
        Ok(())
    }

    fn sync_all(&self) -> io::Result<()> {
        Ok(())
    }
}

struct OsFileHandle(std::fs::File);

impl VfsFile for OsFileHandle {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            self.0.read_at(buf, offset)
        }
        #[cfg(not(unix))]
        {
            use std::io::{Read, Seek, SeekFrom};
            let mut f = &self.0;
            f.seek(SeekFrom::Start(offset))?;
            f.read(buf)
        }
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            self.0.write_at(buf, offset)
        }
        #[cfg(not(unix))]
        {
            use std::io::{Seek, SeekFrom, Write};
            let mut f = &self.0;
            f.seek(SeekFrom::Start(offset))?;
            f.write(buf)
        }
    }

    fn set_len(&self, len: u64) -> io::Result<()> {
        self.0.set_len(len)
    }

    fn len(&self) -> io::Result<u64> {
        self.0.metadata().map(|m| m.len())
    }

    fn sync_data(&self) -> io::Result<()> {
        self.0.sync_data()
    }

    fn sync_all(&self) -> io::Result<()> {
        self.0.sync_all()
    }
}

impl Vfs for TestVfs {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        let path_buf = path.to_path_buf();
        let mut guard = self.state.write();

        if guard.files.contains_key(&path_buf) {
            if opts.create_new {
                return Err(ContextraError::Io(Error::new(
                    ErrorKind::AlreadyExists,
                    "file already exists",
                )));
            }
            if opts.truncate {
                if let Some(buf) = guard.files.get_mut(&path_buf) {
                    buf.clear();
                }
            }
            return Ok(Box::new(InMemoryFileHandle {
                path: path_buf,
                state: Arc::clone(&self.state),
            }));
        }

        if Self::is_on_disk(path) {
            let mut open_opts = std::fs::OpenOptions::new();
            open_opts.read(opts.read);
            open_opts.write(opts.write);
            open_opts.append(opts.append);
            if opts.create_new {
                open_opts.create_new(true);
            } else if opts.create {
                open_opts.create(true);
            }
            if opts.truncate {
                open_opts.truncate(true);
            }
            let file = open_opts.open(path).map_err(ContextraError::Io)?;
            return Ok(Box::new(OsFileHandle(file)));
        }

        if opts.create || opts.create_new {
            guard.files.insert(path_buf.clone(), Vec::new());
            return Ok(Box::new(InMemoryFileHandle {
                path: path_buf,
                state: Arc::clone(&self.state),
            }));
        }

        Err(ContextraError::NotFound(path.display().to_string()))
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let guard = self.state.read();
        if let Some(data) = guard.files.get(path) {
            return Ok(data.clone());
        }
        if path.exists() {
            return std::fs::read(path).map_err(ContextraError::Io);
        }
        Err(ContextraError::NotFound(path.display().to_string()))
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        if Self::is_on_disk(path) {
            return std::fs::write(path, data).map_err(ContextraError::Io);
        }
        let mut guard = self.state.write();
        guard.files.insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        if Self::is_on_disk(path) {
            return std::fs::create_dir_all(path).map_err(ContextraError::Io);
        }
        let mut guard = self.state.write();
        guard.dirs.insert(path.to_path_buf(), ());
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        let mut guard = self.state.write();
        if guard.files.remove(path).is_some() {
            return Ok(());
        }
        if path.exists() {
            return std::fs::remove_file(path).map_err(ContextraError::Io);
        }
        Err(ContextraError::NotFound(path.display().to_string()))
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let mut guard = self.state.write();
        if let Some(content) = guard.files.remove(from) {
            guard.files.insert(to.to_path_buf(), content);
            return Ok(());
        }
        if from.exists() {
            return std::fs::rename(from, to).map_err(ContextraError::Io);
        }
        Err(ContextraError::NotFound(from.display().to_string()))
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        let guard = self.state.read();
        Ok(guard.files.contains_key(path) || guard.dirs.contains_key(path) || path.exists())
    }

    fn len(&self, path: &Path) -> Result<u64> {
        let guard = self.state.read();
        if let Some(v) = guard.files.get(path) {
            return Ok(v.len() as u64);
        }
        if path.exists() {
            return std::fs::metadata(path)
                .map(|m| m.len())
                .map_err(ContextraError::Io);
        }
        Err(ContextraError::NotFound(path.display().to_string()))
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        let guard = self.state.read();
        let mut names = Vec::new();
        for file_path in guard.files.keys() {
            if let Some(parent) = file_path.parent() {
                if parent == path {
                    if let Some(name) = file_path.file_name() {
                        names.push(name.to_string_lossy().into_owned());
                    }
                }
            }
        }
        if names.is_empty() && path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    names.push(entry.file_name().to_string_lossy().into_owned());
                }
            }
        }
        Ok(names)
    }

    fn sync_dir(&self, path: &Path) -> Result<()> {
        if path.exists() {
            let f = std::fs::File::open(path).map_err(ContextraError::Io)?;
            f.sync_all().map_err(ContextraError::Io)?;
        }
        Ok(())
    }
}
