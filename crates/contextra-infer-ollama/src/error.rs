//! Fehlerdefinitionen für `contextra-infer-ollama`.
//!
//! Entspricht B4 (einheitliches Fehlermodell, `#[non_exhaustive]`, `thiserror`).

/// Kanonischer Ergebnistyp für Operationen in `contextra-infer-ollama`.
pub type Result<T> = std::result::Result<T, OllamaError>;

/// Einheitlicher Fehlertyp für Ollama-Inferenz, Anfragen-Validierung und Adapter.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OllamaError {
    /// Ungültige Eingabedaten (z. B. leere Prompts, unzulässige Modellbezeichner oder Batch-Grenzen).
    #[error("ungültige Eingabedaten: {0}")]
    InvalidInput(String),

    /// Ressource oder Modell nicht gefunden.
    #[error("nicht gefunden: {0}")]
    NotFound(String),

    /// Zeitüberschreitung bei HTTP- oder Berechnungsoperation.
    #[error("Zeitüberschreitung: {0}")]
    Timeout(String),

    /// Verbindungsaufbau verweigert oder Gegenstelle nicht erreichbar.
    #[error("Verbindungsfehler: {0}")]
    ConnectionRefused(String),

    /// HTTP- oder Netzwerkfehler bei der Übertragung.
    #[error("HTTP-Übertragungsfehler: {0}")]
    Http(String),

    /// Speicherebenen- oder persistenter Übertragungsfehler.
    #[error("Speicherfehler: {0}")]
    Storage(String),

    /// Serialisierungs- oder Deserialisierungsfehler (z. B. fehlerhaftes JSON / XML).
    #[error("Serialisierungsfehler: {0}")]
    Serialization(String),

    /// Berechnungsfehler oder Dimension-Mismatch (z. B. Embedding-Größe weicht ab).
    #[error("Dimensions- oder Berechnungsfehler: {0}")]
    DimensionMismatch(String),

    /// Interner Verarbeitungsfehler oder Circuit-Breaker ausgelöst.
    #[error("interner Fehler: {0}")]
    Internal(String),

    /// E/A-Fehler.
    #[error("E/A-Fehler: {0}")]
    Io(#[from] std::io::Error),
}

impl OllamaError {
    /// Prüft, ob ein Fehler transienter Natur ist (z. B. Timeout, 5xx, Verbindungsausfall).
    pub fn is_transient(&self) -> bool {
        match self {
            Self::Timeout(_) | Self::ConnectionRefused(_) | Self::Io(_) => true,
            Self::Http(msg) | Self::Internal(msg) => {
                let l = msg.to_lowercase();
                l.contains("503")
                    || l.contains("500")
                    || l.contains("502")
                    || l.contains("504")
                    || l.contains("timeout")
                    || l.contains("connect")
                    || l.contains("connection reset")
                    || l.contains("broken pipe")
            }
            _ => false,
        }
    }
}

#[cfg(feature = "ports-compat")]
impl From<OllamaError> for contextra_types::ContextraError {
    fn from(err: OllamaError) -> Self {
        match err {
            OllamaError::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
            OllamaError::NotFound(msg) => contextra_types::ContextraError::NotFound(msg),
            OllamaError::Timeout(msg) => contextra_types::ContextraError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                msg,
            )),
            OllamaError::ConnectionRefused(msg) => contextra_types::ContextraError::Io(
                std::io::Error::new(std::io::ErrorKind::ConnectionRefused, msg),
            ),
            OllamaError::Http(msg) | OllamaError::Storage(msg) => {
                contextra_types::ContextraError::Storage(msg)
            }
            OllamaError::Serialization(msg) => contextra_types::ContextraError::Serialization(msg),
            OllamaError::DimensionMismatch(msg) => contextra_types::ContextraError::Internal(msg),
            OllamaError::Internal(msg) => contextra_types::ContextraError::Internal(msg),
            OllamaError::Io(e) => contextra_types::ContextraError::Io(e),
        }
    }
}
