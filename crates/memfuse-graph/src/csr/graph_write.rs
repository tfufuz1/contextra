use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::{Mutex, RwLock};
use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};

use memfuse_core::{
    BoxFuture, DocId, Entity, EntityId, GraphIndex, GraphIndexStats, MemFuseError, Result,
    StorageEngine, TxId,
};
use crate::consistency_enforcement::{ConsistencyEnforcer, EdgeAssertion};
use crate::error::GraphMutationError;
use crate::GraphIndexExt;

use super::types::{CsrGraphConfig, Edge, EdgePayload, EdgeType, PersistedEdgePayload, GRAPH_COMMUNITY_PREFIX, GRAPH_EDGE_PREFIX, GRAPH_ENTITY_DELETED_PREFIX, GRAPH_ENTITY_PREFIX};
use super::inner::{sentinel_entity, GraphInner, InnerWriteGuard, MemoryEstimate};
use super::visibility::{is_edge_visible, is_edge_visible_bitemporal, is_edge_visible_business, is_suspicious_tx_id};

/// Compressed Sparse Row graph for entity-relation traversal.
///
/// Implements `GraphIndex` trait as Signal 3 in the 4-Signal Fusion architecture.
pub struct CsrGraph {
    pub config: CsrGraphConfig,
    pub inner: Arc<ArcSwap<GraphInner>>,
    pub write_state: Arc<Mutex<GraphInner>>,
    /// Optionaler Persistenz-Handle. None = reiner In-Memory-Modus (z.B. Tests).
    pub storage: Option<Arc<dyn StorageEngine>>,
    pub last_tx_id: AtomicU64,
    /// Optionales Register für Widerspruchsprävention/Consistency-Enforcement (F-04/ADR-073).
    /// None = disabled (default, P1-safe).
    pub consistency_enforcer: Option<RwLock<ConsistencyEnforcer>>,
    /// Rückverfolgung DocId -> betroffene Kanten, für Cascading-Invalidation (INV-GRAPH-PROV-1).
    pub doc_edge_index: crate::provenance::DocEdgeIndex,
    /// In-memory FIFO cascade queue for hyperedges deferred during high fan-out invalidation (§6.6 / §6.8).
    pub cascade_queue: Arc<Mutex<VecDeque<(DocId, crate::hyperedge::HyperEdgeId)>>>,
}

impl CsrGraph {
    /// Creates a new, empty CSR graph with default configuration.
    pub fn new() -> Self {
        Self::with_config(CsrGraphConfig::default())
    }

