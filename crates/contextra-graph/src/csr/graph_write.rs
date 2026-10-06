use arc_swap::ArcSwap;
use parking_lot::{Mutex, RwLock};
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use crate::consistency_enforcement::{ConsistencyEnforcer, EdgeAssertion};
use crate::error::GraphMutationError;
use contextra_ports::{GraphCollectionMutation, GraphIndex, StorageEngine};
use contextra_types::{ContextraError, DocId, Entity, EntityId, Result, TxId};

use super::inner::{sentinel_entity, GraphInner, InnerWriteGuard, MemoryEstimate};
use super::types::CsrGraphConfig;
pub type EdgePayload = super::types::EdgePayload;

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
    /// Optionaler Edge-Reinforcement-Buffer für co-occurrence & traversal feedback signals (F-03).
    #[cfg(feature = "edge-reinforcement-learning")]
    pub reinforcement_buffer: Arc<crate::edge_reinforcement_buffer::EdgeReinforcementBuffer>,
    /// Thread-safe string interner for hyperedge participant roles.
    pub role_interner: Arc<crate::hyperedge::RoleInterner>,
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
            #[cfg(feature = "edge-reinforcement-learning")]
            reinforcement_buffer: Arc::new(
                crate::edge_reinforcement_buffer::EdgeReinforcementBuffer::new(),
            ),
            role_interner: Arc::new(crate::hyperedge::RoleInterner::new()),
        }
    }

    /// Interns a role name and returns its assigned RoleId.
    pub fn intern_role(&self, name: &str) -> crate::hyperedge::RoleId {
        self.role_interner.get_or_intern(name)
    }

    /// Resolves a RoleId to an owned String.
    pub fn resolve_role_string(&self, id: crate::hyperedge::RoleId) -> Option<String> {
        self.role_interner.resolve_string(id)
    }

    /// Checks if a role name exists in the role interner.
    pub fn contains_role(&self, name: &str) -> bool {
        self.role_interner.contains_role(name)
    }

    /// Checks if a RoleId exists in the role interner.
    pub fn contains_role_id(&self, id: crate::hyperedge::RoleId) -> bool {
        self.role_interner.contains_id(id)
    }

    /// Appends a co-occurrence signal to the internal edge reinforcement buffer (F-03).
    #[cfg(feature = "edge-reinforcement-learning")]
    pub fn push_cooccurrence_signal(&self, from: EntityId, to: EntityId, co_activation: f32) {
        self.reinforcement_buffer
            .push_cooccurrence(from, to, co_activation);
    }

    /// Appends a traversal signal to the internal edge reinforcement buffer (F-03).
    #[cfg(feature = "edge-reinforcement-learning")]
    pub fn push_traversal_signal(&self, from: EntityId, to: EntityId, path_length: usize) {
        self.reinforcement_buffer
            .push_traversal(from, to, path_length);
    }

    /// Returns the counts of pending co-occurrence and traversal signals (F-03).
    #[cfg(feature = "edge-reinforcement-learning")]
    pub fn reinforcement_signal_counts(&self) -> (usize, usize) {
        (
            self.reinforcement_buffer.cooccurrence_count(),
            self.reinforcement_buffer.traversal_count(),
        )
    }

    /// Flushes buffered reinforcement signals to the CSR graph (F-03).
    #[cfg(feature = "edge-reinforcement-learning")]
    pub fn flush_reinforcement_buffer(
        &self,
        config: &crate::edge_reinforcement::EdgeReinforcementConfig,
    ) {
        self.reinforcement_buffer.flush_to_graph(self, config);
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
        let mut graph = Self::with_config(config);
        graph.storage = Some(storage);
        graph
    }

    /// Erstellt CsrGraph mit aktiviertem ConsistencyEnforcer für Widerspruchsprävention (F-04/ADR-073).
    pub fn with_consistency_enforcer(suppression_threshold: u32) -> Self {
        let mut graph = Self::new();
        graph.consistency_enforcer =
            Some(RwLock::new(ConsistencyEnforcer::new(suppression_threshold)));
        graph
    }

    /// Exposes active conflict patterns from the optional ConsistencyEnforcer.
    pub fn active_conflict_patterns(&self) -> Vec<crate::consistency_enforcement::ConflictPattern> {
        if let Some(ref enforcer_lock) = self.consistency_enforcer {
            let enforcer = enforcer_lock.read();
            enforcer.active_patterns().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Detects whether two edge assertions conflict using the provided ContradictionDetector.
    pub fn detect_contradiction<D: crate::consistency_enforcement::ContradictionDetector>(
        &self,
        detector: &D,
        a: &EdgeAssertion,
        b: &EdgeAssertion,
    ) -> bool {
        if let Some(ref enforcer_lock) = self.consistency_enforcer {
            let enforcer = enforcer_lock.read();
            enforcer.detect_contradiction(detector, a, b)
        } else {
            detector.conflicts(a, b)
        }
    }

    /// Retrieves a conflict pattern by hash if tracked by ConsistencyEnforcer.
    pub fn get_conflict_pattern(
        &self,
        pattern_hash: &[u8; 32],
    ) -> Option<crate::consistency_enforcement::ConflictPattern> {
        let enforcer_lock = self.consistency_enforcer.as_ref()?;
        let enforcer = enforcer_lock.read();
        enforcer.get_pattern(pattern_hash).cloned()
    }

    /// Checks if a pattern hash is suppressed by ConsistencyEnforcer.
    pub fn is_conflict_suppressed(&self, pattern_hash: [u8; 32]) -> bool {
        if let Some(ref enforcer_lock) = self.consistency_enforcer {
            let enforcer = enforcer_lock.read();
            enforcer.is_suppressed(pattern_hash)
        } else {
            false
        }
    }

    /// Suggests tombstone candidate edges for a conflict pattern using ConsistencyEnforcer.
    pub fn suggest_tombstone_candidates_for_pattern(
        &self,
        matching_edges: &[crate::consistency_enforcement::EdgeId],
    ) -> Vec<crate::consistency_enforcement::EdgeId> {
        if let Some(ref enforcer_lock) = self.consistency_enforcer {
            let enforcer = enforcer_lock.read();
            enforcer.suggest_tombstone_candidates(matching_edges)
        } else {
            matching_edges.to_vec()
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

    /// Entfernt ein Dokument aus dem Graph und tombstoniert alle Kanten, die ausschließlich
    /// von diesem Dokument belegt wurden (DSGVO-Kaskaden-Löschung).
    ///
    /// # Semantik (INV-GRAPH-PROV-1)
    /// (a) Das Dokument wird aus allen `source_doc_ids` der betroffenen Kanten im Herkunftsindex entfernt.
    /// (b) Kanten, deren `source_doc_ids` danach LEER sind, werden atomar tombstoniert (und persistent gelöscht).
    /// (c) Kanten mit weiteren Quell-Dokumenten bleiben bestehen.
    ///
    /// Gibt die Liste der tombstonierten Kanten `Vec<EdgeId>` zurück.
    pub async fn remove_doc(
        &self,
        doc_id: DocId,
        wal_tx: TxId,
    ) -> Result<Vec<crate::consistency_enforcement::EdgeId>> {
        let tombstone_candidates = self.doc_edge_index.remove_doc(doc_id);
        let tombstone_candidates =
            self.suggest_tombstone_candidates_for_pattern(&tombstone_candidates);

        {
            let mut inner = self.inner_write();
            inner.doc_to_edges.remove(&doc_id);
        }

        #[cfg(feature = "edge-reinforcement-learning")]
        {
            let (co_count, tr_count) = self.reinforcement_signal_counts();
            if co_count > 0 || tr_count > 0 {
                let default_config = crate::edge_reinforcement::EdgeReinforcementConfig::default();
                self.flush_reinforcement_buffer(&default_config);
            }
        }

        if self.pending_cascade_queue_len() > 0 {
            let _ = self.process_cascade_queue(100, wal_tx).await;
        }

        if tombstone_candidates.is_empty() {
            return Ok(Vec::new());
        }

        let (_count, newly_tombstoned_edges, _affected_nodes) =
            self.tombstone_edges_direct(&tombstone_candidates, wal_tx)?;

        if let Some(ref storage) = self.storage {
            for (from_id, to_id) in &newly_tombstoned_edges {
                let _ = self
                    .delete_edge_persistence(storage.as_ref(), wal_tx, from_id, to_id)
                    .await;
            }
        }

        Ok(newly_tombstoned_edges)
    }

    /// Read path contract for RCU snapshot isolation (IP-08):
    /// Returns a lock-free reference `arc_swap::Guard<Arc<GraphInner>>` to the current `GraphInner` snapshot.
    /// Readers never block on compaction or writer locks.
    ///
    /// # PPR Integration Contract
    /// Downstream PPR algorithms (e.g. `ppr.rs`) consume this snapshot directly via `inner_read()`.
    /// The returned `Arc<GraphInner>` snapshot is point-in-time immutable and guaranteed not to be mutated in-place.
    pub fn inner_read(&self) -> arc_swap::Guard<Arc<GraphInner>> {
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
    /// Tombstones each hyperedge and all its non-tombstoned ancestors in topological order,
    /// and removes corresponding queue keys from persistent storage.
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
            let mut visited = HashSet::new();
            let mut on_stack = HashSet::new();
            let mut closure_nodes = Vec::new();

            fn dfs_upward(
                graph: &CsrGraph,
                node: crate::hyperedge::HyperEdgeId,
                visited: &mut HashSet<crate::hyperedge::HyperEdgeId>,
                on_stack: &mut HashSet<crate::hyperedge::HyperEdgeId>,
                closure_nodes: &mut Vec<crate::hyperedge::HyperEdgeId>,
            ) {
                if !visited.insert(node) {
                    return;
                }
                on_stack.insert(node);
                closure_nodes.push(node);

                for parent in graph.parent_hyperedges_of(node) {
                    if on_stack.contains(&parent) {
                        tracing::warn!(
                            child = %node.inner(),
                            parent = %parent.inner(),
                            "Cycle detected in hyperedge child-parent graph during background cascade queue processing"
                        );
                    } else {
                        dfs_upward(graph, parent, visited, on_stack, closure_nodes);
                    }
                }

                on_stack.remove(&node);
            }

            dfs_upward(self, hid, &mut visited, &mut on_stack, &mut closure_nodes);

            let topo_order =
                crate::cascade::topological_sort_hyperedge_closure(self, &closure_nodes);

            for node in topo_order {
                if self.tombstone_hyperedge(node, wal_tx) {
                    tombstoned_count += 1;
                }
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
                    return self.get_source_doc_id(j);
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

    /// Creates and persists an n-ary hyperedge with storage persistence.
    pub async fn relate_n_ary_and_persist(
        &self,
        tx: TxId,
        id: crate::hyperedge::HyperEdgeId,
        predicate: crate::csr::EdgeType,
        participants: Vec<crate::hyperedge::RoleBinding>,
        weight: f32,
        doc_id: Option<DocId>,
    ) -> std::result::Result<crate::hyperedge::HyperEdge, GraphMutationError> {
        let hyperedge = self.relate_n_ary(id, predicate, participants, weight, doc_id)?;
        if let Err(err) = self.persist_hyperedge(tx, &hyperedge).await {
            tracing::warn!(error = %err, "Failed to persist hyperedge");
        }
        Ok(hyperedge)
    }

    /// Persists a hyperedge to storage under `__graph:hyperedge:` and secondary index `__graph:hyperedge_by_entity:`.
    pub async fn persist_hyperedge(
        &self,
        tx: TxId,
        hyperedge: &crate::hyperedge::HyperEdge,
    ) -> Result<()> {
        if let Some(storage) = self.storage() {
            let key = format!(
                "{}{:016x}",
                crate::hyperedge::HYPEREDGE_PREFIX,
                hyperedge.id.inner()
            );
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
        for &child_id in hyperedge_arc.child_edge_ids.iter() {
            inner
                .child_to_parents
                .entry(child_id)
                .or_default()
                .insert(hyperedge_id);
        }
        inner.hyperedges.insert(hyperedge_id, hyperedge_arc);
    }

    /// Returns all non-tombstoned parent hyperedge IDs that contain `child` in `child_edge_ids`.
    ///
    /// The returned list is sorted in ascending order by `HyperEdgeId`.
    pub fn parent_hyperedges_of(
        &self,
        child: crate::hyperedge::HyperEdgeId,
    ) -> Vec<crate::hyperedge::HyperEdgeId> {
        let inner = self.inner_read();
        let mut parents: Vec<_> = inner
            .child_to_parents
            .get(&child)
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
            .unwrap_or_default();
        parents.sort_unstable();
        parents
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

    /// Returns the maximum `HyperEdgeId` present in the current graph snapshot, or 0 if empty.
    pub fn max_hyperedge_id(&self) -> u64 {
        let inner = self.inner_read();
        inner
            .hyperedges
            .keys()
            .map(|id| id.inner())
            .max()
            .unwrap_or(0)
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
    /// NOTE: Callers in `cascade.rs` will adjust to pass `wal_tx: contextra_types::TxId` as part of
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

        for &child_id in updated_arc.child_edge_ids.iter() {
            if let Some(set) = inner_ptr.child_to_parents.get_mut(&child_id) {
                set.remove(&id);
                if set.is_empty() {
                    inner_ptr.child_to_parents.remove(&child_id);
                }
            }
        }

        inner_ptr.hyperedges.insert(id, updated_arc);

        true
    }

    /// Batched, atomic commit of hyperedge tombstoning and new superedge insertion.
    ///
    /// Ein Aufruf = ein `ArcSwap::store` = ein atomarer RCU-Publish für
    /// alle Tombstones und die neue Superkante gemeinsam.
    ///
    /// # Errors
    /// Returns `GraphMutationError::DuplicateHyperEdgeId` if any new edge ID collides with an existing
    /// hyperedge or another edge in the batch. No mutations are applied on error (all-or-nothing semantics).
    pub fn commit_super_edge_batch(
        &self,
        tombstone_ids: &[crate::hyperedge::HyperEdgeId],
        new_edges: Vec<crate::hyperedge::HyperEdge>,
        wal_tx: TxId,
    ) -> std::result::Result<(), GraphMutationError> {
        let mut inner = self.inner_write();
        let inner_ptr = &mut *inner;

        // Collision & Validation pre-check
        let mut seen_ids = HashSet::with_capacity(new_edges.len());
        for edge in &new_edges {
            if !seen_ids.insert(edge.id) || inner_ptr.hyperedges.contains_key(&edge.id) {
                return Err(GraphMutationError::DuplicateHyperEdgeId(edge.id));
            }
            edge.validate()?;
        }

        // Apply tombstone updates
        for &id in tombstone_ids {
            let existing = match inner_ptr.hyperedges.get(&id) {
                Some(edge) => {
                    if edge.tx_valid_to.is_some() {
                        continue;
                    }
                    edge.clone()
                }
                None => continue,
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

            for &child_id in updated_arc.child_edge_ids.iter() {
                if let Some(set) = inner_ptr.child_to_parents.get_mut(&child_id) {
                    set.remove(&id);
                    if set.is_empty() {
                        inner_ptr.child_to_parents.remove(&child_id);
                    }
                }
            }

            inner_ptr.hyperedges.insert(id, updated_arc);
        }

        // Apply new hyperedges
        for edge in new_edges {
            let hyperedge_arc = Arc::new(edge);
            let hyperedge_id = hyperedge_arc.id;
            if let Some(doc_id) = hyperedge_arc.source_doc_id {
                inner_ptr
                    .doc_to_hyperedges
                    .entry(doc_id)
                    .or_default()
                    .insert(hyperedge_id);
            }
            for participant in hyperedge_arc.participants.iter() {
                inner_ptr
                    .hyperedge_index
                    .entry(participant.entity)
                    .or_default()
                    .insert(hyperedge_id);
            }
            for &child_id in hyperedge_arc.child_edge_ids.iter() {
                inner_ptr
                    .child_to_parents
                    .entry(child_id)
                    .or_default()
                    .insert(hyperedge_id);
            }
            inner_ptr.hyperedges.insert(hyperedge_id, hyperedge_arc);
        }

        Ok(())
    }

    /// Batched, atomic commit of hyperedges with async storage persistence.
    pub async fn commit_super_edge_batch_async(
        &self,
        tombstone_ids: &[crate::hyperedge::HyperEdgeId],
        new_edges: Vec<crate::hyperedge::HyperEdge>,
        wal_tx: TxId,
    ) -> std::result::Result<(), GraphMutationError> {
        for edge in &new_edges {
            if let Err(err) = self.persist_hyperedge(wal_tx, edge).await {
                tracing::warn!(error = %err, "Failed to persist hyperedge during batch commit");
            }
        }
        self.commit_super_edge_batch(tombstone_ids, new_edges, wal_tx)
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
            return Err(ContextraError::InvalidInput(format!(
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
                        return Err(ContextraError::PolicyViolation(
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
            let prov = crate::provenance::EdgeProvenance::new(
                (from, to),
                vec![doc_id],
                tx_valid_from.unwrap_or_else(|| TxId::new(0)),
            );
            self.doc_edge_index.record_provenance(&prov);
        }

        #[cfg(feature = "edge-reinforcement-learning")]
        {
            self.push_cooccurrence_signal(from, to, weight);
            self.push_traversal_signal(from, to, 1);
        }

        // Phase 2: Compact außerhalb des Write-Locks (falls nötig)
        if needs_compact {
            // compact_async holt sich intern den Write-Lock in spawn_blocking
            self.compact_async().await?;
        }

        Ok(())
    }

    /// Auto-routes direct edge insertion to the appropriate validity-specialized helper.
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_edge_auto(
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
        if business_valid_from.is_some() || business_valid_to.is_some() || source_doc_id.is_some() {
            self.insert_edge_direct_with_bitemporal_validity(
                from,
                to,
                weight,
                tx_valid_from,
                tx_valid_to,
                business_valid_from,
                business_valid_to,
                source_doc_id,
            )
            .await
        } else if tx_valid_from.is_some() || tx_valid_to.is_some() {
            self.insert_edge_direct_with_validity(from, to, weight, tx_valid_from, tx_valid_to)
                .await
        } else {
            self.insert_edge_direct(from, to, weight).await
        }
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
            return Err(ContextraError::InvalidInput(format!(
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
            let prov = crate::provenance::EdgeProvenance::new(
                (from, to),
                vec![doc_id],
                tx_valid_from.unwrap_or_else(|| TxId::new(0)),
            );
            self.doc_edge_index.record_provenance(&prov);
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

impl GraphCollectionMutation for CsrGraph {
    fn relate_n_ary(
        &self,
        predicate_tag: u32,
        participants: &[contextra_ports::graph::RoleBinding],
        doc_id: DocId,
    ) -> std::result::Result<
        contextra_ports::graph::HyperEdgeId,
        contextra_ports::graph::GraphMutationError,
    > {
        let next_id = crate::hyperedge::HyperEdgeId::new(self.max_hyperedge_id() + 1);
        let mapped_participants: Vec<crate::hyperedge::RoleBinding> = participants
            .iter()
            .map(|p| {
                crate::hyperedge::RoleBinding::new(
                    crate::hyperedge::RoleId::new(p.role.inner()),
                    p.entity,
                )
            })
            .collect();

        let _ = predicate_tag;
        let hyperedge = self
            .relate_n_ary(
                next_id,
                crate::csr::EdgeType::Default,
                mapped_participants,
                1.0,
                Some(doc_id),
            )
            .map_err(|err| match err {
                GraphMutationError::LockAcquisitionTimeout(msg) => {
                    contextra_ports::graph::GraphMutationError::LockAcquisitionTimeout(msg)
                }
                GraphMutationError::RoleBindingInvalid(msg) => {
                    contextra_ports::graph::GraphMutationError::RoleBindingInvalid(msg)
                }
                GraphMutationError::EpochReclamationPending(epoch) => {
                    contextra_ports::graph::GraphMutationError::EpochReclamationPending(epoch)
                }
                GraphMutationError::InsufficientParticipants { expected, found } => {
                    contextra_ports::graph::GraphMutationError::InsufficientParticipants {
                        expected,
                        found,
                    }
                }
                GraphMutationError::HyperedgeNotFound(id) => {
                    contextra_ports::graph::GraphMutationError::HyperedgeNotFound(id)
                }
                GraphMutationError::DuplicateHyperEdgeId(id) => {
                    contextra_ports::graph::GraphMutationError::Internal(format!(
                        "Duplicate hyperedge ID detected during batch commit: {id}"
                    ))
                }
                GraphMutationError::InvalidWeight { weight, reason } => {
                    contextra_ports::graph::GraphMutationError::InvalidWeight { weight, reason }
                }
                GraphMutationError::Internal(msg) => {
                    contextra_ports::graph::GraphMutationError::Internal(msg)
                }
            })?;

        Ok(contextra_ports::graph::HyperEdgeId::new(
            hyperedge.id.inner(),
        ))
    }
}
