use contextra_core::{DocId, EntityId, ResourceTracker, TxId};
use serde::{Deserialize, Serialize};

pub(crate) const GRAPH_ENTITY_PREFIX: &[u8] = b"__graph:entity:";
pub(crate) const GRAPH_EDGE_PREFIX: &[u8] = b"__graph:edge:";
pub(crate) const GRAPH_ENTITY_DELETED_PREFIX: &[u8] = b"graph:entity:deleted:";
pub(crate) const GRAPH_COMMUNITY_PREFIX: &[u8] = b"__graph:community:";

/// Edge type representation for CSR edges.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EdgeType {
    #[default]
    Default,
}

/// Edge structure in CSR graph representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub target: EntityId,
    pub weight: f32,
    pub edge_type: EdgeType,
    #[cfg(feature = "edge-reinforcement-learning")]
    pub cooccurrence_weight: f32, // w_ij, initialisiert mit 0.0
    #[cfg(feature = "edge-reinforcement-learning")]
    pub traversal_weight: f32, // τ_ij, initialisiert mit 0.0
}

impl Edge {
    pub fn new(target: EntityId, weight: f32) -> Self {
        Self {
            target,
            weight,
            edge_type: EdgeType::Default,
            #[cfg(feature = "edge-reinforcement-learning")]
            cooccurrence_weight: 0.0,
            #[cfg(feature = "edge-reinforcement-learning")]
            traversal_weight: 0.0,
        }
    }
}
use std::sync::Arc;

/// Persisted edge payload format for storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedEdgePayload {
    pub weight: f32,
    #[serde(default, alias = "valid_from")]
    pub tx_valid_from: Option<TxId>,
    #[serde(default, alias = "valid_to")]
    pub tx_valid_to: Option<TxId>,
    #[serde(default)]
    pub business_valid_from: Option<i64>,
    #[serde(default)]
    pub business_valid_to: Option<i64>,
    #[serde(default)]
    pub source_doc_id: Option<DocId>,
}

/// Score decay factor per hop (0.7^hop).
pub const SCORE_DECAY: f32 = 0.7;

/// Maximum traversal depth.
pub const MAX_TRAVERSAL_HOPS: u8 = 3;

/// Maximum visited nodes limit during BFS traversal to prevent intermediate hub-node memory explosion.
pub const MAX_VISITED_NODES: usize = 10_000;
#[derive(Debug, Clone)]
pub struct CsrGraphConfig {
    /// Rebuild threshold: max number of uncompacted pending edges in delta buffer before triggering an automatic full CSR rebuild.
    pub rebuild_threshold: usize,
    /// Max compaction peak memory limit in MB (IP-08). Compaction will be deferred if current graph memory + rebuild allocation exceeds this threshold.
    pub max_compaction_peak_memory_mb: Option<usize>,
    /// Optional global ResourceTracker handle for cross-crate memory budget coupling (IP-08).
    pub resource_tracker: Option<Arc<ResourceTracker>>,
}

impl Default for CsrGraphConfig {
    fn default() -> Self {
        Self {
            rebuild_threshold: 1000,
            max_compaction_peak_memory_mb: Some(1024),
            resource_tracker: None,
        }
    }
}

/// Internal contiguous index for CSR arrays.
pub(crate) type InternalIndex = usize;

/// Internal representation of an edge payload.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EdgePayload {
    pub(crate) target: InternalIndex,
    pub(crate) weight: f32,
    pub(crate) tx_valid_from: Option<TxId>,
    pub(crate) tx_valid_to: Option<TxId>,
    pub(crate) business_valid_from: Option<i64>,
    pub(crate) business_valid_to: Option<i64>,
    pub(crate) source_doc_id: Option<DocId>,
}

/// Staging representation of an edge before index allocation at commit time.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StagedEdgePayload {
    pub(crate) target: EntityId,
    pub(crate) weight: f32,
    pub(crate) tx_valid_from: Option<TxId>,
    pub(crate) tx_valid_to: Option<TxId>,
    pub(crate) business_valid_from: Option<i64>,
    pub(crate) business_valid_to: Option<i64>,
    pub(crate) source_doc_id: Option<DocId>,
}
