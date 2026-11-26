//! # Error types for `contextra-vector` (B4-compliance).
//!
//! Provides a dedicated, non-exhaustive error enum capturing all error states of the vector index.

use thiserror::Error;

/// Error states of `contextra-vector`.
#[derive(Debug, Error, PartialEq, Clone)]
#[non_exhaustive]
pub enum VectorError {
    /// Invalid input or configuration parameter.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Index structure corruption or inconsistency.
    #[error("index error: {0}")]
    Index(String),

    /// Dimension mismatch during insertion or search.
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch {
        /// Expected vector dimensionality.
        expected: usize,
        /// Actual vector dimensionality provided.
        actual: usize,
    },

    /// HNSW connectivity degradation detected.
    #[error("HNSW connectivity degraded: deleted ratio {deleted_ratio:.4}")]
    ConnectivityDegraded {
        /// Ratio of deleted to total nodes.
        deleted_ratio: f64,
    },

    /// Graph repair failure during node deletion.
    #[error("graph repair failed: {0}")]
    GraphRepairFailed(String),

    /// Concurrency or lock acquisition failure.
    #[error("concurrency error: {0}")]
    Concurrency(String),

    /// I/O or storage persistence error.
    #[error("io error: {0}")]
    Io(String),

    /// Capability unsupported by the index engine.
    #[error("unsupported capability: {0}")]
    Unsupported(String),
}

/// Standard Result type for `contextra-vector`.
pub type Result<T> = std::result::Result<T, VectorError>;

impl From<std::io::Error> for VectorError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

#[cfg(feature = "ports-compat")]
impl From<VectorError> for contextra_types::ContextraError {
    fn from(err: VectorError) -> Self {
        match err {
            VectorError::InvalidInput(msg) => contextra_types::ContextraError::invalid_input(msg),
            VectorError::Index(msg) => contextra_types::ContextraError::Index(msg),
            VectorError::DimensionMismatch { expected, actual } => {
                contextra_types::ContextraError::invalid_input(format!(
                    "dimension mismatch: expected {expected}, got {actual}"
                ))
            }
            VectorError::ConnectivityDegraded { deleted_ratio } => {
                contextra_types::ContextraError::HnswConnectivityDegraded { deleted_ratio }
            }
            VectorError::GraphRepairFailed(_msg) => {
                contextra_types::ContextraError::GraphRepairFailed(
                    contextra_types::HnswDeletionError::VerificationFailed {
                        remaining_pointers: 1,
                    },
                )
            }
            VectorError::Concurrency(msg) => contextra_types::ContextraError::Index(msg),
            VectorError::Io(msg) => contextra_types::ContextraError::Io(std::io::Error::other(msg)),
            VectorError::Unsupported(msg) => {
                contextra_types::ContextraError::capability_unsupported("vector", msg)
            }
        }
    }
}

#[cfg(feature = "ports-compat")]
impl From<contextra_types::ContextraError> for VectorError {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::InvalidInput(msg) => VectorError::InvalidInput(msg),
            contextra_types::ContextraError::Index(msg) => VectorError::Index(msg),
            contextra_types::ContextraError::HnswConnectivityDegraded { deleted_ratio } => {
                VectorError::ConnectivityDegraded { deleted_ratio }
            }
            contextra_types::ContextraError::GraphRepairFailed(e) => {
                VectorError::GraphRepairFailed(e.to_string())
            }
            contextra_types::ContextraError::Io(e) => VectorError::Io(e.to_string()),
            contextra_types::ContextraError::CapabilityUnsupported { capability, reason } => {
                VectorError::Unsupported(format!("{capability}: {reason}"))
            }
            other => VectorError::Index(other.to_string()),
        }
    }
}
