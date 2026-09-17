// FILE-CONTEXT
// ZWECK: Contract-First Hyperkanten-Datenstrukturen (HyperEdge, HyperEdgeId, RoleBinding)
// INVARIANTEN: Zero-Panic, pure Rust data types.

use memfuse_core::{DocId, EntityId};
use serde::{Deserialize, Serialize};

/// Eindeutiger Bezeichner für eine Hyperkante in MemFuse Graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HyperEdgeId(pub u64);

impl HyperEdgeId {
    /// Erzeugt eine neue `HyperEdgeId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den zugrundeliegenden `u64`-Wert zurück.
    pub const fn inner(&self) -> u64 {
        self.0
    }
}

/// Rolle und Entitätsbindung innerhalb einer Hyperkante.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleBinding {
    pub role: String,
    pub entity_id: EntityId,
}

impl RoleBinding {
    /// Erzeugt eine neue `RoleBinding`.
    pub fn new(role: impl Into<String>, entity_id: EntityId) -> Self {
        Self {
            role: role.into(),
            entity_id,
        }
    }
}

/// N-äre Hyperkante, die mehrere Entitäten über Rollenbindungen verknüpft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HyperEdge {
    pub id: HyperEdgeId,
    pub label: String,
    pub bindings: Vec<RoleBinding>,
    pub source_doc_id: Option<DocId>,
    pub is_tombstoned: bool,
}

impl HyperEdge {
    /// Erzeugt eine neue `HyperEdge`.
    pub fn new(
        id: HyperEdgeId,
        label: impl Into<String>,
        bindings: Vec<RoleBinding>,
        source_doc_id: Option<DocId>,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            bindings,
            source_doc_id,
            is_tombstoned: false,
        }
    }
}
