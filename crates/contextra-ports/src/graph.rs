//! Graph ports and collection mutation trait definitions (Anhang B §B.5.1.3).

pub use crate::graph_index::{GraphIndex, GraphIndexStats};
use contextra_types::types::domain::{DocId, EntityId};
use contextra_types::ContextraError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Identifier for a participant's role within an n-ary hyperedge.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[repr(transparent)]
pub struct RoleId(pub u32);

impl RoleId {
    /// Creates a new `RoleId`.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the underlying `u32` value.
    #[inline]
    pub const fn inner(self) -> u32 {
        self.0
    }
}

impl From<u32> for RoleId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for RoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RoleId({})", self.0)
    }
}

/// Binding of an entity to a specific role within an n-ary hyperedge.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleBinding {
    /// The role assigned to the entity in the hyperedge.
    pub role: RoleId,
    /// The bound entity.
    pub entity: EntityId,
}

impl RoleBinding {
    /// Creates a new `RoleBinding`.
    pub fn new(role: RoleId, entity: EntityId) -> Self {
        Self { role, entity }
    }
}

/// Unique identifier for an n-ary hyperedge.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[repr(transparent)]
pub struct HyperEdgeId(pub u64);

impl HyperEdgeId {
    /// Creates a new `HyperEdgeId`.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying `u64` value.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }
}

impl From<u64> for HyperEdgeId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for HyperEdgeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HyperEdgeId({})", self.0)
    }
}

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

/// Trait defining n-ary hyperedge collection mutations for graph indexing (Anhang B §B.5.1.3).
///
/// # Dyn-Kompatibilität
/// Dieser Trait ist durch synchrone Rückgabetypen vtable-kompatibel (dyn-safe).
pub trait GraphCollectionMutation: Send + Sync + 'static {
    /// Relates multiple entities into an n-ary hyperedge associated with a document ID.
    fn relate_n_ary(
        &self,
        predicate_tag: u32,
        participants: &[RoleBinding],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, GraphMutationError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[derive(Default)]
    struct MockGraphMutation {
        next_id: AtomicU64,
    }

    impl GraphCollectionMutation for MockGraphMutation {
        fn relate_n_ary(
            &self,
            predicate_tag: u32,
            participants: &[RoleBinding],
            _doc_id: DocId,
        ) -> Result<HyperEdgeId, GraphMutationError> {
            if participants.len() < 2 {
                return Err(GraphMutationError::InsufficientParticipants {
                    expected: 2,
                    found: participants.len(),
                });
            }
            let _ = predicate_tag;
            let id = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
            Ok(HyperEdgeId::new(id))
        }
    }

    #[test]
    fn test_graph_collection_mutation_mock() {
        let mutator = MockGraphMutation::default();
        let bindings = vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(10)),
            RoleBinding::new(RoleId::new(2), EntityId::new(20)),
        ];

        let res = mutator.relate_n_ary(100, &bindings, DocId::from(1u64));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), HyperEdgeId::new(1));

        let err_res = mutator.relate_n_ary(100, &bindings[..1], DocId::from(1u64));
        assert!(matches!(
            err_res,
            Err(GraphMutationError::InsufficientParticipants {
                expected: 2,
                found: 1
            })
        ));
    }

    #[test]
    fn test_graph_mutation_error_conversion() {
        let err = GraphMutationError::RoleBindingInvalid("role_0".into());
        let contextra_err: ContextraError = err.into();
        assert!(
            matches!(contextra_err, ContextraError::InvalidInput(ref msg) if msg.contains("Invalid role binding: role_0"))
        );
    }
}