    /// Creates a new, empty CSR graph with specified configuration.
    pub fn with_config(config: CsrGraphConfig) -> Self {
        let initial = Arc::new(GraphInner::new());
        Self {
            config,
            inner: Arc::new(ArcSwap::from(initial.clone())),
            write_state: Arc::new(Mutex::new((*initial).clone())),
            storage: None,
            last_tx_id: AtomicU64::new(0),
            consistency_enforcer: None,
            doc_edge_index: crate::provenance::DocEdgeIndex::new(),
            cascade_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Creates a new CSR graph with persistent storage and default config.
    pub fn with_storage(storage: Arc<dyn StorageEngine>) -> Self {
        Self::with_config_and_storage(CsrGraphConfig::default(), storage)
    }

    /// Creates a new CSR graph with configuration and persistent storage.
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

    /// Erstellt CsrGraph mit aktiviertem ConsistencyEnforcer für Widerspruchsprävention (F-04/ADR-073).
    pub fn with_consistency_enforcer(suppression_threshold: u32) -> Self {
        let initial = Arc::new(GraphInner::new());
        Self {
            config: CsrGraphConfig::default(),
            inner: Arc::new(ArcSwap::from(initial.clone())),
            write_state: Arc::new(Mutex::new((*initial).clone())),
            storage: None,
            last_tx_id: AtomicU64::new(0),
            consistency_enforcer: Some(RwLock::new(ConsistencyEnforcer::new(
                suppression_threshold,
            ))),
            doc_edge_index: crate::provenance::DocEdgeIndex::new(),
            cascade_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Tombstoniert eine Kante direkt für eine Transaktions-ID via Cascading-Invalidation (INV-GRAPH-PROV-1).
    pub async fn tombstone_edge(
        &self,
        edge_id: crate::consistency_enforcement::EdgeId,
        tx: TxId,
    ) -> Result<()> {
        let (from, to) = edge_id;
        GraphIndex::remove_edge(self, tx, from, to).await?;
        GraphIndex::commit(self, tx).await?;
        Ok(())
    }

    /// Read path contract for RCU snapshot isolation (IP-08):
    /// Returns a lock-free reference `arc_swap::Guard<Arc<GraphInner>>` to the current `GraphInner` snapshot.
    /// Readers never block on compaction or writer locks.
    ///
    /// # PPR Integration Contract
    /// Downstream PPR algorithms (e.g. `ppr.rs`) consume this snapshot directly via `inner_read()`.
    /// The returned `Arc<GraphInner>` snapshot is point-in-time immutable and guaranteed not to be mutated in-place.
    pub(crate) fn inner_read(&self) -> arc_swap::Guard<Arc<GraphInner>> {
        self.inner.load()
    }

    pub(crate) fn inner_write(&self) -> InnerWriteGuard<'_> {
        InnerWriteGuard {
            guard: self.write_state.lock(),
            arc_swap: &self.inner,
        }
    }

    /// Sets or replaces the persistent storage handle.
    pub fn set_storage(&mut self, storage: Arc<dyn StorageEngine>) {
        self.storage = Some(storage);
    }

    /// Returns a reference to the optional persistent storage handle.
    pub fn storage(&self) -> Option<Arc<dyn StorageEngine>> {
        self.storage.clone()
    }

    /// Enqueues deferred hyperedges for background cascade invalidation.
    pub fn enqueue_cascade_deferred(
        &self,
        doc_id: DocId,
        hyperedge_ids: &[crate::hyperedge::HyperEdgeId],
    ) {
        let mut queue = self.cascade_queue.lock();
        for &hid in hyperedge_ids {
            queue.push_back((doc_id, hid));
        }
    }

    /// Returns the number of hyperedges pending in the in-memory cascade queue.
    pub fn pending_cascade_queue_len(&self) -> usize {
        self.cascade_queue.lock().len()
    }

    /// Drains and processes up to `batch_size` deferred hyperedges from the cascade queue.
    ///
    /// Tombstones each hyperedge and removes its key from persistent storage if present.
    pub async fn process_cascade_queue(&self, batch_size: usize, wal_tx: TxId) -> Result<usize> {
        let mut items = Vec::new();
        {
            let mut queue = self.cascade_queue.lock();
            let count = batch_size.min(queue.len());
            for _ in 0..count {
                if let Some(item) = queue.pop_front() {
                    items.push(item);
                }
            }
        }

        if items.is_empty() {
            return Ok(0);
        }

        let mut tombstoned_count = 0usize;
        for (doc_id, hid) in items {
            if self.tombstone_hyperedge(hid, wal_tx) {
                tombstoned_count += 1;
            }
            if let Some(storage) = self.storage() {
                let key = format!(
                    "{}{:016x}:{:016x}",
                    crate::cascade::CASCADE_QUEUE_PREFIX,
                    doc_id.inner(),
                    hid.inner()
                );
                let _ = storage.delete(wal_tx, key.as_bytes()).await;
            }
        }

        Ok(tombstoned_count)
    }

    /// Returns a detailed memory estimate of the graph (§6.3).
    pub fn estimate_memory(&self, presized: bool) -> MemoryEstimate {
        let inner = self.inner_read();
        (*inner).estimate_memory(presized)
    }

    /// Returns the estimated total memory usage of the graph in bytes based on allocation capacities (§6.3).
    pub fn estimate_memory_bytes(&self) -> usize {
        let inner = self.inner_read();
        (*inner).estimate_memory_bytes()
    }

    /// Returns the estimated peak memory usage during compaction (§6.3),
    /// combining residence total bytes with the private structural bytes of a presized rebuild.
    pub fn estimate_compaction_peak_bytes(&self) -> usize {
        let inner = self.inner_read();
        (*inner).estimate_compaction_peak_bytes()
    }

    /// Returns the optional source document ID stored at the given edge index in `source_doc_ids`.
    pub fn get_source_doc_id(&self, index: usize) -> Option<DocId> {
        let inner = self.inner_read();
        inner.source_doc_id_at(index)
    }

    /// Returns the optional source document ID from which the edge (from, to) was derived.
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
            let start = inner.offsets[from_idx];
            let end = inner.offsets[from_idx + 1];
            for j in start..end {
                if inner.targets.get(j) == Some(&to_idx) {
                    return inner.source_doc_id_at(j);
                }
            }
        }

        None
    }

    /// Atomically tombstones a list of edges with WAL sequence provenance (INV-GRAPH-PROV-1),
    /// returning newly tombstoned edges and affected node IDs.
    #[allow(clippy::type_complexity)]
    pub(crate) fn tombstone_edges_direct(
        &self,
        edges: &[(EntityId, EntityId)],
        wal_tx: TxId,
    ) -> Result<(usize, Vec<(EntityId, EntityId)>, Vec<EntityId>)> {
        let mut inner = self.inner_write();
        let mut newly_tombstoned = Vec::new();
        let mut affected_nodes_set = HashSet::new();

        for &(from_id, to_id) in edges {
            let from_idx = inner.id_map.get(&from_id).copied();
            let to_idx = inner.id_map.get(&to_id).copied();

            if let (Some(f_idx), Some(t_idx)) = (from_idx, to_idx) {
                if inner.tombstoned_edges.insert((f_idx, t_idx)) {
                    // Record WAL transaction invalidation provenance on pending edge payloads
                    if let Some(pending) = inner.pending_edges.get_mut(&f_idx) {
                        for edge in pending.iter_mut() {
                            if edge.target == t_idx {
                                edge.tx_valid_to = Some(wal_tx);
                            }
                        }
                    }
                    // Record WAL transaction invalidation provenance on compacted CSR arrays
                    if f_idx < inner.offsets.len() - 1 {
                        let start = inner.offsets[f_idx];
                        let end = inner.offsets[f_idx + 1];
                        for j in start..end {
                            if inner.targets.get(j) == Some(&t_idx) {
                                if let Some(tx_to) = inner.tx_valid_tos.get_mut(j) {
                                    *tx_to = wal_tx;
                                }
                            }
                        }
                    }
                    inner.is_dirty = true;
                    newly_tombstoned.push((from_id, to_id));
                    affected_nodes_set.insert(from_id);
                    affected_nodes_set.insert(to_id);
                }
            }
        }

        let mut affected_node_ids: Vec<EntityId> = affected_nodes_set.into_iter().collect();
        affected_node_ids.sort();

        Ok((newly_tombstoned.len(), newly_tombstoned, affected_node_ids))
    }

    /// Directly inserts a hyperedge into memory.
    /// Erstellt eine n-äre Hyperkante mit H2-Multi-Key-Locking unter Verwending von [`ConsolidationNodesGuard`].
    ///
    /// Sortiert alle beteiligten Entity-IDs deterministisch vor dem Erwerb/Mutation, um Deadlocks
    /// bei überlappenden Knotenmengen zu verhindern (H2 / §5.1.2).
    pub fn relate_n_ary(
        &self,
        id: crate::hyperedge::HyperEdgeId,
        predicate: crate::csr::EdgeType,
        participants: Vec<crate::hyperedge::RoleBinding>,
        weight: f32,
        doc_id: Option<DocId>,
    ) -> std::result::Result<crate::hyperedge::HyperEdge, GraphMutationError> {
        let entity_ids: Vec<EntityId> = participants.iter().map(|p| p.entity).collect();
        let guard = crate::hyperedge::ConsolidationNodesGuard::try_acquire(&entity_ids)?;

        let hyperedge = crate::hyperedge::HyperEdge::new(id, predicate, participants, weight)
            .with_source_doc_id(doc_id);
        hyperedge.validate()?;

        guard.with_entities(|_sorted_entities| {
            self.insert_hyperedge_direct(hyperedge.clone());
        });

        Ok(hyperedge)
    }

    /// Persists a hyperedge to storage under `__graph:hyperedge:` and secondary index `__graph:hyperedge_by_entity:`.
    pub async fn persist_hyperedge(&self, tx: TxId, hyperedge: &crate::hyperedge::HyperEdge) -> Result<()> {
        if let Some(storage) = self.storage() {
            let key = format!("{}{:016x}", crate::hyperedge::HYPEREDGE_PREFIX, hyperedge.id.inner());
            let value = hyperedge.serialize()?;
            storage.put(tx, key.as_bytes(), &value).await?;

            for participant in hyperedge.participants.iter() {
                let sec_key = format!(
                    "{}{:016x}:{:016x}",
                    crate::hyperedge::HYPEREDGE_BY_ENTITY_PREFIX,
                    participant.entity.inner(),
                    hyperedge.id.inner()
                );
                storage.put(tx, sec_key.as_bytes(), &[]).await?;
            }
        }
        Ok(())
    }

    pub fn insert_hyperedge_direct(&self, hyperedge: crate::hyperedge::HyperEdge) {
        let hyperedge_arc = Arc::new(hyperedge);
        let mut inner = self.inner_write();
        let hyperedge_id = hyperedge_arc.id;
        if let Some(doc_id) = hyperedge_arc.source_doc_id {
            inner
                .doc_to_hyperedges
                .entry(doc_id)
                .or_default()
                .insert(hyperedge_id);
        }
        for participant in hyperedge_arc.participants.iter() {
            inner
                .hyperedge_index
                .entry(participant.entity)
                .or_default()
                .insert(hyperedge_id);
        }
        inner.hyperedges.insert(hyperedge_id, hyperedge_arc);
    }

    /// Returns all non-tombstoned hyperedge IDs derived from `doc_id`.
    pub fn hyperedges_for_doc(&self, doc_id: DocId) -> Vec<crate::hyperedge::HyperEdgeId> {
        let inner = self.inner_read();
        inner
            .doc_to_hyperedges
            .get(&doc_id)
            .map(|set| {
                set.iter()
                    .copied()
                    .filter(|id| {
                        inner
                            .hyperedges
                            .get(id)
                            .is_some_and(|edge| edge.tx_valid_to.is_none())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Retrieves a non-tombstoned hyperedge by its ID if present.
    pub fn get_hyperedge(
        &self,
        id: crate::hyperedge::HyperEdgeId,
    ) -> Option<Arc<crate::hyperedge::HyperEdge>> {
        let inner = self.inner_read();
        inner
            .hyperedges
            .get(&id)
            .filter(|edge| edge.tx_valid_to.is_none())
            .cloned()
    }

    /// Returns all non-tombstoned hyperedge IDs associated with `entity_id`.
    pub fn hyperedges_for_entity(&self, entity_id: EntityId) -> Vec<crate::hyperedge::HyperEdgeId> {
        let inner = self.inner_read();
        inner
            .hyperedge_index
            .get(&entity_id)
            .map(|set| {
                set.iter()
                    .copied()
                    .filter(|id| {
                        inner
                            .hyperedges
                            .get(id)
                            .is_some_and(|edge| edge.tx_valid_to.is_none())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Atomically tombstones a hyperedge by setting `tx_valid_to = Some(wal_tx)` and removing
    /// it from `hyperedge_index` and `doc_to_hyperedges` in a single write lock.
    ///
    /// NOTE: Callers in `cascade.rs` will adjust to pass `wal_tx: memfuse_core::TxId` as part of
    /// parallel wave updates.
    ///
    /// Returns `true` if the hyperedge was found and newly tombstoned, or `false` if
    /// it was already tombstoned or does not exist.
    pub fn tombstone_hyperedge(&self, id: crate::hyperedge::HyperEdgeId, wal_tx: TxId) -> bool {
        let mut inner = self.inner_write();
        let inner_ptr = &mut *inner;
        let existing = match inner_ptr.hyperedges.get(&id) {
            Some(edge) => {
                if edge.tx_valid_to.is_some() {
                    return false;
                }
                edge.clone()
            }
            None => return false,
        };

        let mut updated = (*existing).clone();
        updated.tx_valid_to = Some(wal_tx);
        let updated_arc = Arc::new(updated);

        if let Some(doc_id) = updated_arc.source_doc_id {
            if let Some(set) = inner_ptr.doc_to_hyperedges.get_mut(&doc_id) {
                set.remove(&id);
                if set.is_empty() {
                    inner_ptr.doc_to_hyperedges.remove(&doc_id);
                }
            }
        }

        for participant in updated_arc.participants.iter() {
            if let Some(set) = inner_ptr.hyperedge_index.get_mut(&participant.entity) {
                set.remove(&id);
                if set.is_empty() {
                    inner_ptr.hyperedge_index.remove(&participant.entity);
                }
            }
        }

        inner_ptr.hyperedges.insert(id, updated_arc);

        true
    }

    /// Returns all edge IDs derived from the given source `DocId`.
    pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<(EntityId, EntityId)> {
        let inner = self.inner_read();
        let mut edges: HashSet<(EntityId, EntityId)> =
            inner.doc_to_edges.get(&doc_id).cloned().unwrap_or_default();
        for edge in self.doc_edge_index.edges_for_doc(doc_id) {
            edges.insert(edge);
        }
        edges.into_iter().collect()
    }

    /// Atomically inserts a hyperedge into the graph and updates the entity secondary index.
    pub fn insert_hyperedge(&self, edge: crate::hyperedge::HyperEdge) {
        self.insert_hyperedge_direct(edge);
    }

    /// Directly inserts an entity into the CSR graph without staging.
    pub fn insert_entity_direct(&self, entity: Entity) -> Result<()> {
        let mut inner = self.inner_write();
        let idx = inner.get_or_create_index(entity.id);
        if idx >= inner.entities.len() {
            inner.entities.resize(idx + 1, sentinel_entity());
        }
        inner.entities[idx] = entity;
        Ok(())
    }

    /// Inserts an edge directly into the CSR graph with bi-temporal validity, offloading compaction asynchronously if needed.
    #[allow(clippy::too_many_arguments)]
    pub async fn add_edge(
        self: &Arc<Self>,
        from: EntityId,
        to: EntityId,
        weight: f32,
        tx_valid_from: Option<TxId>,
        tx_valid_to: Option<TxId>,
        business_valid_from: Option<i64>,
        business_valid_to: Option<i64>,
        source_doc_id: Option<DocId>,
        predicate_hash: Option<[u8; 32]>,
        object_repr: Option<Vec<u8>>,
    ) -> Result<()> {
        if !weight.is_finite() || weight < 0.0 {
            return Err(MemFuseError::InvalidInput(format!(
                "Invalid edge weight {weight}: weight must be finite and non-negative"
            )));
        }

        // Consistency-Check wenn aktiviert
        if let Some(ref enforcer_lock) = self.consistency_enforcer {
            if let (Some(pred_hash), Some(obj)) = (predicate_hash, object_repr.as_ref()) {
                let assertion = EdgeAssertion {
                    subject: from.inner(),
                    predicate_hash: pred_hash,
                    object_repr: obj.clone(),
                };
                let mut enforcer = enforcer_lock.write();
                if let Some(pattern) = enforcer.check_before_insert(&assertion) {
                    if pattern.suppressed {
                        // Widerspruch unterdrückt — Einfügen blockiert
                        tracing::warn!(
                            suppression_count = pattern.contradiction_count,
                            "ConsistencyEnforcer: edge insertion suppressed by conflict pattern"
                        );
                        return Err(MemFuseError::PolicyViolation(
                            "Contradictory edge suppressed by consistency enforcer".to_string(),
                        ));
                    }
                    // Widerspruch erkannt aber noch nicht suppressed — loggen, trotzdem einfügen
                    tracing::warn!(
                        "ConsistencyEnforcer: contradictory edge detected (not yet suppressed)"
                    );
                }
            }
        }

        // Phase 1: Edge einfügen (Write-Lock kurz halten, kein I/O)
        let needs_compact = {
            let mut inner = self.inner_write();
            let from_idx = inner.get_or_create_index(from);
            let to_idx = inner.get_or_create_index(to);
            if let Some(doc_id) = source_doc_id {
                inner
                    .doc_to_edges
                    .entry(doc_id)
                    .or_default()
                    .insert((from, to));
            }
            inner
                .pending_edges
                .entry(from_idx)
                .or_default()
                .push(EdgePayload {
                    target: to_idx,
                    weight,
                    tx_valid_from,
                    tx_valid_to,
                    business_valid_from,
                    business_valid_to,
                    source_doc_id,
                });
            inner.pending_edge_count += 1;
            inner.is_dirty = true;
            inner.add_to_out_weight_sum(from_idx, weight);
            inner.pending_edge_count >= self.config.rebuild_threshold
        }; // Write-Lock freigegeben

        if let Some(doc_id) = source_doc_id {
            self.doc_edge_index.record(doc_id, (from, to));
        }

        // Phase 2: Compact außerhalb des Write-Locks (falls nötig)
        if needs_compact {
            // compact_async holt sich intern den Write-Lock in spawn_blocking
            self.compact_async().await?;
        }

        Ok(())
    }

    /// Directly inserts an edge into the CSR graph without staging.
    pub async fn insert_edge_direct(
        self: &Arc<Self>,
        from: EntityId,
        to: EntityId,
        weight: f32,
    ) -> Result<()> {
        self.add_edge(from, to, weight, None, None, None, None, None, None, None)
            .await
    }

    /// Directly inserts an edge with validity into the CSR graph without staging.
    ///
    /// # Weight Validation & Policy
    /// Edge weights represent relationship strengths or score-decay factors in graph traversal and PPR.
    /// Negative, infinite, or NaN weights can cause pruning failure in BFS traversal or invalid PageRank calculations.
    /// Therefore, edge weights MUST be finite and non-negative (`0.0 <= weight`).
    pub async fn insert_edge_direct_with_validity(
        self: &Arc<Self>,
        from: EntityId,
        to: EntityId,
        weight: f32,
        tx_valid_from: Option<TxId>,
        tx_valid_to: Option<TxId>,
    ) -> Result<()> {
        self.add_edge(
            from,
            to,
            weight,
            tx_valid_from,
            tx_valid_to,
            None,
            None,
            None,
            None,
            None,
        )
        .await
    }

    /// Directly inserts an edge with full bi-temporal validity into the CSR graph without staging.
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_edge_direct_with_bitemporal_validity(
        self: &Arc<Self>,
        from: EntityId,
        to: EntityId,
        weight: f32,
        tx_valid_from: Option<TxId>,
        tx_valid_to: Option<TxId>,
        business_valid_from: Option<i64>,
        business_valid_to: Option<i64>,
        source_doc_id: Option<DocId>,
    ) -> Result<()> {
        self.add_edge(
            from,
            to,
            weight,
            tx_valid_from,
            tx_valid_to,
            business_valid_from,
            business_valid_to,
            source_doc_id,
            None,
            None,
        )
        .await
    }

    /// Fügt eine Entity direkt ein (für load_from_storage).
    pub fn load_entity_direct(&self, entity: Entity) -> Result<()> {
        let mut inner = self.inner_write();
        let idx = inner.get_or_create_index(entity.id);
        while inner.entities.len() <= idx {
            inner.entities.push(sentinel_entity());
        }
        inner.entities[idx] = entity;
        Ok(())
    }

    /// Fügt eine Edge direkt in committed_staged / pending_edges ein (für load_from_storage).
    /// Umgeht das TX-Staging, da beim Laden alle Daten bereits committed sind.
    #[allow(clippy::too_many_arguments)]
    pub fn load_edge_direct(
        &self,
        from: EntityId,
        to: EntityId,
        weight: f32,
        tx_valid_from: Option<TxId>,
        tx_valid_to: Option<TxId>,
        business_valid_from: Option<i64>,
        business_valid_to: Option<i64>,
        source_doc_id: Option<DocId>,
    ) -> Result<()> {
        if !weight.is_finite() || weight < 0.0 {
            return Err(MemFuseError::InvalidInput(format!(
                "Invalid edge weight {weight}: weight must be finite and non-negative"
            )));
        }
        let mut inner = self.inner_write();
        let from_idx = inner.get_or_create_index(from);
        let to_idx = inner.get_or_create_index(to);
        if let Some(doc_id) = source_doc_id {
            inner
                .doc_to_edges
                .entry(doc_id)
                .or_default()
                .insert((from, to));
        }
        inner
            .pending_edges
            .entry(from_idx)
            .or_default()
            .push(EdgePayload {
                target: to_idx,
                weight,
                tx_valid_from,
                tx_valid_to,
                business_valid_from,
                business_valid_to,
                source_doc_id,
            });
        inner.pending_edge_count += 1;
        inner.is_dirty = true;
        inner.add_to_out_weight_sum(from_idx, weight);
        Ok(())
    }


}
