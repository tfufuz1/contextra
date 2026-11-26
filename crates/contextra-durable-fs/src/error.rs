//! # Fehlertypen für `contextra-durable-fs`
//!
//! Einheitlicher Fehlertyp [`DurableFsError`] / [`Error`] gemäß Klasse-A-Vertrag (B4).

use std::path::PathBuf;
use thiserror::Error;

/// Primärer Fehlertyp für alle dauerhaften Dateisystemoperationen (`contextra-durable-fs`).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DurableFsError {
    /// Dateisystem- oder I/O-Fehler des zugrundeliegenden Betriebssystems.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Pfad wurde nicht gefunden.
    #[error("path not found: {path}")]
    NotFound {
        /// Betroffener Pfad.
        path: PathBuf,
    },

    /// Das Dateihandle wurde aufgrund eines vorangegangenen fsync-Fehlers dauerhaft vergiftet (`INV-FSYNC-1`).
    #[error("handle is poisoned due to previous fsync failure (INV-FSYNC-1)")]
    PoisonedHandle,

    /// Ungültige Eingabe oder Parameterverletzung.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Prüfsummen- oder Korruptionsfehler in gespeicherten Daten.
    #[error("checksum or corruption error: {0}")]
    Corruption(String),

    /// Bereinigung (Scrubbing) aus Sicherheits- oder Plattformgründen verweigert.
    #[error("scrubbing refused: {0}")]
    ScrubRefused(String),

    /// Nicht unterstütztes Dateisystem für Durability/Locking-Garantien.
    #[error("unsupported filesystem: {0}")]
    UnsupportedFilesystem(String),
}

/// Kanonischer Alias `Error` gemäß Klasse-A-Vertrag (B4).
pub type Error = DurableFsError;

impl DurableFsError {
    /// Gibt den entsprechenden [`std::io::ErrorKind`] für Kompatibilität mit IO-Operationen zurück.
    #[must_use]
    pub fn kind(&self) -> std::io::ErrorKind {
        match self {
            Self::Io(e) => e.kind(),
            Self::NotFound { .. } => std::io::ErrorKind::NotFound,
            Self::PoisonedHandle => std::io::ErrorKind::Other,
            Self::InvalidArgument(_) => std::io::ErrorKind::InvalidInput,
            Self::Corruption(_) => std::io::ErrorKind::InvalidData,
            Self::ScrubRefused(_) => std::io::ErrorKind::PermissionDenied,
            Self::UnsupportedFilesystem(_) => std::io::ErrorKind::Unsupported,
        }
    }
}

impl From<DurableFsError> for std::io::Error {
    fn from(err: DurableFsError) -> Self {
        match err {
            DurableFsError::Io(io_err) => io_err,
            other => std::io::Error::new(other.kind(), other.to_string()),
        }
    }
}

/// Einheitlicher Result-Typ für `contextra-durable-fs`.
pub type Result<T> = std::result::Result<T, DurableFsError>;
