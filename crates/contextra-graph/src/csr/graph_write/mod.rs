//! Schreib- und Mutationsoperationen auf der CSR-Graphstruktur.
//!
//! # Zweck
//! Definiert den Haupttyp `CsrGraph` und synchronisiert parallele Modifikationen über Mutexe.

// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Compressed Sparse Row (CSR) Graph-Engine Kern.
// INVARIANTEN: Bitemporale Sichtbarkeit, lock-freie Lesepfade.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md


// SECTION Mod Core Implementation
pub mod edge_mutation;
pub mod entity;
pub mod hyperedge;
pub mod tombstone;

use arc_swap::ArcSwap;
use parking_lot::{Mutex, RwLock};
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use crate::consistency_enforcement::ConsistencyEnforcer;
#[cfg(feature = "ports-compat")]
use contextra_ports::StorageEngine;
use contextra_types::{DocId, EntityId};

use super::inner::{GraphInner, InnerWriteGuard, MemoryEstimate};
use super::types::CsrGraphConfig;

// ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-001]
pub struct CsrGraph {
    pub config: CsrGraphConfig,
    pub inner: Arc<ArcSwap<GraphInner>>,
    pub write_state: Arc<Mutex<GraphInner>>,
    #[cfg(feature = "ports-compat")]
    pub storage: Option<Arc<dyn StorageEngine>>,
    pub last_tx_id: AtomicU64,
    pub consistency_enforcer: Option<RwLock<ConsistencyEnforcer>>,
    pub doc_edge_index: crate::provenance::DocEdgeIndex,
    pub cascade_queue: Arc<Mutex<VecDeque<(DocId, crate::hyperedge::HyperEdgeId)>>>,
}

impl CsrGraph {
    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-002]
    pub fn new() -> Self {
        Self::with_config(CsrGraphConfig::default())
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-003]
    pub fn with_config(config: CsrGraphConfig) -> Self {
        let initial = Arc::new(GraphInner::new());
        Self {
            config,
            inner: Arc::new(ArcSwap::from(initial.clone())),
            write_state: Arc::new(Mutex::new((*initial).clone())),
            #[cfg(feature = "ports-compat")]
            storage: None,
            last_tx_id: AtomicU64::new(0),
            consistency_enforcer: None,
            doc_edge_index: crate::provenance::DocEdgeIndex::new(),
            cascade_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-004]
    #[cfg(feature = "ports-compat")]
    pub fn with_storage(storage: Arc<dyn StorageEngine>) -> Self {
        Self::with_config_and_storage(CsrGraphConfig::default(), storage)
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-005]
    #[cfg(feature = "ports-compat")]
    pub fn with_config_and_storage(
        config: CsrGraphConfig,
        storage: Arc<dyn StorageEngine>,
    ) -> Self {
        let initial = Arc::new(GraphInner::new());
        Self {
            config,
            inner: Arc::new(ArcSwap::from(initial.clone())),
            write_state: Arc::new(Mutex::new((*initial).clone())),
            storage: Some(storage),
            last_tx_id: AtomicU64::new(0),
            consistency_enforcer: None,
            doc_edge_index: crate::provenance::DocEdgeIndex::new(),
            cascade_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-006]
    pub fn with_consistency_enforcer(suppression_threshold: u32) -> Self {
        let initial = Arc::new(GraphInner::new());
        Self {
            config: CsrGraphConfig::default(),
            inner: Arc::new(ArcSwap::from(initial.clone())),
            write_state: Arc::new(Mutex::new((*initial).clone())),
            #[cfg(feature = "ports-compat")]
            storage: None,
            last_tx_id: AtomicU64::new(0),
            consistency_enforcer: Some(RwLock::new(ConsistencyEnforcer::new(
                suppression_threshold,
            ))),
            doc_edge_index: crate::provenance::DocEdgeIndex::new(),
            cascade_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-007]
    /// Delegiert an [`GraphIndex::personalized_page_rank_at`].
    #[cfg(feature = "ports-compat")]
    pub fn personalized_page_rank_at<'a>(
        &'a self,
        seed_nodes: &'a [contextra_types::EntityId],
        config: &'a contextra_types::PprConfig,
        seq_no: u64,
    ) -> contextra_ports::BoxFuture<
        'a,
        contextra_types::Result<Vec<(contextra_types::EntityId, f32)>>,
    > {
        contextra_ports::GraphIndex::personalized_page_rank_at(self, seed_nodes, config, seq_no)
    }

    /// Delegiert an [`GraphIndex::rollback_to_tx`].
    #[cfg(feature = "ports-compat")]
    pub fn rollback_to_tx<'a>(
        &'a self,
        tx_id: contextra_types::TxId,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        contextra_ports::GraphIndex::rollback_to_tx(self, tx_id)
    }

    pub fn inner_read(&self) -> arc_swap::Guard<Arc<GraphInner>> {
        self.inner.load()
    }

    pub(crate) fn inner_write(&self) -> InnerWriteGuard<'_> {
        InnerWriteGuard {
            guard: self.write_state.lock(),
            arc_swap: &self.inner,
        }
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-008]
    #[cfg(feature = "ports-compat")]
    pub fn set_storage(&mut self, storage: Arc<dyn StorageEngine>) {
        self.storage = Some(storage);
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-009]
    #[cfg(feature = "ports-compat")]
    pub fn storage(&self) -> Option<Arc<dyn StorageEngine>> {
        self.storage.clone()
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-010]
    pub fn estimate_memory(&self, presized: bool) -> MemoryEstimate {
        let inner = self.inner_read();
        (*inner).estimate_memory(presized)
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-011]
    pub fn estimate_memory_bytes(&self) -> usize {
        let inner = self.inner_read();
        (*inner).estimate_memory_bytes()
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-012]
    pub fn estimate_compaction_peak_bytes(&self) -> usize {
        let inner = self.inner_read();
        (*inner).estimate_compaction_peak_bytes()
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-013]
    pub fn get_source_doc_id(&self, index: usize) -> Option<DocId> {
        let inner = self.inner_read();
        inner.source_doc_id_at(index)
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-014]
    pub fn source_doc_id_at(&self, from: EntityId, to: EntityId) -> Option<DocId> {
        let inner = self.inner_read();

        for ((_, staged_from), staged_vec) in inner.staged_edges.iter() {
            if *staged_from == from {
                if let Some(staged) = staged_vec.iter().find(|e| e.target == to) {
                    if staged.source_doc_id.is_some() {
                        return staged.source_doc_id;
                    }
                }
            }
        }

        let from_idx = *inner.id_map.get(&from)?;
        let to_idx = *inner.id_map.get(&to)?;

        if let Some(pending) = inner.pending_edges.get(&from_idx) {
            if let Some(edge) = pending.iter().find(|e| e.target == to_idx) {
                return edge.source_doc_id;
            }
        }

        if from_idx < inner.offsets.len() - 1 {
            let start = inner.offsets.get(from_idx).copied().unwrap_or(0);
            let end = inner.offsets.get(from_idx + 1).copied().unwrap_or(0);
            for j in start..end {
                if inner.targets.get(j) == Some(&to_idx) {
                    return inner.source_doc_id_at(j);
                }
            }
        }

        None
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-MOD-015]
    pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<(EntityId, EntityId)> {
        let inner = self.inner_read();
        let mut edges: HashSet<(EntityId, EntityId)> =
            inner.doc_to_edges.get(&doc_id).cloned().unwrap_or_default();
        for edge in self.doc_edge_index.edges_for_doc(doc_id) {
            edges.insert(edge);
        }
        edges.into_iter().collect()
    }
}

// !SECTION Mod Core Implementation
