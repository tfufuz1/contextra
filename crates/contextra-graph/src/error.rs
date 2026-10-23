//! Graph mutation error taxonomy for Contextra Graph.

pub use contextra_ports::graph::GraphMutationError;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
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

        let contextra_err: contextra_types::ContextraError = err1.into();
        assert!(
            matches!(contextra_err, contextra_types::ContextraError::InvalidInput(msg) if msg.contains("Insufficient participants"))
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
