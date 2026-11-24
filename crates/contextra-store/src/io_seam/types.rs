//! Grundlegende Typen und Traits der VFS-Naht für `contextra-store`.
//!
//! Invarianten:
//! - POSIX-Semantik für Lese-/Schreibzugriffe.
//! - Fail-closed bei I/O-Fehlern.

use std::path::{Path, PathBuf};
use crate::error::{StoreError, StoreResult as Result};

/// Methode, über welche die Medienverifikation durchgeführt wurde.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerifiedVia {
    /// Direkt vom Medium gelesen.
    Direct,
    /// Nach Cache-Invalidierung vom Medium gelesen.
    CacheDropped,
    /// Aus dem Betriebssystem-/Dateisystem-Cache bedient.
    Cached,
}

/// Optionen zum Öffnen von Dateien über das VFS.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VfsOpenOptions {
    /// Lesezugriff aktivieren.
    pub read: bool,
    /// Schreibzugriff aktivieren.
    pub write: bool,
    /// Datei erzeugen, falls nicht existent.
    pub create: bool,
    /// Datei neu erzeugen (Fehler, falls existent).
    pub create_new: bool,
    /// Datei beim Öffnen abschneiden.
    pub truncate: bool,
}

impl VfsOpenOptions {
    /// Standardoptionen für Nur-Lese-Zugriff.
    pub fn read_only() -> Self {
        Self { read: true, ..Default::default() }
    }
}

/// Abstraktes positionelles Dateihandle.
pub trait VfsFile: Send + Sync + 'static {
    /// Liest positionell ab `offset` in `buf`.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;
    /// Schreibt positionell ab `offset` aus `buf`.
    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>;
    /// Setzt die Dateilänge.
    fn set_len(&self, len: u64) -> std::io::Result<()>;
    /// Liefert die Dateilänge in Bytes.
    fn len(&self) -> std::io::Result<u64>;
    /// Synchronisiert Datenblöcke auf Platte (`fdatasync`).
    fn sync_data(&self) -> std::io::Result<()>;
    /// Synchronisiert Datenblöcke und Metadaten auf Platte (`fsync`).
    fn sync_all(&self) -> std::io::Result<()>;
}

/// Abstraktes virtuelles Dateisystem für `contextra-store`.
pub trait Vfs: Send + Sync + 'static {
    /// Öffnet eine Datei mit den übergebenen Optionen.
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>>;
    /// Schreibt Daten atomar über temporäre Datei und Rename.
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>;
    /// Erzeugt ein Verzeichnis rekursiv.
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    /// Entfernt eine Datei.
    fn remove_file(&self, path: &Path) -> Result<()>;
    /// Benennt eine Datei atomar um.
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;
    /// Liest Einträge eines Verzeichnisses als Pfade.
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    /// Prüft auf Existenz eines Pfads.
    fn exists(&self, path: &Path) -> Result<bool>;
    /// Liefert die Dateigröße.
    fn len(&self, path: &Path) -> Result<u64>;
    /// Listet Dateinamen in einem Verzeichnis als Strings auf.
    fn list_dir(&self, path: &Path) -> Result<Vec<String>>;
    /// Synchronisiert Verzeichnis-Metadaten.
    fn sync_dir(&self, path: &Path) -> Result<()>;
    /// Liest eine Datei vollständig ein.
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
