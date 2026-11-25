//! Graph mutation error taxonomy for Contextra Graph.

use contextra_types::ContextraError;
use thiserror::Error;

/// Error variants for graph mutation operations.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum GraphMutationError {
    /// Lock acquisition timed out during concurrent mutation.
    #[error("Lock acquisition timed out: {0}")]
    LockAcquisitionTimeout(String),

    /// Provided role binding is invalid.
    #[error("Invalid role binding: {0}")]
    RoleBindingInvalid(String),

    /// Epoch reclamation is pending for the specified epoch.
    #[error("Epoch reclamation pending for epoch: {0}")]
    EpochReclamationPending(u64),

    /// Hyperedge does not meet minimum participant count requirements.
    #[error(
        "Insufficient participants for hyperedge: expected at least {expected}, found {found}"
    )]
    InsufficientParticipants {
        /// Expected minimum participant count.
        expected: usize,
        /// Actual participant count provided.
        found: usize,
    },

    /// Specified hyperedge ID was not found.
    #[error("Hyperedge not found: {0}")]
    HyperedgeNotFound(u64),

    /// Hyperedge ID collision detected during batch commit.
    #[error("Duplicate hyperedge ID detected during batch commit: {0}")]
    DuplicateHyperEdgeId(crate::hyperedge::HyperEdgeId),

    /// Invalid hyperedge weight.
    #[error("Invalid hyperedge weight {weight}: {reason}")]
    InvalidWeight {
        /// The invalid weight value.
        weight: f32,
        /// Reason describing why the weight is invalid.
        reason: String,
    },

    /// Internal error during graph mutation.
    #[error("Internal graph mutation error: {0}")]
    Internal(String),
}

impl From<GraphMutationError> for ContextraError {
    fn from(err: GraphMutationError) -> Self {
        ContextraError::InvalidInput(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_mutation_error_display_and_conversion() {
        let err1 = GraphMutationError::InsufficientParticipants {
            expected: 2,
            found: 1,
        };
        assert_eq!(
            err1.to_string(),
            "Insufficient participants for hyperedge: expected at least 2, found 1"
        );

        let contextra_err: ContextraError = err1.into();
        assert!(
            matches!(contextra_err, ContextraError::InvalidInput(msg) if msg.contains("Insufficient participants"))
        );

        let err2 = GraphMutationError::LockAcquisitionTimeout("test_lock".into());
        assert_eq!(err2.to_string(), "Lock acquisition timed out: test_lock");

        let err4 = GraphMutationError::RoleBindingInvalid("invalid_role".into());
        assert_eq!(err4.to_string(), "Invalid role binding: invalid_role");

        let err5 = GraphMutationError::EpochReclamationPending(7);
        assert_eq!(err5.to_string(), "Epoch reclamation pending for epoch: 7");

        let err6 = GraphMutationError::HyperedgeNotFound(100);
        assert_eq!(err6.to_string(), "Hyperedge not found: 100");

        let err7 = GraphMutationError::InvalidWeight {
            weight: -1.0,
            reason: "must be non-negative".into(),
        };
        assert_eq!(
            err7.to_string(),
            "Invalid hyperedge weight -1: must be non-negative"
        );

        let err8 = GraphMutationError::Internal("db panic".into());
        assert_eq!(err8.to_string(), "Internal graph mutation error: db panic");

        let err9 = GraphMutationError::DuplicateHyperEdgeId(crate::hyperedge::HyperEdgeId::new(42));
        assert_eq!(
            err9.to_string(),
            "Duplicate hyperedge ID detected during batch commit: HyperEdgeId(42)"
        );
    }
}
