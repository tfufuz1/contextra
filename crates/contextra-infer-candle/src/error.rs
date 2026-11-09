//! Kanonische Fehlerarchitektur für `contextra-infer-candle`.
//!
//! # Invarianten
//! - Autarker Klasse-A Baustein: Genau ein primärer Fehlertyp [`Error`].
//! - Zero-Panic: Alle Fehlerbedingungen werden typisiert zurückgegeben.

// ANCHOR[contextra-infer-candle:ERROR-001]

/// Primärer Fehlertyp für Inferenz- und Modellierungsoperationen.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// E/A-Fehler beim Zugriff auf Modelldateien oder Tensordaten.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Fehler beim Laden oder Parsen eines Modells (GGUF, Safetensors).
    #[error("Model load error: {reason}")]
    ModelLoad {
        /// Grund des Ladefehlers.
        reason: String,
    },

    /// Fehler während der Tokenisierung oder Dekodierung.
    #[error("Tokenizer error: {0}")]
    Tokenizer(String),

    /// Fehler während der Tensor-Berechnung oder Inferenz.
    #[error("Inference execution error: {0}")]
    Inference(String),

    /// Ungültige oder inkonsistente Parameter.
    #[error("Invalid parameter: {0}")]
    InvalidInput(String),

    /// Ressourcen- oder Budgetüberschreitung (Kontextlänge, Batchgröße, Timeout).
    #[error("Resource limit exceeded: {0}")]
    ResourceExceeded(String),

    /// Validierungs- oder Grounding-Fehler.
    #[error("Grounding policy violation: {0}")]
    PolicyViolation(String),

    /// Backend ist nicht verfügbar oder Feature nicht aktiviert.
    #[error("Backend unavailable: {0}")]
    BackendUnavailable(String),
}

/// Kanonischer Result-Typ für `contextra-infer-candle`.
pub type Result<T> = std::result::Result<T, Error>;
