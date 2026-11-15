// FILE-CONTEXT
// ZWECK: Atomares Schreiben von Persistenz-Dateien (temp -> sync_all -> rename -> dir sync).
// INVARIANTEN: INV-FSYNC-1 (Fail-closed durability ohne ungesicherte Fehler, Cleanup bei Abbruch).
// HOTSPOTS: write_atomic, write_atomic_with
// STAND: TS:2026-11-12T00:00:00Z

use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;

use super::io as persistence_io;

/// Fuehrt einen Schreibvorgang atomar ueber eine temporaere Datei aus.
///
/// 1. Erstellt temporaere Datei im selben Verzeichnis (PID-basiert).
/// 2. Uebergibt das geoeffnete `File`-Handle an die Callback-Funktion `write_fn`.
/// 3. Synchronisiert Datei-Inhalt und Metadaten (`sync_all`).
/// 4. Benennt temporaere Datei atomar auf `destination` um.
/// 5. Synchronisiert das Elternverzeichnis via `sync_dir` (INV-FSYNC-1).
/// 6. Raeumt bei jedem Fehler die temporaere Datei auf.
pub(crate) fn write_atomic_with<F>(destination: &Path, write_fn: F) -> Result<()>
where
    F: FnOnce(&mut File) -> Result<()>,
{
    let tmp_path = persistence_io::tmp_path_for(destination);
    let mut tmp_file = persistence_io::create_tmp(&tmp_path)?;

    let write_res = (|| -> Result<()> {
        write_fn(&mut tmp_file)?;
        tmp_file.sync_all()?;
        Ok(())
    })();

    if let Err(e) = write_res {
        let _ = persistence_io::remove_file(&tmp_path);
        return Err(e);
    }

    // Explizit Handle schliessen vor Rename
    drop(tmp_file);

    if let Err(e) = persistence_io::rename(&tmp_path, destination) {
        let _ = persistence_io::remove_file(&tmp_path);
        return Err(e);
    }

    if let Err(e) = persistence_io::sync_dir(destination) {
        return Err(e);
    }

    Ok(())
}

/// Writes data to `destination` atomically via a temporary file in the same parent directory,
/// ensuring both file data and parent directory entry are durably synchronized to disk.
pub fn write_atomic(destination: &Path, data: &[u8]) -> Result<()> {
    write_atomic_with(destination, |file| file.write_all(data))
}
