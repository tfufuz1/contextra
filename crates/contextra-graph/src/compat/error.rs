//! Canonical error conversion between `GraphError` and `ContextraError`.

// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Kompatibilitätsbrücke für Ports und Types (SPEC-v2 §4.3).
// INVARIANTEN: Isolierte Workspace-Konvertierungen.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

use crate::error::{GraphError, GraphMutationError};
use contextra_types::ContextraError;


// SECTION Error Core Implementation
impl From<GraphError> for ContextraError {
    fn from(err: GraphError) -> Self {
        match err {
            GraphError::InvalidInput(msg) => ContextraError::InvalidInput(msg),
            GraphError::NotFound(msg) => ContextraError::NotFound(msg),
            GraphError::PolicyViolation(msg) => ContextraError::PolicyViolation(msg),
            GraphError::Serialization(msg) => ContextraError::Serialization(msg),
            GraphError::Storage(msg) => ContextraError::Storage(msg),
            GraphError::BudgetExhausted(msg) => {
                ContextraError::Internal(format!("Budget exhausted: {msg}"))
            }
            GraphError::Mutation(e) => e.into(),
            GraphError::Internal(msg) => ContextraError::Internal(msg),
        }
    }
}

impl From<GraphMutationError> for ContextraError {
    fn from(err: GraphMutationError) -> Self {
        match err {
            GraphMutationError::LockAcquisitionTimeout(msg) => {
                ContextraError::Transaction(format!("Lock acquisition timed out: {msg}"))
            }
            GraphMutationError::RoleBindingInvalid(msg) => {
                ContextraError::InvalidInput(format!("Invalid role binding: {msg}"))
            }
            GraphMutationError::EpochReclamationPending(epoch) => {
                ContextraError::Conflict(format!("Epoch reclamation pending for epoch: {epoch}"))
            }
            GraphMutationError::InsufficientParticipants { expected, found } => {
                ContextraError::InvalidInput(format!(
                    "Insufficient participants for hyperedge: expected at least {expected}, found {found}"
                ))
            }
            GraphMutationError::HyperedgeNotFound(id) => {
                ContextraError::NotFound(format!("Hyperedge not found: {id}"))
            }
            GraphMutationError::InvalidWeight { weight, reason } => {
                ContextraError::InvalidInput(format!("Invalid hyperedge weight {weight}: {reason}"))
            }
            GraphMutationError::DuplicateHyperEdgeId(id) => {
                ContextraError::Conflict(format!("Duplicate hyperedge ID: {id}"))
            }
            GraphMutationError::Internal(msg) => ContextraError::Internal(msg),
        }
    }
}

// !SECTION Error Core Implementation
