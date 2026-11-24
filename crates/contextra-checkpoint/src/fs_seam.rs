//! # Lokale Dateisystem-Naht für Checkpoint
//!
//! Kapselt Dateisystem-Operationen für Hardlink-Klone und Orphan-State-Persistenz,
//! ohne auf externe VFS-Ports angewiesen zu sein.

use std::fs;
use std::io;
use std::path::Path;

/// Minimale Schnittstelle für Dateisystemzugriffe.
pub trait FsSeam: Send + Sync {
    /// Schreibt Daten atomar oder direkt in eine Datei.
    fn write_file(&self, path: &Path, content: &[u8]) -> io::Result<()>;

    /// Liest den gesamten Inhalt einer Datei.
    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>>;

    /// Prüft, ob ein Pfad existiert.
    fn exists(&self, path: &Path) -> bool;

    /// Löscht eine Datei, falls sie existiert.
    fn remove_file(&self, path: &Path) -> io::Result<()>;

    /// Erzeugt ein Verzeichnis rekursiv.
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
}

/// Standard-Implementierung basierend auf `std::fs`.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdFsSeam;

impl StdFsSeam {
    /// Erzeugt eine neue Instanz von [`StdFsSeam`].
    pub fn new() -> Self {
        Self
    }
}

impl FsSeam for StdFsSeam {
    fn write_file(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        // Write atomic via temp file in same directory
        let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));
        fs::write(&tmp_path, content)?;
        fs::rename(&tmp_path, path)
    }

    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        if path.exists() {
            fs::remove_file(path)
        } else {
            Ok(())
        }
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }
}
