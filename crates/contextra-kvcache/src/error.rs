//! Typisierte Fehler für `contextra-kvcache`.

use thiserror::Error;

/// Zentraler Fehlertyp für alle KV-Cache-Operationen.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum KvCacheError {
    /// Ungültiger Mandanten-Identifier.
    #[error("Ungültiger Mandant: {0}")]
    InvalidTenant(String),

    /// Ungültige Eingabedaten.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Ungültige Segment- oder Schlüsseldaten.
    #[error("Ungültiges Segment: {0}")]
    InvalidSegment(String),

    /// Das angeforderte Segment existiert nicht.
    #[error("Segment nicht gefunden: {0}")]
    SegmentNotFound(u64),

    /// Fehler bei kryptografischen Operationen.
    #[error("Crypto error: {0}")]
    CryptoError(String),

    /// Der Segmentschlüssel wurde neutral zerstört / ungültig gemacht.
    #[error("Segmentschlüssel ist zerstört")]
    SegmentKeyDestroyed,

    /// Kapazität oder Budget überschritten.
    #[error("Budget oder Kapazität überschritten: {0}")]
    BudgetExceeded(String),

    /// Pin-Budget überschritten.
    #[error("Pin budget exceeded: {0}")]
    PinBudgetExceeded(String),

    /// Kapazitätsgrenze überschritten.
    #[error("Limit exceeded: {context} (limit: {limit})")]
    LimitExceeded {
        /// Konfigurierte Grenze.
        limit: usize,
        /// Kontextbeschreibung.
        context: String,
    },

    /// Regel- oder Richtlinienverletzung.
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Serialisierungs- oder Deserialisierungsfehler.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Interner oder Speicherfehler.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Fehlerhafte Konfiguration.
    #[error("Konfigurationsfehler: {0}")]
    ConfigError(String),

    /// Quantisierungsfehler.
    #[error("KV quantization error: {0}")]
    QuantizationError(String),

    /// IO-Fehler.
    #[error("IO error: {0}")]
    IoError(#[source] std::io::Error),
}

impl KvCacheError {
    /// Hilfskonstruktor für PinBudgetExceeded.
    pub fn pin_budget_exceeded(msg: impl Into<String>) -> Self {
        Self::PinBudgetExceeded(msg.into())
    }
}

impl From<std::io::Error> for KvCacheError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

/// Standard-Ergebnistyp für `contextra-kvcache`.
pub type Result<T> = std::result::Result<T, KvCacheError>;
