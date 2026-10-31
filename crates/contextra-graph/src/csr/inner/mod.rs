//! Modul `mod` im Ring 0 (Graph-Subsystem) von Contextra.
//!
//! Stellt Kernkomponenten für Graphtraversierung, CSR-Repräsentationen und
//! Analysemethoden bereit. Wird von Integrationsschichten und Algorithmenmodulen verwendet.

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
// ANCHOR[contextra-graph:GRAPHINNER-0163]
/// Struktur `GraphInner` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
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
// ANCHOR[contextra-graph:SENTINEL_ENT-0164]
/// Funktion `sentinel_entity` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub(crate) fn sentinel_entity() -> Entity {
    Entity::new(EntityId::new(0), "", "")
}

impl Default for GraphInner {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphInner {
    // ANCHOR[contextra-graph:NEW-0165]
    /// Funktion `new` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
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
    // ANCHOR[contextra-graph:ENTITY_AT-0166]
    /// Funktion `entity_at` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn entity_at(&self, idx: usize) -> Option<&Entity> {
        self.entities.get(idx).filter(|e| e.id != EntityId::new(0))
    }

    #[inline]
    // ANCHOR[contextra-graph:TX_VALID_FRO-0167]
    /// Funktion `tx_valid_from_at` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn tx_valid_from_at(&self, idx: usize) -> Option<TxId> {
        self.tx_valid_froms
            .get(idx)
            .copied()
            .filter(|&tx| tx != TxId::INVALID)
    }

    #[inline]
    // ANCHOR[contextra-graph:TX_VALID_TO_-0168]
    /// Funktion `tx_valid_to_at` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn tx_valid_to_at(&self, idx: usize) -> Option<TxId> {
        self.tx_valid_tos
            .get(idx)
            .copied()
            .filter(|&tx| tx != TxId::INVALID)
    }

    #[inline]
    // ANCHOR[contextra-graph:BUSINESS_VAL-0169]
    /// Funktion `business_valid_from_at` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn business_valid_from_at(&self, idx: usize) -> Option<i64> {
        self.business_valid_froms
            .get(idx)
            .copied()
            .filter(|&ts| ts != i64::MIN)
    }

    #[inline]
    // ANCHOR[contextra-graph:BUSINESS_VAL-0170]
    /// Funktion `business_valid_to_at` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn business_valid_to_at(&self, idx: usize) -> Option<i64> {
        self.business_valid_tos
            .get(idx)
            .copied()
            .filter(|&ts| ts != i64::MIN)
    }

    #[inline]
    // ANCHOR[contextra-graph:SOURCE_DOC_I-0171]
    /// Funktion `source_doc_id_at` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn source_doc_id_at(&self, idx: usize) -> Option<DocId> {
        self.source_doc_ids
            .get(idx)
            .copied()
            .filter(|d| d.inner() != 0)
    }
}

// ANCHOR[contextra-graph:INNERWRITEGU-0172]
/// Struktur `InnerWriteGuard` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
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
