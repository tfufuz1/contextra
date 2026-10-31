//! Kern-Datenstrukturen des inneren CSR-Graphzustands (`GraphInner`).
//!
//! # Zweck
//! Hält Knoten-, Offset- und Kanten-Arrays flach im RAM für optimale CPU-Cache-Ausnutzung.

pub mod compaction;
pub mod estimate;
pub mod reinforcement;

use arc_swap::ArcSwap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use contextra_types::{DocId, Entity, EntityId, TxId};

#[cfg(feature = "edge-reinforcement-learning")]
use super::types::CsrEdge;
use super::types::{EdgePayload, InternalIndex, StagedEdgePayload};

pub use estimate::MemoryEstimate;

#[derive(Clone)]
// ANCHOR[contextra-graph:SRC-CSR-INNER-MOD-001]
pub struct GraphInner {
    pub(crate) id_map: HashMap<EntityId, InternalIndex>,
    pub(crate) reverse_map: Vec<EntityId>,
    pub(crate) entities: Vec<Entity>,
    pub(crate) communities: HashMap<EntityId, u64>,
    pub(crate) communities_loaded: bool,

    pub(crate) offsets: Vec<usize>,
    pub(crate) targets: Vec<InternalIndex>,
    pub(crate) weights: Vec<f32>,
    pub(crate) tx_valid_froms: Vec<TxId>,
    pub(crate) tx_valid_tos: Vec<TxId>,
    pub(crate) business_valid_froms: Vec<i64>,
    pub(crate) business_valid_tos: Vec<i64>,
    pub(crate) source_doc_ids: Vec<DocId>,
    pub(crate) out_weight_sums: Vec<f32>,

    pub(crate) doc_to_edges: ahash::AHashMap<DocId, HashSet<(EntityId, EntityId)>>,

    pub(crate) staged_entities: ahash::AHashMap<(TxId, EntityId), Entity>,
    pub(crate) staged_edges: ahash::AHashMap<(TxId, EntityId), Vec<StagedEdgePayload>>,
    pub(crate) staged_removals: ahash::AHashMap<TxId, Vec<(EntityId, EntityId)>>,
    pub(crate) pending_edges: HashMap<InternalIndex, Vec<EdgePayload>>,
    pub(crate) tombstoned_edges: HashSet<(InternalIndex, InternalIndex)>,
    pub(crate) pending_edge_count: usize,
    pub(crate) is_dirty: bool,
    #[cfg(feature = "edge-reinforcement-learning")]
    pub(crate) edge_store: HashMap<EntityId, Vec<CsrEdge>>,

    pub(crate) hyperedges: HashMap<crate::hyperedge::HyperEdgeId, Arc<crate::hyperedge::HyperEdge>>,
    pub(crate) doc_to_hyperedges: ahash::AHashMap<DocId, HashSet<crate::hyperedge::HyperEdgeId>>,
    pub(crate) hyperedge_index: ahash::AHashMap<EntityId, HashSet<crate::hyperedge::HyperEdgeId>>,
    pub(crate) child_to_parents:
        ahash::AHashMap<crate::hyperedge::HyperEdgeId, HashSet<crate::hyperedge::HyperEdgeId>>,
}

#[inline]
pub(crate) fn sentinel_entity() -> Entity {
    Entity::new(EntityId::new(0), "", "")
}

impl Default for GraphInner {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphInner {
// ANCHOR[contextra-graph:SRC-CSR-INNER-MOD-002]
    pub fn new() -> Self {
        Self {
            id_map: HashMap::new(),
            reverse_map: Vec::new(),
            entities: Vec::new(),
            communities: HashMap::new(),
            communities_loaded: false,
            offsets: vec![0],
            targets: Vec::new(),
            weights: Vec::new(),
            tx_valid_froms: Vec::new(),
            tx_valid_tos: Vec::new(),
            business_valid_froms: Vec::new(),
            business_valid_tos: Vec::new(),
            source_doc_ids: Vec::new(),
            out_weight_sums: Vec::new(),
            doc_to_edges: ahash::AHashMap::new(),
            staged_entities: ahash::AHashMap::default(),
            staged_edges: ahash::AHashMap::default(),
            staged_removals: ahash::AHashMap::default(),
            pending_edges: HashMap::new(),
            tombstoned_edges: HashSet::new(),
            pending_edge_count: 0,
            is_dirty: false,
            #[cfg(feature = "edge-reinforcement-learning")]
            edge_store: HashMap::new(),
            hyperedges: HashMap::new(),
            doc_to_hyperedges: ahash::AHashMap::new(),
            hyperedge_index: ahash::AHashMap::new(),
            child_to_parents: ahash::AHashMap::new(),
        }
    }

    #[inline]
    pub(crate) fn entity_at(&self, idx: usize) -> Option<&Entity> {
        self.entities.get(idx).filter(|e| e.id != EntityId::new(0))
    }

    #[inline]
    pub(crate) fn tx_valid_from_at(&self, idx: usize) -> Option<TxId> {
        self.tx_valid_froms
            .get(idx)
            .copied()
            .filter(|&tx| tx != TxId::INVALID)
    }

    #[inline]
    pub(crate) fn tx_valid_to_at(&self, idx: usize) -> Option<TxId> {
        self.tx_valid_tos
            .get(idx)
            .copied()
            .filter(|&tx| tx != TxId::INVALID)
    }

    #[inline]
    pub(crate) fn business_valid_from_at(&self, idx: usize) -> Option<i64> {
        self.business_valid_froms
            .get(idx)
            .copied()
            .filter(|&ts| ts != i64::MIN)
    }

    #[inline]
    pub(crate) fn business_valid_to_at(&self, idx: usize) -> Option<i64> {
        self.business_valid_tos
            .get(idx)
            .copied()
            .filter(|&ts| ts != i64::MIN)
    }

    #[inline]
    pub(crate) fn source_doc_id_at(&self, idx: usize) -> Option<DocId> {
        self.source_doc_ids
            .get(idx)
            .copied()
            .filter(|d| d.inner() != 0)
    }
}

pub(crate) struct InnerWriteGuard<'a> {
    pub(crate) guard: parking_lot::MutexGuard<'a, GraphInner>,
    pub(crate) arc_swap: &'a ArcSwap<GraphInner>,
}

impl<'a> std::ops::Deref for InnerWriteGuard<'a> {
    type Target = GraphInner;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a> std::ops::DerefMut for InnerWriteGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<'a> Drop for InnerWriteGuard<'a> {
    fn drop(&mut self) {
        self.arc_swap.store(Arc::new((*self.guard).clone()));
    }
}
