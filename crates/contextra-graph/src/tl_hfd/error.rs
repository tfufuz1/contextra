//! Error types for Thresholded Local Hyper-Flow Diffusion (TL-HFD).

use contextra_types::ContextraError;
use thiserror::Error;

/// Error type for TL-HFD operations and parameter validations.
#[derive(Error, Debug)]
pub enum TlHfdError {
    /// Contextra core error.
    #[error("Contextra error: {0}")]
    Contextra(#[from] ContextraError),

    /// Invalid parameter value provided to TL-HFD.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Non-finite value encountered during diffusion calculation.
    #[error("Non-finite value encountered during diffusion calculation")]
    NonFinite,
}

impl From<TlHfdError> for ContextraError {
    fn from(err: TlHfdError) -> Self {
        match err {
            TlHfdError::Contextra(e) => e,
            other => ContextraError::InvalidInput(other.to_string()),
        }
    }
}
