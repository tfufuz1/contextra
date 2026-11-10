//! Crate-lokaler Fehlertyp für `contextra-checkpoint`.

use thiserror::Error;

/// Crate-lokaler Fehlertyp für das Checkpoint-Subsystem.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CheckpointError {
    /// Interner Logikfehler.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Ungültige Eingabeparameter.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Angefragter Checkpoint existiert nicht.
    #[error("Checkpoint not found")]
    CheckpointNotFound,

    /// Transaktionsfehler oder Konflikt.
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Serialisierungs- oder Deserialisierungsfehler.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Fehler beim Hardlinken über Dateisystemgrenzen hinweg.
    #[error("Cross-device link failed: cannot hard link '{source_path}' to '{target_path}' across distinct filesystems/mounts")]
    CrossDeviceLink {
        /// Pfad der Quelldatei.
        source_path: String,
        /// Pfad der Zieldatei.
        target_path: String,
    },

    /// Standard I/O-Fehler.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Mismatch der ID-Breite im Header/Manifest (SPEC §5.2).
    #[error("ID width mismatch: stored {stored} bits, build expects {build} bits")]
    IdWidthMismatch {
        /// Im Format gespeicherte ID-Breite in Bits.
        stored: u16,
        /// Vom aktuellen Build erwartete ID-Breite in Bits.
        build: u16,
    },
}

impl CheckpointError {
    /// Hilfsfunktion zur Konstruktion von `InvalidInput`.
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Hilfsfunktion zur Konstruktion von `CrossDeviceLink`.
    pub fn cross_device_link(source_path: impl Into<String>, target_path: impl Into<String>) -> Self {
        Self::CrossDeviceLink {
            source_path: source_path.into(),
            target_path: target_path.into(),
        }
    }
}

/// Lokales Result-Alias für `contextra-checkpoint`.
pub type Result<T> = std::result::Result<T, CheckpointError>;
