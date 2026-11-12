//! Standard-POSIX- und Windows-Implementierung des VFS für `contextra-store`.
//!
//! Invarianten:
//! - INV-FSYNC-1: Dauerhafte Persistierung durch fdatasync/fsync vor Bestätigung.
//! - Atomares Schreiben: tmp-Datei -> sync_all -> rename -> parent sync.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::error::{StoreError, StoreResult as Result};
use super::types::{Vfs, VfsFile, VfsOpenOptions};

/// Standard POSIX/Betriebssystem-Dateisystemimplementierung für `contextra-store`.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealFs;

impl Vfs for RealFs {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        let mut o = OpenOptions::new();
        o.read(opts.read);
        o.write(opts.write);
        o.create(opts.create);
        o.create_new(opts.create_new);
        o.truncate(opts.truncate);
        let f = o.open(path)?;
        Ok(Box::new(RealFsFile::new(f)))
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        let parent = path.parent().ok_or_else(|| {
            StoreError::InvalidInput("Path has no parent directory".into())
        })?;

        let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));

        // 1. Write to tmp file
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .map_err(StoreError::Io)?;
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
        let f = File::open(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StoreError::NotFound(path.display().to_string())
            } else {
                StoreError::Io(e)
            }
        })?;
        f.sync_all().map_err(StoreError::Io)
    }
}

/// Dateihandle für `RealFs`.
#[derive(Debug)]
pub struct RealFsFile {
    file: File,
}

impl RealFsFile {
    /// Erzeugt ein neues `RealFsFile` aus einem offenen Dateihandle.
    pub fn new(file: File) -> Self {
        Self { file }
    }

    /// Liefert eine Referenz auf das innere Dateihandle.
    pub fn file(&self) -> &File {
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
