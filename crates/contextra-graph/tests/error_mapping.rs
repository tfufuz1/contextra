//! Integration test for error mapping between GraphError and ContextraError (GR6.2).

use contextra_graph::error::{GraphError, GraphMutationError};
use contextra_types::ContextraError;

#[test]
fn test_all_graph_error_variants_map_to_contextra_error() {
    let variants: Vec<(GraphError, &str)> = vec![
        (
            GraphError::InvalidInput("param".into()),
            "Invalid input: param",
        ),
        (GraphError::NotFound("id".into()), "Not found: id"),
        (
            GraphError::PolicyViolation("pol".into()),
            "Policy violation: pol",
        ),
        (
            GraphError::Serialization("ser".into()),
            "Serialization error: ser",
        ),
        (GraphError::Storage("io".into()), "Storage error: io"),
        (
            GraphError::BudgetExhausted("mem".into()),
            "Internal error: Budget exhausted: mem",
        ),
        (GraphError::Internal("fail".into()), "Internal error: fail"),
        (
            GraphError::Mutation(GraphMutationError::LockAcquisitionTimeout("lock".into())),
            "Transaction error: Lock acquisition timed out: lock",
        ),
        (
            GraphError::Mutation(GraphMutationError::RoleBindingInvalid("role".into())),
            "Invalid input: Invalid role binding: role",
        ),
        (
            GraphError::Mutation(GraphMutationError::EpochReclamationPending(42)),
            "Conflict: Epoch reclamation pending for epoch: 42",
        ),
        (
            GraphError::Mutation(GraphMutationError::InsufficientParticipants {
                expected: 3,
                found: 1,
            }),
            "Invalid input: Insufficient participants for hyperedge: expected at least 3, found 1",
        ),
        (
            GraphError::Mutation(GraphMutationError::HyperedgeNotFound(99)),
            "Not found: Hyperedge not found: 99",
        ),
        (
            GraphError::Mutation(GraphMutationError::InvalidWeight {
                weight: -1.0,
                reason: "negative".into(),
            }),
            "Invalid input: Invalid hyperedge weight -1: negative",
        ),
        (
            GraphError::Mutation(GraphMutationError::DuplicateHyperEdgeId(123)),
            "Conflict: Duplicate hyperedge ID: 123",
        ),
        (
            GraphError::Mutation(GraphMutationError::Internal("mut_internal".into())),
            "Internal error: mut_internal",
        ),
        (
            GraphError::Mutation(GraphMutationError::HyperedgeNotFound(99)),
            "Not found: Hyperedge not found: 99",
        ),
    ];

    for (g_err, expected_str) in variants {
        let c_err: ContextraError = g_err.into();
        assert_eq!(c_err.to_string(), expected_str);
    }
}
