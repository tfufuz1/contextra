// FILE-CONTEXT
// ZWECK: Dateisystem-Naht fuer HNSW-Persistenzpfad (Vorbereitung fuer VFS-Entkopplung).
// INVARIANTEN: Buendelt alle verbleibenden std::fs-Zugriffe des HNSW-Persistenzpfads.
// HOTSPOTS: open_readonly, create_tmp, rename, sync_dir
// STAND: TS:2026-11-12T00:00:00Z

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

/// Oeffnet eine Datei schreibgeschuetzt.
#[inline]
pub(crate) fn open_readonly(path: &Path) -> io::Result<File> {
    File::open(path)
}

/// Erzeugt einen Pfad fuer eine temporaere Datei im selben Verzeichnis wie `destination`.
#[inline]
pub(crate) fn tmp_path_for(destination: &Path) -> PathBuf {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let file_name = destination
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("index.hnsw");
    let pid = std::process::id();
    parent.join(format!(".{file_name}.{pid}.tmp"))
}

/// Erstellt eine temporaere Datei zum Schreiben (truncate/create).
#[inline]
pub(crate) fn create_tmp(tmp_path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(tmp_path)
}

/// Benennt eine Datei atomar um (`from` -> `to`).
#[inline]
pub(crate) fn rename(from: &Path, to: &Path) -> io::Result<()> {
    std::fs::rename(from, to)
}

/// Entfernt eine temporaere Datei (fuer Cleanup bei Fehlern).
#[inline]
pub(crate) fn remove_file(path: &Path) -> io::Result<()> {
    std::fs::remove_file(path)
}

/// Synchronisiert das Elternverzeichnis von `path` auf Platte (Durability nach Rename).
#[inline]
pub(crate) fn sync_dir(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        let dir = File::open(parent)?;
        dir.sync_all()?;
    }
    Ok(())
}
