//! # Atomares Schreiben von Dateien
//!
//! Implementiert `write_atomic` nach SPEC §8 und §9.2:
//! Temp `.building` -> `sync_all` -> `rename` -> `sync_dir`.

use contextra_storage_ports::{Fs, OpenMode, Perm};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static ATOMIC_WRITE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Schreibt eine Datei atomar über eine temporäre Datei mit `.building`-Präfix.
pub fn write_atomic(fs: &dyn Fs, path: &Path, data: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let counter = ATOMIC_WRITE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp_name = format!(".building.{file_name}.{counter}");
    let temp_path: PathBuf = parent.join(temp_name);

    // 1. Temp-Datei anlegen
    let file = fs.open(&temp_path, OpenMode::CreateNew(Perm::Private))?;

    // 2. Daten schreiben und synchronisieren
    let write_res = (|| -> io::Result<()> {
        file.write_all_at(data, 0)?;
        file.sync_all()?;
        Ok(())
    })();

    drop(file);

    if let Err(e) = write_res {
        let _ = fs.remove_file(&temp_path);
        return Err(e);
    }

    // 3. Atomar umbenennen
    if let Err(e) = fs.rename(&temp_path, path) {
        let _ = fs.remove_file(&temp_path);
        return Err(e);
    }

    // 4. Verzeichnis synchronisieren
    fs.sync_dir(parent)?;

    Ok(())
}
