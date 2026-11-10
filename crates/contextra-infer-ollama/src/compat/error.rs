//! Fehlertransformation und Brückenkonvertierungen zwischen OllamaError und Contextra-Typen.
//! SPEC-v2 §6.2: Bitgleiche, table-driven Abbildung nach ContextraError in compat/error.rs.

use crate::error::OllamaError;

impl From<OllamaError> for contextra_types::ContextraError {
    fn from(err: OllamaError) -> Self {
        match err {
            OllamaError::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
            OllamaError::ModelNotFound(msg) => contextra_types::ContextraError::NotFound(msg),
            OllamaError::Network(msg) => contextra_types::ContextraError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                msg,
            )),
            OllamaError::Timeout(msg) => contextra_types::ContextraError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                msg,
            )),
            OllamaError::Server(msg) => contextra_types::ContextraError::Storage(msg),
            OllamaError::Deserialization(msg) => contextra_types::ContextraError::Internal(msg),
            OllamaError::EgressViolation(msg) => {
                contextra_types::ContextraError::PolicyViolation(msg)
            }
            OllamaError::CircuitBreakerOpen => {
                contextra_types::ContextraError::Internal("circuit breaker open".to_string())
            }
            OllamaError::Internal(msg) => contextra_types::ContextraError::Internal(msg),
        }
    }
}

impl From<contextra_types::ContextraError> for OllamaError {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::InvalidInput(msg) => OllamaError::InvalidInput(msg),
            contextra_types::ContextraError::NotFound(msg) => OllamaError::ModelNotFound(msg),
            contextra_types::ContextraError::PolicyViolation(msg) => {
                OllamaError::EgressViolation(msg)
            }
            contextra_types::ContextraError::Io(io_err) => {
                if io_err.kind() == std::io::ErrorKind::TimedOut {
                    OllamaError::Timeout(io_err.to_string())
                } else {
                    OllamaError::Network(io_err.to_string())
                }
            }
            contextra_types::ContextraError::Storage(msg) => OllamaError::Server(msg),
            contextra_types::ContextraError::Internal(msg) => OllamaError::Internal(msg),
            other => OllamaError::Internal(other.to_string()),
        }
    }
}

impl From<OllamaError> for contextra_ports::EmbeddingError {
    fn from(err: OllamaError) -> Self {
        match err {
            OllamaError::ModelNotFound(msg) | OllamaError::InvalidInput(msg) => {
                contextra_ports::EmbeddingError::Unavailable(msg)
            }
            other => contextra_ports::EmbeddingError::ComputationFailed(other.to_string()),
        }
    }
}
