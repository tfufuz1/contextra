//! Fehler-Abbildung für `contextra-vector` (SPEC-v2 §5.3, compat/).
// Nur aktiv wenn das workspace-Feature (oder das ältere ports-compat-Alias) eingeschaltet ist.
#![cfg(any(feature = "ports-compat", feature = "workspace"))]

use crate::error::VectorError;

impl From<VectorError> for contextra_types::ContextraError {
    fn from(err: VectorError) -> Self {
        match err {
            VectorError::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
            VectorError::Index(msg) => contextra_types::ContextraError::Index(msg),
            VectorError::Storage(msg) => contextra_types::ContextraError::Storage(msg),
            VectorError::DimensionMismatch { expected, actual } => {
                contextra_types::ContextraError::EmbeddingDimensionMismatch {
                    expected,
                    got: actual,
                }
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
            VectorError::NotFound(msg) => contextra_types::ContextraError::NotFound(msg),
            VectorError::Conflict(msg) => contextra_types::ContextraError::Conflict(msg),
            VectorError::Concurrency(msg) => contextra_types::ContextraError::Index(msg),
            VectorError::Io(msg) => contextra_types::ContextraError::Io(std::io::Error::other(msg)),
            VectorError::Unsupported(msg) => {
                contextra_types::ContextraError::capability_unsupported("vector", msg)
            }
            VectorError::Internal(msg) => contextra_types::ContextraError::Internal(msg),
        }
    }
}

impl From<contextra_types::ContextraError> for VectorError {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::InvalidInput(msg) => VectorError::InvalidInput(msg),
            contextra_types::ContextraError::Index(msg) => VectorError::Index(msg),
            contextra_types::ContextraError::Storage(msg) => VectorError::Storage(msg),
            contextra_types::ContextraError::EmbeddingDimensionMismatch { expected, got } => {
                VectorError::DimensionMismatch {
                    expected,
                    actual: got,
                }
            }
            contextra_types::ContextraError::HnswConnectivityDegraded { deleted_ratio } => {
                VectorError::ConnectivityDegraded { deleted_ratio }
            }
            contextra_types::ContextraError::GraphRepairFailed(e) => {
                VectorError::GraphRepairFailed(e.to_string())
            }
            contextra_types::ContextraError::NotFound(msg) => VectorError::NotFound(msg),
            contextra_types::ContextraError::Conflict(msg) => VectorError::Conflict(msg),
            contextra_types::ContextraError::Io(e) => VectorError::Io(e.to_string()),
            contextra_types::ContextraError::CapabilityUnsupported { capability, reason } => {
                VectorError::Unsupported(format!("{capability}: {reason}"))
            }
            contextra_types::ContextraError::Internal(msg) => VectorError::Internal(msg),
            other => VectorError::Index(other.to_string()),
        }
    }
}
