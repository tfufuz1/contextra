// TODO(acta-altlast): Bruecken-Modul der Altquelle (HR-6 verbietet Bruecken). Referenz fuer die Trait-Implementierungen (GraphIndex, Purger); beim Umbau in direkte Implementierungen aufloesen.
//! Canonical error conversion between  and .

use crate::error::{GraphError, GraphMutationError};
use acta_base::ActaError; // TODO(acta-hr3): Hub-Import (ex types/ports) nach HR-2/HR-3 aufloesen

impl From<GraphError> for ActaError {
    fn from(err: GraphError) -> Self {
        match err {
            GraphError::InvalidInput(msg) => ActaError::InvalidInput(msg),
            GraphError::NotFound(msg) => ActaError::NotFound(msg),
            GraphError::PolicyViolation(msg) => ActaError::PolicyViolation(msg),
            GraphError::Serialization(msg) => ActaError::Serialization(msg),
            GraphError::Storage(msg) => ActaError::Storage(msg),
            GraphError::BudgetExhausted(msg) => ActaError::Internal(format!("Budget exhausted: {}", msg)),
            GraphError::Mutation(e) => e.into(),
            GraphError::Internal(msg) => ActaError::Internal(msg),
        }
    }
}

impl From<ActaError> for GraphError {
    fn from(err: ActaError) -> Self {
        match err {
            ActaError::InvalidInput(msg) => GraphError::InvalidInput(msg),
            ActaError::NotFound(msg) => GraphError::NotFound(msg),
            ActaError::PolicyViolation(msg) => GraphError::PolicyViolation(msg),
            ActaError::Serialization(msg) => GraphError::Serialization(msg),
            ActaError::Storage(msg) => GraphError::Storage(msg),
            ActaError::MemoryBudgetExceeded { used_mb, limit_mb } => {
                GraphError::BudgetExhausted(format!("Memory budget exceeded: {} MB used, {} MB limit", used_mb, limit_mb))
            }
            ActaError::Transaction(msg) => GraphError::Mutation(GraphMutationError::LockAcquisitionTimeout(msg)),
            ActaError::Conflict(msg) => GraphError::PolicyViolation(format!("Conflict: {}", msg)),
            other => GraphError::Internal(other.to_string()),
        }
    }
}

impl From<GraphMutationError> for ActaError {
    fn from(err: GraphMutationError) -> Self {
        match err {
            GraphMutationError::LockAcquisitionTimeout(msg) => ActaError::Transaction(format!("Lock acquisition timed out: {}", msg)),
            GraphMutationError::RoleBindingInvalid(msg) => ActaError::InvalidInput(format!("Invalid role binding: {}", msg)),
            GraphMutationError::EpochReclamationPending(epoch) => ActaError::Conflict(format!("Epoch reclamation pending for epoch: {}", epoch)),
            GraphMutationError::InsufficientParticipants { expected, found } => {
                ActaError::InvalidInput(format!("Insufficient participants for hyperedge: expected at least {}, found {}", expected, found))
            }
            GraphMutationError::HyperedgeNotFound(id) => ActaError::NotFound(format!("Hyperedge not found: {}", id)),
            GraphMutationError::InvalidWeight { weight, reason } => ActaError::InvalidInput(format!("Invalid hyperedge weight {}: {}", weight, reason)),
            GraphMutationError::DuplicateHyperEdgeId(id) => ActaError::Conflict(format!("Duplicate hyperedge ID: {}", id)),
            GraphMutationError::Internal(msg) => ActaError::Internal(msg),
        }
    }
}
