//! # Typzustand für Dauerhaftigkeit (M-5)
//!
//! SPEC §8 und IMPL M-5: Synced<T> garantiert, dass zugehörige Daten
//! durch sync_all und sync_dir (oder write_atomic) dauerhaft persistiert wurden,
//! bevor der Katalog-Commit erfolgt.

use contextra_storage_ports::{File, Fs};
use std::io;
use std::ops::Deref;
use std::path::Path;

/// Ein durch Synchronisation dauerhaft gesicherter Wert (M-5).
///
/// Kann von außen nicht direkt konstruiert werden; die Konstruktion
/// erzwingt die Ausführung von `sync_all` und `sync_dir` oder `write_atomic`.
#[derive(Debug, PartialEq, Eq)]
pub struct Synced<T> {
    inner: T,
}

impl<T> Synced<T> {
    /// Gibt den inneren Wert frei.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Referenz auf den inneren Wert.
    pub fn get(&self) -> &T {
        &self.inner
    }
}

impl<T> Deref for Synced<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Erzeugt ein `Synced<T>` nach erfolgreichem `sync_all` auf der Datei und `sync_dir` auf dem Verzeichnis.
pub fn sync_existing<T>(
    fs: &dyn Fs,
    file: &dyn File,
    parent_dir: &Path,
    val: T,
) -> io::Result<Synced<T>> {
    file.sync_all()?;
    fs.sync_dir(parent_dir)?;
    Ok(Synced { inner: val })
}

/// Erzeugt ein `Synced<T>` über atomares Schreiben mittels `write_atomic` aus P06.
pub fn write_atomic_synced<T>(
    fs: &dyn Fs,
    path: &Path,
    data: &[u8],
    val: T,
) -> io::Result<Synced<T>> {
    crate::env::write_atomic(fs, path, data)?;
    Ok(Synced { inner: val })
}
