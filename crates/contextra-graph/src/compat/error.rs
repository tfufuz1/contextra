//! Canonical error conversion between  and .

use crate::error::{GraphError, GraphMutationError};
use contextra_types::ContextraError;

impl From<GraphError> for ContextraError {
    fn from(err: GraphError) -> Self {
        match err {
            GraphError::InvalidInput(msg) => ContextraError::InvalidInput(msg),
            GraphError::NotFound(msg) => ContextraError::NotFound(msg),
            GraphError::PolicyViolation(msg) => ContextraError::PolicyViolation(msg),
            GraphError::Serialization(msg) => ContextraError::Serialization(msg),
            GraphError::Storage(msg) => ContextraError::Storage(msg),
            GraphError::BudgetExhausted(msg) => ContextraError::Internal(format!("Budget exhausted: {}", msg)),
            GraphError::Mutation(e) => e.into(),
            GraphError::Internal(msg) => ContextraError::Internal(msg),
        }
    }
}

impl From<ContextraError> for GraphError {
    fn from(err: ContextraError) -> Self {
        match err {
            ContextraError::InvalidInput(msg) => GraphError::InvalidInput(msg),
            ContextraError::NotFound(msg) => GraphError::NotFound(msg),
            ContextraError::PolicyViolation(msg) => GraphError::PolicyViolation(msg),
            ContextraError::Serialization(msg) => GraphError::Serialization(msg),
            ContextraError::Storage(msg) => GraphError::Storage(msg),
            ContextraError::MemoryBudgetExceeded { used_mb, limit_mb } => {
                GraphError::BudgetExhausted(format!("Memory budget exceeded: {} MB used, {} MB limit", used_mb, limit_mb))
            }
            ContextraError::Transaction(msg) => GraphError::Mutation(GraphMutationError::LockAcquisitionTimeout(msg)),
            ContextraError::Conflict(msg) => GraphError::PolicyViolation(format!("Conflict: {}", msg)),
            other => GraphError::Internal(other.to_string()),
        }
    }
}

impl From<GraphMutationError> for ContextraError {
    fn from(err: GraphMutationError) -> Self {
        match err {
            GraphMutationError::LockAcquisitionTimeout(msg) => ContextraError::Transaction(format!("Lock acquisition timed out: {}", msg)),
            GraphMutationError::RoleBindingInvalid(msg) => ContextraError::InvalidInput(format!("Invalid role binding: {}", msg)),
            GraphMutationError::EpochReclamationPending(epoch) => ContextraError::Conflict(format!("Epoch reclamation pending for epoch: {}", epoch)),
            GraphMutationError::InsufficientParticipants { expected, found } => {
                ContextraError::InvalidInput(format!("Insufficient participants for hyperedge: expected at least {}, found {}", expected, found))
            }
            GraphMutationError::HyperedgeNotFound(id) => ContextraError::NotFound(format!("Hyperedge not found: {}", id)),
            GraphMutationError::InvalidWeight { weight, reason } => ContextraError::InvalidInput(format!("Invalid hyperedge weight {}: {}", weight, reason)),
            GraphMutationError::DuplicateHyperEdgeId(id) => ContextraError::Conflict(format!("Duplicate hyperedge ID: {}", id)),
            GraphMutationError::Internal(msg) => ContextraError::Internal(msg),
        }
    }
}
