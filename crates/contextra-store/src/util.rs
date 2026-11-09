//! # Hilfswerkzeuge fuer den Store (Ring 1)
//!
//! Stellt universelle Hilfsfunktionen bereit, insbesondere fuer deterministische
//! Hashberechnungen und Byte-Konvertierungen.

// FILE-CONTEXT: Utility functions for contextra-store (fsync helpers, etc.) (TS: 2026-08-29T17:17:31Z) (SESSION: 8f882f1f)
//! Utility functions for storage engine operations.

use contextra_types::{ContextraError, Result};
use std::fmt;
use std::path::Path;

#[cfg(not(loom))]
use crate::lsm::config::DurabilityMode;
#[cfg(not(loom))]
use std::fs::{File, TryLockError};

pub(crate) use crate::fsync::fsync_parent_dir;

/// Kapselung eines versiegelten Datensatzes.
#[derive(Clone, PartialEq, Eq)]
pub struct SealedEnvelope {
    payload: Vec<u8>,
}

impl SealedEnvelope {
    /// Erzeugt einen neuen [`SealedEnvelope`].
    #[must_use]
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }

    /// Liefert die Rohdaten des Umschlags.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

impl fmt::Debug for SealedEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SealedEnvelope([REDACTED, len={}])", self.payload.len())
    }
}

/// Lokaler Trait zur Versiegelung von Datensätzen (Ersatz für externe Krypto-Kopplungen).
pub trait RecordSealer: Send + Sync {
    /// Versiegelt Klartextdaten deterministisch oder mit Umschlag.
    ///
    /// # Errors
    /// Liefert einen Fehler zurück, falls die Versiegelung fehlschlägt.
    fn seal(&self, key: &[u8], plaintext: &[u8]) -> Result<SealedEnvelope>;

    /// Öffnet einen versiegelten Datensatz.
    ///
    /// # Errors
    /// Liefert einen Fehler zurück, falls die Entsiegelung fehlschlägt.
    fn open(&self, key: &[u8], envelope: &[u8]) -> Result<Vec<u8>>;
}

/// Lokaler Trait zur Authentisierung von Datensätzen und Blöcken.
pub trait RecordAuthenticator: Send + Sync {
    /// Berechnet ein Authentisierungs-Tag über gegebene Daten.
    ///
    /// # Errors
    /// Liefert einen Fehler zurück, falls das Tag nicht erzeugt werden kann.
    fn authenticate(&self, key: &[u8], data: &[u8]) -> Result<[u8; 32]>;

    /// Verifiziert ein Authentisierungs-Tag gegen gegebene Daten.
    ///
    /// # Errors
    /// Liefert einen Fehler zurück, falls die Verifikation fehlschlägt.
    fn verify(&self, key: &[u8], data: &[u8], expected_tag: &[u8; 32]) -> Result<bool>;
}

/// Handle for exclusive database directory locking.
#[cfg(not(loom))]
#[derive(Debug)]
// ANCHOR[contextra-store:ROOT-UTIL-001]
/// Handle fuer exklusive Datenverzeichnissperre.
pub(crate) struct DirLock {
    _file: Option<File>,
}

#[cfg(not(loom))]
impl DirLock {
    /// Acquires an exclusive lock on `dir/LOCK`.
    pub(crate) fn acquire(dir: &Path, durability_mode: DurabilityMode) -> Result<Self> {
        let lock_path = dir.join("LOCK");
        let file = File::create(&lock_path).map_err(|e| {
            ContextraError::Storage(format!(
                "Failed to open/create LOCK file at {}: {e}",
                lock_path.display()
            ))
        })?;

        match file.try_lock() {
            Ok(()) => Ok(Self { _file: Some(file) }),
            Err(TryLockError::WouldBlock) => Err(ContextraError::Storage(
                "Datenverzeichnis bereits in Benutzung".into(),
            )),
            Err(TryLockError::Error(e)) => {
                if durability_mode == DurabilityMode::Full {
                    Err(ContextraError::Storage(format!(
                        "Dateisperre auf diesem Dateisystem nicht unterstützt: {e}"
                    )))
                } else {
                    tracing::warn!(
                        "Dateisperre auf diesem Dateisystem nicht unterstützt ({}); fahre ohne Sperre fort: {e}",
                        lock_path.display()
                    );
                    Ok(Self { _file: None })
                }
            }
        }
    }
}
