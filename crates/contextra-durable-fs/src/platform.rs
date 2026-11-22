//! # Plattformspezifische synchrone Durability-Flushes
//!
//! Reines synchrones safe Rust über `std::fs::File`.
//! Verzeichnis-fsync via `File::open(dir)?.sync_all()` unter Unix, no-op unter Windows.

use std::fs::File;
use std::io;
use std::path::Path;

/// Spezifikation des Flush-Umfangs für geöffnete Dateihandles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlushKind {
    /// Flusht nur die Dateinutzdaten auf stabile Speichermedien (`fdatasync`).
    DataOnly,
    /// Flusht Dateinutzdaten und Metadaten auf stabile Speichermedien (`fsync`).
    DataAndMetadata,
}

/// Flusht eine geöffnete Datei synchron und crash-sicher auf das Speichermedium.
///
/// # Errors
/// Liefert einen [`io::Error`], falls der Betriebssystem-Flush fehlschlägt.
pub fn sync_file(file: &File, kind: FlushKind) -> io::Result<()> {
    match kind {
        FlushKind::DataOnly => file.sync_data(),
        FlushKind::DataAndMetadata => file.sync_all(),
    }
}

/// Flusht Verzeichniseinträge synchron auf stabile Speichermedien.
///
/// Unter Unix wird das Verzeichnis als Datei geöffnet und `sync_all` ausgeführt.
/// Unter Windows werden Verzeichnis-Metadaten von NTFS automatisch journalisiert.
///
/// # Errors
/// Liefert einen [`io::Error`], falls das Verzeichnis nicht geöffnet oder gesynct werden kann.
pub fn sync_dir_path(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        let file = File::open(dir)?;
        file.sync_all()
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
        Ok(())
    }
}
