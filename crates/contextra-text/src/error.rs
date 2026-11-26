//! Fehlerdefinitionen für `contextra-text` (Klasse A / SPEC §4.1).

use thiserror::Error;

/// Typisierter Fehler für Volltextindexierung, Tokenisierung und Scoring.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TextError {
    /// Ungültige Eingabedaten (z. B. leere/ungültige Abfrage, ungültige Scoring-Parameter).
    #[error("ungültige Eingabedaten: {0}")]
    InvalidInput(String),

    /// Fehler bei Textanalyse, Regex oder Tokenisierung.
    #[error("Textverarbeitungsfehler: {0}")]
    Text(String),

    /// Überschreitung von Kapazitäts- oder Längengrenzen (z. B. MAX_TEXT_BYTES).
    #[error("Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}")]
    CapacityExceeded {
        /// Geforderte Größe oder Länge.
        requested: usize,
        /// Maximal zulässige Grenze.
        maximum: usize,
    },

    /// Speicher- oder E/A-Fehler bei Zugriff auf Segmente oder persistente Daten.
    #[error("Speicher- oder E/A-Fehler: {0}")]
    Storage(String),

    /// Serialisierungs- oder Deserialisierungsfehler.
    #[error("Serialisierungsfehler: {0}")]
    Serialization(String),

    /// E/A-Fehler der Standardbibliothek.
    #[error("E/A-Fehler aufgetreten: {0}")]
    Io(#[from] std::io::Error),

    /// Interner Zustandskonflikt oder unerwarteter Ausführungszustand.
    #[error("interner Zustandskonflikt: {0}")]
    Internal(String),
}

/// Einheitlicher Result-Typ für `contextra-text`.
pub type Result<T> = core::result::Result<T, TextError>;

#[cfg(feature = "types-compat")]
impl From<TextError> for contextra_types::ContextraError {
    fn from(err: TextError) -> Self {
        match err {
            TextError::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
            TextError::Text(msg) => contextra_types::ContextraError::Text(msg),
            TextError::CapacityExceeded { requested, maximum } => {
                contextra_types::ContextraError::InvalidInput(format!(
                    "Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}"
                ))
            }
            TextError::Storage(msg) => contextra_types::ContextraError::Storage(msg),
            TextError::Serialization(msg) => contextra_types::ContextraError::Storage(msg),
            TextError::Io(e) => contextra_types::ContextraError::Io(e),
            TextError::Internal(msg) => contextra_types::ContextraError::Internal(msg),
        }
    }
}

#[cfg(feature = "types-compat")]
impl From<contextra_types::ContextraError> for TextError {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::InvalidInput(msg) => TextError::InvalidInput(msg),
            contextra_types::ContextraError::Text(msg) => TextError::Text(msg),
            contextra_types::ContextraError::Storage(msg) => TextError::Storage(msg),
            contextra_types::ContextraError::Io(e) => TextError::Storage(e.to_string()),
            other => TextError::Internal(other.to_string()),
        }
    }
}
