//! Kanonische Fehlerarchitektur für `contextra-infer-candle`.
//!
//! # Invarianten
//! - Autarker Klasse-A Baustein: Genau ein primärer Fehlertyp [`CandleError`].
//! - Zero-Panic: Alle Fehlerbedingungen werden typisiert zurückgegeben.

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CandleError {
    /// E/A-Fehler beim Zugriff auf Modelldateien oder Tensordaten.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Model load error: {reason}")]
    ModelLoad {
        reason: String,
    },

    #[error("Tokenizer error: {0}")]
    Tokenizer(String),

    #[error("Inference execution error: {0}")]
    Inference(String),

    #[error("Invalid parameter: {0}")]
    InvalidInput(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceExceeded(String),

    #[error("Grounding policy violation: {0}")]
    PolicyViolation(String),

    #[error("Backend unavailable: {0}")]
    BackendUnavailable(String),
}

/// Typ-Alias zur Abwärtskompatibilität innerhalb des Crates und für externe Aufrufer.
pub type Error = CandleError;

/// Kanonischer Result-Typ für `contextra-infer-candle`.
pub type Result<T> = std::result::Result<T, CandleError>;
