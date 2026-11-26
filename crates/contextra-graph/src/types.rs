//! Standalone domain types and identifiers for `contextra-graph` (Klasse A, SPEC §5).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Maximum candidates in search results.
pub const MAX_SEARCH_K: usize = 1_000;

/// Identifier of an entity node in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct EntityId(pub u64);

impl EntityId {
    /// Creates a new `EntityId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner numeric representation.
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Creates an entity id from a string slice deterministically.
    pub fn from_key(key: impl AsRef<[u8]>) -> Result<Self, std::num::ParseIntError> {
        let s = std::str::from_utf8(key.as_ref()).unwrap_or("");
        if let Ok(num) = s.parse::<u64>() {
            return Ok(Self(num));
        }
        let h = blake3::hash(key.as_ref());
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&h.as_bytes()[..8]);
        Ok(Self(u64::from_le_bytes(bytes)))
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for EntityId {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<EntityId> for u64 {
    fn from(id: EntityId) -> Self {
        id.0
    }
}

/// Document identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct DocId(pub u64);

impl DocId {
    /// Creates a new `DocId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner numeric representation.
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Derives a `DocId` deterministically from a key slice using BLAKE3.
    pub fn from_key(key: impl AsRef<[u8]>) -> Result<Self, std::num::ParseIntError> {
        let s = std::str::from_utf8(key.as_ref()).unwrap_or("");
        if let Ok(num) = s.parse::<u64>() {
            return Ok(Self(num));
        }
        let h = blake3::hash(key.as_ref());
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&h.as_bytes()[..8]);
        Ok(Self(u64::from_le_bytes(bytes)))
    }
}

impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for DocId {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<DocId> for u64 {
    fn from(id: DocId) -> Self {
        id.0
    }
}

/// Transaction or sequence identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct TxId(pub u64);

impl TxId {
    /// Creates a new `TxId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner numeric representation.
    pub const fn inner(self) -> u64 {
        self.0
    }
}

impl fmt::Display for TxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for TxId {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<TxId> for u64 {
    fn from(tx: TxId) -> Self {
        tx.0
    }
}

/// Entity node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
