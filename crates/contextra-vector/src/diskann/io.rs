// FILE-CONTEXT
// ZWECK: Gebündelte Dateizugriffe und Durability-Primitive für DiskANN (WP-VEC-07 / S-80).
// INVARIANTEN: Strengste Durability-Reihenfolge: tmp sync_all -> rename -> parent dir sync_all; Zero-Panic.
// HOTSPOTS: atomic_replace, append_wal, read_all, remove_if_exists, open_mmap_readonly
// STAND: 2026-11-11

//! E/A- und Durability-Primitive für `DiskANN`.
//!
//! Zentralisiert alle `std::fs`- und Speicherabbildungs-Aufrufe, um
//! I/O-Zugriffe zu entkoppeln und die spätere Auslagerung in ein eigenes Crate vorzubereiten.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use crate::error::{Result, VectorError};

// ANCHOR[contextra-vector:W07-IO-001]
/// Führt ein atomares Ersetzen von `dest` durch `tmp` durch (INV-DISKANN-1).
///
/// Reihenfolge (strengste vorhandene Variante):
/// 1. `tmp_file.sync_all()` zur Garantie vollständiger Dateiinhalte auf Platte
/// 2. `rename(tmp, dest)`
/// 3. Verzeichnis-`sync_all()` auf dem Elternverzeichnis von `dest`
pub(crate) fn atomic_replace(tmp: &Path, dest: &Path) -> Result<()> {
    if tmp.exists() {
        let tmp_file =
            File::open(tmp).map_err(|e| VectorError::Storage(format!("fsync tmp: {e}")))?;
        tmp_file
            .sync_all()
            .map_err(|e| VectorError::Storage(format!("fsync tmp: {e}")))?;
        drop(tmp_file);
    }

    std::fs::rename(tmp, dest).map_err(|e| VectorError::Storage(format!("rename: {e}")))?;

    if let Some(parent) = dest.parent() {
        if let Ok(dir) = File::open(parent) {
            let _ = dir
                .sync_all()
                .map_err(|e| VectorError::Storage(format!("parent fsync: {e}")));
        }
    }

    Ok(())
}

// ANCHOR[contextra-vector:W07-IO-002]
/// Entfernt eine Datei, falls sie existiert.
pub(crate) fn remove_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}

// ANCHOR[contextra-vector:W07-IO-003]
/// Liest den gesamten Inhalt einer Datei als Bytevektor.
pub(crate) fn read_all(path: &Path) -> Result<Vec<u8>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Ok(Vec::new()),
    };
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(VectorError::from)?;
    Ok(buf)
}

// ANCHOR[contextra-vector:W07-IO-004]
/// Schreibt Daten an das Ende einer WAL-Datei, inkl. optionalem Header bei Neuerstellung und Datei-`sync_all`.
pub(crate) fn append_wal(
    path: &Path,
    magic: &[u8],
    version: u8,
    entry_bytes: &[u8],
    hmac_bytes: &[u8],
) -> Result<()> {
    let is_new = match std::fs::metadata(path) {
        Ok(meta) => meta.len() == 0,
        Err(_) => true,
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(VectorError::from)?;

    if is_new {
        file.write_all(magic).map_err(VectorError::from)?;
        file.write_all(&[version]).map_err(VectorError::from)?;
    }

    file.write_all(entry_bytes).map_err(VectorError::from)?;
    file.write_all(hmac_bytes).map_err(VectorError::from)?;
    file.sync_all().map_err(VectorError::from)?;
    Ok(())
}

// ANCHOR[contextra-vector:W07-IO-005]
/// Öffnet eine Datei nur lesend.
pub(crate) fn open_read(path: &Path) -> Result<File> {
    File::open(path).map_err(VectorError::from)
}

// ANCHOR[contextra-vector:W07-IO-006]
/// Erstellt eine Datei neu oder schneidet sie ab zum Schreiben.
pub(crate) fn create_truncate(path: &Path) -> Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(VectorError::from)
}

// ANCHOR[contextra-vector:W07-IO-007]
/// Öffnet ein sicheres Nur-Lese-Memory-Mapping über `contextra_sys::mmap_readonly`.
pub(crate) fn open_mmap_readonly(file: &File) -> Result<memmap2::Mmap> {
    contextra_sys::mmap_readonly(file).map_err(VectorError::from)
}

// ANCHOR[contextra-vector:W07-IO-008]
/// Schreibt Bytes in eine Datei (nur für Initialisierungs-/Testzwecke).
pub(crate) fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = create_truncate(path)?;
    file.write_all(bytes).map_err(VectorError::from)?;
    file.sync_all().map_err(VectorError::from)?;
    Ok(())
}
