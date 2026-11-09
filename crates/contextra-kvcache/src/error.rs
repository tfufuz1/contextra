//! Typisierte Fehler für `contextra-kvcache`.

use thiserror::Error;

/// Zentraler Fehlertyp für alle KV-Cache-Operationen.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum KvCacheError {
    /// Ungültiger Mandanten-Identifier.
    #[error("Ungültiger Mandant: {0}")]
    InvalidTenant(String),

    /// Ungültige Segment- oder Schlüsseldaten.
    #[error("Ungültiges Segment: {0}")]
    InvalidSegment(String),

    /// Das angeforderte Segment existiert nicht.
    #[error("Segment nicht gefunden: {0}")]
    SegmentNotFound(u64),

    /// Fehler bei kryptografischen Operationen.
    #[error("Kryptografie-Fehler: {0}")]
    CryptoError(String),

    /// Der Segmentschlüssel wurde neutral zerstört / ungültig gemacht.
    #[error("Segmentschlüssel ist zerstört")]
    SegmentKeyDestroyed,

    /// Kapazität oder Budget überschritten.
    #[error("Budget oder Kapazität überschritten: {0}")]
    BudgetExceeded(String),

    /// Interner oder Speicherfehler.
    #[error("Interner Cache-Fehler: {0}")]
    Internal(String),

    /// Fehlerhafte Konfiguration.
    #[error("Konfigurationsfehler: {0}")]
    ConfigError(String),

    /// Quantisierungsfehler.
    #[error("Quantisierungsfehler: {0}")]
    QuantizationError(String),

    /// IO-Fehler.
    #[error("IO-Fehler: {0}")]
    IoError(String),
}

impl From<std::io::Error> for KvCacheError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

/// Standard-Ergebnistyp für `contextra-kvcache`.
pub type Result<T> = std::result::Result<T, KvCacheError>;
