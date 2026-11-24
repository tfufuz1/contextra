//! # Lokaler Fehlertyp für contextra-checkpoint
//!
//! Bildet alle Fehlerfälle der Checkpoint-Registry und RAII-Guards typisiert ab,
//! ohne harte Kopplung an externe Fehlertypen.

use thiserror::Error;

/// Typalias für Resultate mit [`CheckpointError`].
pub type CheckpointResult<T> = std::result::Result<T, CheckpointError>;

/// Kurzform für interne Module, identisch zu [`CheckpointResult`].
pub type Result<T> = CheckpointResult<T>;

/// Fehlerzustände im Checkpoint-Subsystem.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CheckpointError {
    /// Ungültige Eingabeparameter (z. B. leere Namen oder Grenzwertüberschreitungen).
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Serialisierungs- oder Deserialisierungsfehler (JSON/BLAKE3).
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Fehler während Transaktions-Rollback, -Commit oder Monotonie-Verletzungen.
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Der angeforderte Checkpoint existiert nicht.
    #[error("Checkpoint not found")]
    CheckpointNotFound,

    /// Cross-Device Link Fehler beim Hardlinking über Mount-Punkte hinweg.
    #[error("Cross-device link failed: cannot hard link '{source_path}' to '{target_path}' across distinct filesystems/mounts")]
    CrossDeviceLink {
        /// Quellpfad der Datei.
        source_path: String,
        /// Zielpfad des Hardlinks.
        target_path: String,
    },

    /// I/O-Fehler beim Dateizugriff.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Gespeicherte ID-Breite passt nicht zur Build-Konfiguration (SPEC §5.2).
    #[error("ID width mismatch: stored {stored} bits, build {build} bits")]
    IdWidthMismatch {
        /// Im Checkpoint gespeicherte Breite.
        stored: u16,
        /// Breite dieses Builds.
        build: u16,
    },

    /// Interner System- oder Synchronisationsfehler.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl CheckpointError {
    /// Erzeugt einen `InvalidInput`-Fehler.
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Erzeugt einen `Internal`-Fehler.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Erzeugt einen `CrossDeviceLink`-Fehler.
    pub fn cross_device_link(
        source_path: impl Into<String>,
        target_path: impl Into<String>,
    ) -> Self {
        Self::CrossDeviceLink {
            source_path: source_path.into(),
            target_path: target_path.into(),
        }
    }
}
