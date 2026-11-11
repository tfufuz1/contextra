//! Standalone domain types and identifiers for `contextra-graph` (Klasse A, SPEC §5).

// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Autonome Domänentypen (EntityId, TxId, DocId, Edge, Entity).
// INVARIANTEN: Deterministische Hashing- und Identitätsregeln.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Maximum candidates in search results.

// SECTION Types Core Implementation
// ANCHOR[contextra-graph:SRC-TYPES-001]
pub const MAX_SEARCH_K: usize = 1_000;

// Canonical identity types from contextra-ids
pub use contextra_ids::{DocId, EntityId, TxId};

/// Entity node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// ANCHOR[contextra-graph:SRC-TYPES-005]
pub struct Entity {
    /// Unique entity ID.
    pub id: EntityId,
    /// Canonical entity name.
    pub name: Arc<str>,
    /// Entity type / category.
    pub entity_type: Arc<str>,
}

impl Entity {
    /// Creates a new `Entity`.
    pub fn new(id: EntityId, name: impl Into<String>, entity_type: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into().into(),
            entity_type: entity_type.into().into(),
        }
    }
}

/// Directed edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// ANCHOR[contextra-graph:SRC-TYPES-006]
pub struct Edge {
    /// Source entity ID.
    pub from: EntityId,
    /// Target entity ID.
    pub to: EntityId,
    /// Edge relation label.
    pub label: Arc<str>,
    /// Edge weight.
    pub weight: f32,
    /// Source document ID.
    pub source_doc_id: Option<DocId>,
}

impl Edge {
    /// Creates a new `Edge`.
    pub fn new(from: EntityId, to: EntityId, label: impl Into<String>) -> Self {
        Self {
            from,
            to,
            label: label.into().into(),
            weight: 1.0,
            source_doc_id: None,
        }
    }

    /// Sets edge weight.
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    /// Sets source doc id.
    pub fn with_source_doc_id(mut self, doc_id: DocId) -> Self {
        self.source_doc_id = Some(doc_id);
        self
    }
}

/// Hyperedge identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
// ANCHOR[contextra-graph:SRC-TYPES-007]
pub struct HyperEdgeId(pub u64);

impl HyperEdgeId {
    /// Creates a new `HyperEdgeId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner numeric representation.
    pub const fn inner(self) -> u64 {
        self.0
    }
}

impl fmt::Display for HyperEdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Role identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
// ANCHOR[contextra-graph:SRC-TYPES-008]
pub struct RoleId(pub u32);

impl RoleId {
    /// Creates a new `RoleId`.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the inner numeric representation.
    pub const fn inner(self) -> u32 {
        self.0
    }
}

impl fmt::Display for RoleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Role binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
// ANCHOR[contextra-graph:SRC-TYPES-009]
pub struct RoleBinding {
    /// Role identifier.
    pub role: RoleId,
    /// Participant entity ID.
    pub entity: EntityId,
}

impl RoleBinding {
    /// Creates a new `RoleBinding`.
    pub const fn new(role: RoleId, entity: EntityId) -> Self {
        Self { role, entity }
    }
}

/// Thresholded local high-frequency diffusion parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
// ANCHOR[contextra-graph:SRC-TYPES-010]
pub struct TlHfdParams {
    /// Regularization scale.
    pub sigma: f32,
    /// Seed injection multiplier.
    pub delta: f32,
    /// Boundary conductance exponent.
    pub gamma: f32,
    /// Maximum number of diffusion iterations.
    pub max_iterations: u32,
    /// Maximum number of boundary nodes to activate per iteration.
    pub max_top_k_expansion: usize,
    /// Maximum hyperedge participant size before truncation.
    pub max_hyperedge_sort_size: usize,
}

impl Default for TlHfdParams {
    fn default() -> Self {
        Self {
            sigma: 0.1,
            delta: 2.0,
            gamma: 1.0,
            max_iterations: 50,
            max_top_k_expansion: 10,
            max_hyperedge_sort_size: 64,
        }
    }
}

/// Execution algorithm variant for Personalized PageRank.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[non_exhaustive]
// ANCHOR[contextra-graph:SRC-TYPES-011]
pub enum PprAlgorithm {
    /// Auto heuristic dispatch.
    #[default]
    Auto,
    /// Dense power-iteration algorithm.
    DensePowerIteration,
    /// Andersen-Chung-Lang forward push approximation.
    ForwardPush,
    /// Shadow mode comparing ForwardPush vs DensePowerIteration.
    ShadowMode,
    /// Thresholded local high-frequency diffusion.
    TlHfd(TlHfdParams),
    /// Shadow mode comparing TlHfd against baseline.
    ShadowModeTlHfd(TlHfdParams),
}

/// Configuration parameters for Personalized PageRank execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// ANCHOR[contextra-graph:SRC-TYPES-012]
pub struct PprConfig {
    /// Damping factor.
    pub damping_factor: f32,
    /// Maximum convergence iterations.
    pub max_iterations: u32,
    /// Convergence tolerance delta.
    pub convergence_epsilon: f32,
    /// Algorithm variant.
    pub algorithm: PprAlgorithm,
    /// Warning emitted on non-convergence.
    pub warn_on_non_convergence: bool,
}

impl Default for PprConfig {
    fn default() -> Self {
        Self {
            damping_factor: 0.85,
            max_iterations: 50,
            convergence_epsilon: 1e-5,
            algorithm: PprAlgorithm::default(),
            warn_on_non_convergence: true,
        }
    }
}

/// Resource tracker.
#[derive(Debug, Clone, Default)]
// ANCHOR[contextra-graph:SRC-TYPES-013]
pub struct ResourceTracker {
    consumed_bytes: usize,
    max_bytes: usize,
}

impl ResourceTracker {
    /// Unlimited tracker.
    pub fn unlimited() -> Self {
        Self {
            consumed_bytes: 0,
            max_bytes: usize::MAX,
        }
    }

    /// Tracker with byte limit.
    pub fn with_limit(max_bytes: usize) -> Self {
        Self {
            consumed_bytes: 0,
            max_bytes,
        }
    }

    /// Records consumed bytes.
    pub fn record_allocation(&mut self, bytes: usize) {
        self.consumed_bytes = self.consumed_bytes.saturating_add(bytes);
    }

    /// Checks if budget is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.consumed_bytes > self.max_bytes
    }
}

/// Request context placeholder.
#[derive(Debug, Clone, Default)]
// ANCHOR[contextra-graph:SRC-TYPES-014]
pub struct RequestContext {
    /// Optional tenant identifier.
    pub tenant_id: Option<String>,
}

impl RequestContext {
    /// Creates a new default request context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a request context with tenant identifier.
    pub fn with_tenant(tenant: impl Into<String>) -> Self {
        Self {
            tenant_id: Some(tenant.into()),
        }
    }
}

// !SECTION Types Core Implementation
