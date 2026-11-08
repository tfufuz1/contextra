//! # Virtuelles Dateisystem (VFS - Virtual File System)
//!
//! Zweck: Bereitstellung einer Schnittstellenabstraktion für Dateisystemoperationen,
//! um positionsbasiertes I/O (`read_at`, `write_at`), Dateisynchronisation (`sync_data`, `sync_all`)
//! und Verzeichnisverwaltung testbar und mockbar zu machen.
//!
//! Ring-Zugehörigkeit: Ring 0 (I/O- und Speicherschnittstellen).
//! Wird genutzt von: Index- und Speicher-Engines (z. B. HNSW-Vektorindex, Blockmanager).
//! Wichtigste Typen:
//! - [`RealFs`]: Standardimplementierung von [`contextra_ports::Vfs`], die Aufrufe an das reale Host-Dateisystem weiterleitet.
//! - [`RealFsFile`]: Dateihandle-Wrapper für [`std::fs::File`] mit Implementierung von [`contextra_ports::VfsFile`].
//!
//! Invarianten:
//! - Schnittstellenvertrag: Deterministische I/O-Fehlerweiterleitung ohne `panic!`.
//! - Plattformunabhängigkeit: Verwendet auf Unix `FileExt::read_at`/`write_at` und auf Windows `FileExt::seek_read`/`seek_write`.

// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Virtual File System (VFS) Abstraktion fuer deterministische I/O-Injektion und Testbarkeit.
// INVARIANTEN: Zero-Panic, Fail-Closed, SCHNITTSTELLENVERTRAG exakt eingehalten.

use contextra_ports::{ContextraError, Result, VerifiedVia, Vfs, VfsFile, VfsOpenOptions};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::FileExt;

#[cfg(windows)]
use std::os::windows::fs::FileExt;

pub use contextra_ports::{
    Vfs as PortsVfs, VfsFile as PortsVfsFile, VfsOpenOptions as PortsVfsOpenOptions,
};

// ANCHOR[contextra-durable-fs:REAL-FS-001]
/// Standard-Implementierung von [`contextra_ports::Vfs`], die direkt mit dem Host-Betriebssystem interagiert.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealFs;

impl Vfs for RealFs {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        let mut std_opts = OpenOptions::new();
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
        crate::atomic_replace(path, data).map_err(ContextraError::Io)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path).map_err(ContextraError::Io)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        crate::durable_remove(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        crate::rename_durable(from, to).map_err(ContextraError::Io)
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = fs::read_dir(path).map_err(ContextraError::Io)?;
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
        fs::metadata(path).map(|m| m.len()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        let entries = fs::read_dir(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })?;
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
        crate::sync_dir(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })
    }
}

// ANCHOR[contextra-durable-fs:REAL-FS-FILE-001]
/// Implementierung von [`contextra_ports::VfsFile`] auf Basis von [`std::fs::File`].
#[derive(Debug)]
pub struct RealFsFile {
    file: File,
}

impl RealFsFile {
    /// Erzeugt ein neues [`RealFsFile`] aus einem offenen Dateihandle.
    pub fn new(file: File) -> Self {
        Self { file }
    }

    /// Gibt eine Referenz auf die zugrundeliegende [`File`] zurück.
    pub fn file(&self) -> &File {
        &self.file
    }
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

    fn read_verified(&self, offset: u64, len: usize) -> std::io::Result<(Vec<u8>, VerifiedVia)> {
        let mut buf = vec![0u8; len];
        let n = self.read_at(&mut buf, offset)?;
        buf.truncate(n);
        Ok((buf, VerifiedVia::Cached))
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
        contextra_sys::durable_flush_file(&self.file, false)
    }

    fn sync_all(&self) -> std::io::Result<()> {
        contextra_sys::durable_flush_file(&self.file, true)
    }
}
