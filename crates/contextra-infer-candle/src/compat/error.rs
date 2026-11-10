//! Fehlertransformation und Brückenkonvertierungen zwischen Candle Error und ContextraError.

use crate::error::Error;

impl From<Error> for contextra_types::ContextraError {
    fn from(err: Error) -> Self {
        match err {
            Error::Io(io_err) => contextra_types::ContextraError::Io(io_err),
            Error::ModelLoad { reason } => contextra_types::ContextraError::Internal(reason),
            Error::Tokenizer(msg) => contextra_types::ContextraError::Internal(msg),
            Error::Inference(msg) => contextra_types::ContextraError::Internal(msg),
            Error::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
            Error::ResourceExceeded(msg) => contextra_types::ContextraError::limit_exceeded(0, msg),
            Error::PolicyViolation(msg) => contextra_types::ContextraError::PolicyViolation(msg),
            Error::BackendUnavailable(msg) => contextra_types::ContextraError::NotFound(msg),
        }
    }
}

impl From<contextra_types::ContextraError> for Error {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::Io(io_err) => Error::Io(io_err),
            contextra_types::ContextraError::InvalidInput(msg) => Error::InvalidInput(msg),
            contextra_types::ContextraError::PolicyViolation(msg) => Error::PolicyViolation(msg),
            contextra_types::ContextraError::NotFound(msg) => Error::BackendUnavailable(msg),
            other => Error::Inference(other.to_string()),
        }
    }
}
