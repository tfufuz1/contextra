//! Lokaler Storage-Port für persistente Ledger-Backends (WP-8, SPEC §11.8).

use super::error::LedgerError;
use parking_lot::RwLock;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Abstrakter synchroner Speicher-Port für Append-Only Ledgers.
pub trait LedgerStorage: Send + Sync {
    /// Prüft, ob das Ledger bereits existiert.
    fn exists(&self) -> bool;

    /// Prüft, ob der Speicher leer ist.
    fn is_empty(&self) -> Result<bool, LedgerError> {
        Ok(self.len()? == 0)
    }

    /// Liest den gesamten Speicherinhalt.
    fn read_all(&self) -> Result<Vec<u8>, LedgerError>;

    /// Hängt Daten atomar an und garantiert, dass bei einem Fehler vor dem Sync keine Teil-Daten verbleiben.
    fn append_atomic(&self, data: &[u8]) -> Result<(), LedgerError>;

    /// Synchronisiert die geschriebenen Daten dauerhaft auf das Speichermedium (fsync).
    fn sync(&self) -> Result<(), LedgerError>;

    /// Setzt den Speicherinhalt zurück oder kürzt ihn (z. B. für Rollback oder Prune).
    fn truncate(&self, len: usize) -> Result<(), LedgerError>;

    /// Liefert die aktuelle Bytegröße des Speichers.
    fn len(&self) -> Result<usize, LedgerError>;
}

/// In-Memory-Implementierung von `LedgerStorage` für Unit-Tests und In-Memory-Szenarien.
#[derive(Debug, Clone, Default)]
pub struct MemLedgerStorage {
    data: Arc<RwLock<Vec<u8>>>,
}

impl MemLedgerStorage {
    /// Erstellt einen leeren In-Memory-Speicher.
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Erstellt einen In-Memory-Speicher mit initialen Bytes.
    pub fn with_bytes(bytes: Vec<u8>) -> Self {
        Self {
            data: Arc::new(RwLock::new(bytes)),
        }
    }
}

impl LedgerStorage for MemLedgerStorage {
    fn exists(&self) -> bool {
        !self.data.read().is_empty()
    }

    fn read_all(&self) -> Result<Vec<u8>, LedgerError> {
        Ok(self.data.read().clone())
    }

    fn append_atomic(&self, data: &[u8]) -> Result<(), LedgerError> {
        let mut lock = self.data.write();
        lock.extend_from_slice(data);
        Ok(())
    }

    fn sync(&self) -> Result<(), LedgerError> {
        Ok(())
    }

    fn truncate(&self, len: usize) -> Result<(), LedgerError> {
        let mut lock = self.data.write();
        lock.truncate(len);
        Ok(())
    }

    fn len(&self) -> Result<usize, LedgerError> {
        Ok(self.data.read().len())
    }
}

/// Dateibasierte Implementierung von `LedgerStorage` im sicheren Dateisystem.
pub struct FileLedgerStorage {
    path: PathBuf,
}

impl FileLedgerStorage {
    /// Erstellt einen neuen dateibasierten Storage am angegebenen Pfad.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Gibt den Dateipfad zurück.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl LedgerStorage for FileLedgerStorage {
    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn read_all(&self) -> Result<Vec<u8>, LedgerError> {
        if !self.exists() {
            return Err(LedgerError::NotFound);
        }
        let mut file = File::open(&self.path).map_err(LedgerError::from)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(LedgerError::from)?;
        Ok(buf)
    }

    fn append_atomic(&self, data: &[u8]) -> Result<(), LedgerError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(LedgerError::from)?;
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&self.path)
            .map_err(LedgerError::from)?;

        let initial_len = file.metadata().map_err(LedgerError::from)?.len();

        if let Err(e) = file.write_all(data) {
            let _ = file.set_len(initial_len);
            return Err(LedgerError::from(e));
        }

        Ok(())
    }

    fn sync(&self) -> Result<(), LedgerError> {
        if !self.exists() {
            return Ok(());
        }
        let file = OpenOptions::new()
            .write(true)
            .open(&self.path)
            .map_err(LedgerError::from)?;
        file.sync_all().map_err(LedgerError::from)?;

        if let Some(parent) = self.path.parent() {
            if let Ok(parent_file) = File::open(parent) {
                let _ = parent_file.sync_all();
            }
        }
        Ok(())
    }

    fn truncate(&self, len: usize) -> Result<(), LedgerError> {
        if !self.exists() {
            return Ok(());
        }
        let file = OpenOptions::new()
            .write(true)
            .open(&self.path)
            .map_err(LedgerError::from)?;
        file.set_len(len as u64).map_err(LedgerError::from)?;
        file.sync_all().map_err(LedgerError::from)?;
        Ok(())
    }

    fn len(&self) -> Result<usize, LedgerError> {
        if !self.exists() {
            return Ok(0);
        }
        let meta = std::fs::metadata(&self.path).map_err(LedgerError::from)?;
        Ok(usize::try_from(meta.len()).unwrap_or(usize::MAX))
    }
}
