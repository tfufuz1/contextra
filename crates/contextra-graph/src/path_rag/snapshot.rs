//! Snapshot-isolated PathGraph implementation over `GraphInner`.

// FILE-CONTEXT
// STAND: 2026-09-28
// ZWECK: Multi-Version Concurrency Control (MVCC) snapshot-isolated PathGraph for PathRAG.
// REFERENZ: v17 Teil 6.3 ("PathRAG-Lücke") & Teil 12 (Snapshot-Isolation-Prinzip 6).
// INVARIANTEN: Zero-allocation view over `GraphInner`; Panic-free execution; Strict visibility rules matching `multi_traverse_at`.

use std::collections::HashSet;
use std::sync::Arc;

use contextra_types::{EntityId, TxId};

use crate::csr::visibility::is_edge_visible_bitemporal;
use crate::csr::GraphInner;
use crate::hyperedge::{HyperEdge, HyperEdgeId};
use crate::path_rag::PathGraph;

/// Snapshot-isolated wrapper around `GraphInner` at a specific transaction sequence number (`as_of_tx`).
pub struct SnapshotPathGraph<'a> {
    inner: &'a GraphInner,
    as_of_tx: TxId,
}

impl<'a> SnapshotPathGraph<'a> {
    /// Creates a new `SnapshotPathGraph` borrowing `inner` at transaction `as_of_tx`.
    pub fn new(inner: &'a GraphInner, as_of_tx: TxId) -> Self {
        Self { inner, as_of_tx }
    }
}

impl<'a> PathGraph for SnapshotPathGraph<'a> {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        let node_idx = match self.inner.id_map.get(&node) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };
        if self.inner.entity_at(node_idx).is_none() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut seen = HashSet::new();

        // 1. Compacted CSR edges
        let offsets_len_minus_1 = self.inner.offsets.len().saturating_sub(1);
        if node_idx < offsets_len_minus_1 {
            let start_edge = *self.inner.offsets.get(node_idx).unwrap_or(&0);
            let next_i = node_idx.saturating_add(1);
            let end_edge = *self.inner.offsets.get(next_i).unwrap_or(&0);
            for edge_idx in start_edge..end_edge {
                let Some(&neighbor_idx) = self.inner.targets.get(edge_idx) else { continue };
                if self
                    .inner
                    .tombstoned_edges
                    .contains(&(node_idx, neighbor_idx))
                    || self.inner.entity_at(neighbor_idx).is_none()
                {
                    continue;
                }

                let tx_vf = self.inner.tx_valid_from_at(edge_idx);
                let tx_vt = self.inner.tx_valid_to_at(edge_idx);
                let biz_vf = self.inner.business_valid_from_at(edge_idx);
                let biz_vt = self.inner.business_valid_to_at(edge_idx);

                if is_edge_visible_bitemporal(tx_vf, tx_vt, self.as_of_tx, biz_vf, biz_vt, None) {
                    if let Some(&id) = self.inner.reverse_map.get(neighbor_idx) {
                        if seen.insert(id) {
                            let weight = *self.inner.weights.get(edge_idx).unwrap_or(&1.0);
                            result.push((id, weight));
                        }
                    }
                }
            }
        }

        // 2. Delta buffer pending edges
        if let Some(pending) = self.inner.pending_edges.get(&node_idx) {
            for edge in pending {
                let neighbor_idx = edge.target;
                if self
                    .inner
                    .tombstoned_edges
                    .contains(&(node_idx, neighbor_idx))
                    || self.inner.entity_at(neighbor_idx).is_none()
                {
                    continue;
                }

                if is_edge_visible_bitemporal(
                    edge.tx_valid_from,
                    edge.tx_valid_to,
                    self.as_of_tx,
                    edge.business_valid_from,
                    edge.business_valid_to,
                    None,
                ) {
                    if let Some(&id) = self.inner.reverse_map.get(neighbor_idx) {
                        if seen.insert(id) {
                            result.push((id, edge.weight));
                        }
                    }
                }
            }
        }

        result
    }

    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        let target_idx = match self.inner.id_map.get(&node) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };
        if self.inner.entity_at(target_idx).is_none() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut seen = HashSet::new();
        let num_nodes = self.inner.reverse_map.len();

        for u_idx in 0..num_nodes {
            if self.inner.entity_at(u_idx).is_none() {
                continue;
            }
            let u_id = match self.inner.reverse_map.get(u_idx) {
                Some(&id) => id,
                None => continue,
            };

            // 1. Compacted CSR edges
            let offsets_len_minus_1 = self.inner.offsets.len().saturating_sub(1);
            if u_idx < offsets_len_minus_1 {
                let start_edge = *self.inner.offsets.get(u_idx).unwrap_or(&0);
                let next_i = u_idx.saturating_add(1);
                let end_edge = *self.inner.offsets.get(next_i).unwrap_or(&0);
                for edge_idx in start_edge..end_edge {
                    if self.inner.targets.get(edge_idx) == Some(&target_idx)
                        && !self.inner.tombstoned_edges.contains(&(u_idx, target_idx))
                    {
                        let tx_vf = self.inner.tx_valid_from_at(edge_idx);
                        let tx_vt = self.inner.tx_valid_to_at(edge_idx);
                        let biz_vf = self.inner.business_valid_from_at(edge_idx);
                        let biz_vt = self.inner.business_valid_to_at(edge_idx);

                        if is_edge_visible_bitemporal(
                            tx_vf,
                            tx_vt,
                            self.as_of_tx,
                            biz_vf,
                            biz_vt,
                            None,
                        ) && seen.insert(u_id)
                        {
                            let weight = *self.inner.weights.get(edge_idx).unwrap_or(&1.0);
                            result.push((u_id, weight));
                        }
                    }
                }
            }

            // 2. Delta buffer pending edges
            if let Some(pending) = self.inner.pending_edges.get(&u_idx) {
                for edge in pending {
                    if edge.target == target_idx
                        && !self.inner.tombstoned_edges.contains(&(u_idx, target_idx))
                        && is_edge_visible_bitemporal(
                            edge.tx_valid_from,
                            edge.tx_valid_to,
                            self.as_of_tx,
                            edge.business_valid_from,
                            edge.business_valid_to,
                            None,
                        )
                        && seen.insert(u_id)
                    {
                        result.push((u_id, edge.weight));
                    }
                }
            }
        }

        result
    }

    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<HyperEdgeId> {
        let hedge_ids = match self.inner.hyperedge_index.get(&node) {
            Some(set) => set,
            None => return Vec::new(),
        };

        let mut visible = Vec::new();
        for &hedge_id in hedge_ids {
            if let Some(hedge) = self.inner.hyperedges.get(&hedge_id) {
                if is_edge_visible_bitemporal(
                    hedge.tx_valid_from,
                    hedge.tx_valid_to,
                    self.as_of_tx,
                    hedge.business_valid_from,
                    hedge.business_valid_to,
                    None,
                ) {
                    visible.push(hedge_id);
                }
            }
        }
        visible
    }

    fn get_hyperedge(&self, id: HyperEdgeId) -> Option<Arc<HyperEdge>> {
        let hedge = self.inner.hyperedges.get(&id)?;
        if is_edge_visible_bitemporal(
            hedge.tx_valid_from,
            hedge.tx_valid_to,
            self.as_of_tx,
            hedge.business_valid_from,
            hedge.business_valid_to,
            None,
        ) {
            Some(hedge.clone())
        } else {
            None
        }
    }
}
