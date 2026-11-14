//! Standard-POSIX-Implementierung des VFS für `contextra-store`.
//!
//! Nutzt atomares Schreiben (tmp -> fsync -> rename -> parent sync)
//! gemäß Invariante INV-FSYNC-1.

use std::fs::{File, OpenOptions};
use std::os::unix::fs::FileExt;
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
        Ok(Box::new(RealFsFile(f)))
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
        std::fs::write(&tmp, data)?;
        let f = File::open(&tmp)?;
        f.sync_data()?;
        drop(f);
        std::fs::rename(&tmp, path)?;
        if let Some(parent) = path.parent() {
            self.sync_dir(parent)?;
        }
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        std::fs::remove_file(path)?;
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        std::fs::rename(from, to)?;
        if let Some(parent) = to.parent() {
            self.sync_dir(parent)?;
        }
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut v = Vec::new();
        for entry in std::fs::read_dir(path)? {
            v.push(entry?.path());
        }
        Ok(v)
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        Ok(path.exists())
    }

    fn len(&self, path: &Path) -> Result<u64> {
        Ok(std::fs::metadata(path)?.len())
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        let mut v = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if let Ok(name) = entry.file_name().into_string() {
                v.push(name);
            }
        }
        Ok(v)
    }

    fn sync_dir(&self, path: &Path) -> Result<()> {
        let d = File::open(path)?;
        d.sync_all()?;
        Ok(())
    }
}

/// Dateihandle für `RealFs`.
#[derive(Debug)]
pub struct RealFsFile(pub File);

impl VfsFile for RealFsFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        self.0.read_at(buf, offset)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        self.0.write_at(buf, offset)
    }

    fn set_len(&self, len: u64) -> std::io::Result<()> {
        self.0.set_len(len)
    }

    fn len(&self) -> std::io::Result<u64> {
        Ok(self.0.metadata()?.len())
    }

    fn sync_data(&self) -> std::io::Result<()> {
        self.0.sync_data()
    }

    fn sync_all(&self) -> std::io::Result<()> {
        self.0.sync_all()
    }
}
