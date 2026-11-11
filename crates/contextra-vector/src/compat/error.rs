//! Fehler-Abbildung für contextra-vector (SPEC-v2 §5.3, compat/).

use crate::error::VectorError;

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
                #[allow(deprecated)]
                contextra_types::ContextraError::HnswConnectivityDegraded { deleted_ratio }
            }
            VectorError::GraphRepairFailed(_msg) => {
                #[allow(deprecated)]
                contextra_types::ContextraError::GraphRepairFailed(
                    contextra_types::HnswDeletionError::VerificationFailed {
                        remaining_pointers: 1,
                    },
                )
            }
            VectorError::Concurrency(msg) => contextra_types::ContextraError::Index(msg),
            VectorError::Io(msg) => contextra_types::ContextraError::Io(std::io::Error::other(msg)),
            VectorError::Storage(msg) => contextra_types::ContextraError::Storage(msg),
            VectorError::Conflict(msg) => contextra_types::ContextraError::Conflict(msg),
            VectorError::NotFound(msg) => contextra_types::ContextraError::NotFound(msg),
            VectorError::Internal(msg) => contextra_types::ContextraError::Internal(msg),
            VectorError::Unsupported(msg) => {
                contextra_types::ContextraError::capability_unsupported("vector", msg)
            }
        }
    }
}

impl From<contextra_types::ContextraError> for VectorError {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::InvalidInput(msg) => VectorError::InvalidInput(msg),
            contextra_types::ContextraError::Index(msg) => VectorError::Index(msg),
            #[allow(deprecated)]
            contextra_types::ContextraError::HnswConnectivityDegraded { deleted_ratio } => {
                VectorError::ConnectivityDegraded { deleted_ratio }
            }
            #[allow(deprecated)]
            contextra_types::ContextraError::GraphRepairFailed(e) => {
                VectorError::GraphRepairFailed(e.to_string())
            }
            contextra_types::ContextraError::Io(e) => VectorError::Io(e.to_string()),
            contextra_types::ContextraError::Storage(msg) => VectorError::Storage(msg),
            contextra_types::ContextraError::Conflict(msg) => VectorError::Conflict(msg),
            contextra_types::ContextraError::NotFound(msg) => VectorError::NotFound(msg),
            contextra_types::ContextraError::Internal(msg) => VectorError::Internal(msg),
            contextra_types::ContextraError::CapabilityUnsupported { capability, reason } => {
                VectorError::Unsupported(format!("{capability}: {reason}"))
            }
            other => VectorError::Index(other.to_string()),
        }
    }
}
