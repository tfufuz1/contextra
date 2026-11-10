//! # Error types for  (B4-compliance).
//!
//! Provides a dedicated, non-exhaustive error enum capturing all error states of the vector index.

use thiserror::Error;

/// Error states of .
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

impl VectorError {
    /// Helper to construct InvalidInput with String conversion
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }
}

/// Standard Result type for .
pub type Result<T> = std::result::Result<T, VectorError>;

impl From<std::io::Error> for VectorError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}
