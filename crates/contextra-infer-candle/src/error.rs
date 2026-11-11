//! Kanonische Fehlerarchitektur für `contextra-infer-candle`.

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
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

pub type Result<T> = std::result::Result<T, Error>;
