use super::CsrGraph;
use crate::error::GraphMutationError;
use contextra_types::{DocId, EntityId, Result, TxId};
use std::collections::HashSet;
use std::sync::Arc;

fn dfs_upward_cascade(
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
            dfs_upward_cascade(graph, parent, visited, on_stack, closure_nodes);
        }
    }

    on_stack.remove(&node);
}

impl CsrGraph {
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

    pub fn pending_cascade_queue_len(&self) -> usize {
        self.cascade_queue.lock().len()
    }

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

            dfs_upward_cascade(self, hid, &mut visited, &mut on_stack, &mut closure_nodes);

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

    pub fn max_hyperedge_id(&self) -> u64 {
        let inner = self.inner_read();
        inner
            .hyperedges
            .keys()
            .map(|id| id.inner())
            .max()
            .unwrap_or(0)
    }

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

    fn apply_batch_tombstones(
        inner_ptr: &mut crate::csr::GraphInner,
        tombstone_ids: &[crate::hyperedge::HyperEdgeId],
        wal_tx: TxId,
    ) {
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
    }

    fn apply_batch_new_edges(
        inner_ptr: &mut crate::csr::GraphInner,
        new_edges: Vec<crate::hyperedge::HyperEdge>,
    ) {
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
    }

    pub fn commit_super_edge_batch(
        &self,
        tombstone_ids: &[crate::hyperedge::HyperEdgeId],
        new_edges: Vec<crate::hyperedge::HyperEdge>,
        wal_tx: TxId,
    ) -> std::result::Result<(), GraphMutationError> {
        let mut inner = self.inner_write();
        let inner_ptr = &mut *inner;

        let mut seen_ids = HashSet::with_capacity(new_edges.len());
        for edge in &new_edges {
            if !seen_ids.insert(edge.id) || inner_ptr.hyperedges.contains_key(&edge.id) {
                return Err(GraphMutationError::Internal(format!(
                    "Duplicate hyperedge ID {}",
                    edge.id
                )));
            }
            edge.validate()?;
        }

        Self::apply_batch_tombstones(inner_ptr, tombstone_ids, wal_tx);
        Self::apply_batch_new_edges(inner_ptr, new_edges);

        Ok(())
    }

    pub fn insert_hyperedge(&self, edge: crate::hyperedge::HyperEdge) {
        self.insert_hyperedge_direct(edge);
    }
}
