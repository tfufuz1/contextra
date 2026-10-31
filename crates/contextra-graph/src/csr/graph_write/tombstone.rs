//! Soft-Delete- und Tombstone-Verwaltung für Kanten und Hyperkanten.
//!
//! # Zweck
//! Markiert gelöschte Elemente mit Transaktionszeitstempeln zur Wahrung von MVCC-Sichtbarkeiten.

use super::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{EntityId, Result, TxId};
use std::collections::HashSet;

impl CsrGraph {
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
                    if let Some(pending) = inner.pending_edges.get_mut(&f_idx) {
                        for edge in pending.iter_mut() {
                            if edge.target == t_idx {
                                edge.tx_valid_to = Some(wal_tx);
                            }
                        }
                    }
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

// ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-TOMBSTONE-001]
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
        let updated_arc = std::sync::Arc::new(updated);

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
}
