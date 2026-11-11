//! Error taxonomy and Result types for `contextra-graph` (Klasse A, B4).
// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Kanonische Fehler-Taxonomie GraphError und GraphMutationError.
// INVARIANTEN: thiserror-basierte typisierte Fehlerbehandlung, zero-panic, bitgleiche Fehlermeldungen.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md


use thiserror::Error;

// SECTION Error Core Implementation

/// Crate-level Result type.
// ANCHOR[contextra-graph:SRC-ERROR-001]
pub type Result<T, E = GraphError> = std::result::Result<T, E>;

/// Canonical error taxonomy for `contextra-graph`.
#[derive(Debug, Error)]
#[non_exhaustive]
// ANCHOR[contextra-graph:SRC-ERROR-002]
pub enum GraphError {
    /// Invalid input parameter or argument.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Requested node, edge, or hyperedge not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Graph mutation conflict or policy violation.
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Storage or persistence I/O failure.
    #[error("Storage error: {0}")]
    Storage(String),

    /// Resource budget exhausted (e.g. node traversal limit or memory).
    #[error("Budget exhausted: {0}")]
    BudgetExhausted(String),

    /// Specific graph mutation error.
    #[error("Mutation error: {0}")]
    Mutation(#[from] GraphMutationError),

    /// Internal unexpected error.
    #[error("Internal graph error: {0}")]
    Internal(String),
}

/// Detailed errors occurring during graph mutations.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
// ANCHOR[contextra-graph:SRC-ERROR-003]
pub enum GraphMutationError {
    /// Lock acquisition timed out.
    #[error("Lock acquisition timed out: {0}")]
    LockAcquisitionTimeout(String),

    /// Invalid role binding for hyperedge.
    #[error("Invalid role binding: {0}")]
    RoleBindingInvalid(String),

    /// Epoch reclamation pending.
    #[error("Epoch reclamation pending for epoch: {0}")]
    EpochReclamationPending(u64),

    /// Insufficient participants for hyperedge creation.
    #[error(
        "Insufficient participants for hyperedge: expected at least {expected}, found {found}"
    )]
    InsufficientParticipants {
        /// Expected minimum participants.
        expected: usize,
        /// Actual found participants.
        found: usize,
    },

    /// Hyperedge was not found.
    #[error("Hyperedge not found: {0}")]
    HyperedgeNotFound(u64),

    /// Invalid hyperedge weight.
    #[error("Invalid hyperedge weight {weight}: {reason}")]
    InvalidWeight {
        /// Weight value provided.
        weight: f32,
        /// Reason why weight is invalid.
        reason: String,
    },

    /// Duplicate hyperedge ID.
    #[error("Duplicate hyperedge ID: {0}")]
    DuplicateHyperEdgeId(u64),

    /// Internal mutation failure.
    #[error("Internal graph mutation error: {0}")]
    Internal(String),
}

// !SECTION Error Core Implementation
