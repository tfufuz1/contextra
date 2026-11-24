//! # Lokale Nähte und Verträge für contextra-store (Ring 1)
//!
//! Definiert die vom Speicherkern benötigten Verträge (Vfs, Clock, Rng, Metrics, Storage)
//! ohne Kopplung an `contextra-ports` oder `contextra-types`.

use crate::error::StoreResult;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Positional I/O-Zugriff auf eine geöffnete Datei.
pub trait StoreFile: Send + Sync + Debug + 'static {
    /// Liest positionell bei `offset` in `buf`.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;

    /// Schreibt positionell bei `offset` aus `buf`.
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

/// Öffnungs-Optionen für `StoreFile`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreFileOpenOptions {
    /// Lesezugriff aktivieren.
    pub read: bool,
    /// Schreibzugriff aktivieren.
    pub write: bool,
    /// Datei anlegen, falls nicht vorhanden.
    pub create: bool,
    /// Datei neu anlegen (Fehler wenn vorhanden).
    pub create_new: bool,
    /// Datei beim Öffnen abschneiden.
    pub truncate: bool,
    /// An Dateiende anhängen.
    pub append: bool,
}

impl StoreFileOpenOptions {
    /// Standardoptionen für reines Lesen.
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            create: false,
            create_new: false,
            truncate: false,
            append: false,
        }
    }

    /// Standardoptionen für reines Erzeugen / Schreiben.
    pub fn create_new() -> Self {
        Self {
            read: false,
            write: true,
            create: true,
            create_new: true,
            truncate: false,
            append: false,
        }
    }
}

/// Lokale VFS-Naht für den Speicherkern.
pub trait StoreFs: Send + Sync + 'static {
    /// Öffnet eine Datei mit den gegebenen Optionen.
    fn open(&self, path: &Path, opts: &StoreFileOpenOptions) -> StoreResult<Box<dyn StoreFile>>;

    /// Liest eine Datei vollständig ein.
    fn read(&self, path: &Path) -> StoreResult<Vec<u8>>;

    /// Schreibt Daten atomar und crash-konsistent.
    fn write_atomic(&self, path: &Path, data: &[u8]) -> StoreResult<()>;

    /// Erzeugt ein Verzeichnis und alle Elternverzeichnisse.
    fn create_dir_all(&self, path: &Path) -> StoreResult<()>;

    /// Löscht eine Datei.
    fn remove_file(&self, path: &Path) -> StoreResult<()>;

    /// Benennt eine Datei atomar um.
    fn rename(&self, from: &Path, to: &Path) -> StoreResult<()>;

    /// Prüft, ob ein Pfad existiert.
    fn exists(&self, path: &Path) -> StoreResult<bool>;

    /// Liefert die Dateigröße.
    fn len(&self, path: &Path) -> StoreResult<u64>;

    /// Listet Dateinamen in einem Verzeichnis auf.
    fn list_dir(&self, path: &Path) -> StoreResult<Vec<String>>;

    /// Synchronisiert Verzeichnis-Metadaten (`fsync` auf Directory).
    fn sync_dir(&self, path: &Path) -> StoreResult<()>;
}

impl<T: StoreFs + ?Sized> StoreFs for Arc<T> {
    fn open(&self, path: &Path, opts: &StoreFileOpenOptions) -> StoreResult<Box<dyn StoreFile>> {
        (**self).open(path, opts)
    }
    fn read(&self, path: &Path) -> StoreResult<Vec<u8>> {
        (**self).read(path)
    }
    fn write_atomic(&self, path: &Path, data: &[u8]) -> StoreResult<()> {
        (**self).write_atomic(path, data)
    }
    fn create_dir_all(&self, path: &Path) -> StoreResult<()> {
        (**self).create_dir_all(path)
    }
    fn remove_file(&self, path: &Path) -> StoreResult<()> {
        (**self).remove_file(path)
    }
    fn rename(&self, from: &Path, to: &Path) -> StoreResult<()> {
        (**self).rename(from, to)
    }
    fn exists(&self, path: &Path) -> StoreResult<bool> {
        (**self).exists(path)
    }
    fn len(&self, path: &Path) -> StoreResult<u64> {
        (**self).len(path)
    }
    fn list_dir(&self, path: &Path) -> StoreResult<Vec<String>> {
        (**self).list_dir(path)
    }
    fn sync_dir(&self, path: &Path) -> StoreResult<()> {
        (**self).sync_dir(path)
    }
}
