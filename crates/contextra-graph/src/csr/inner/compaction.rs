//! Speicherbereinigung und physische Kompaktierung von `GraphInner`.
//!
//! # Zweck
//! Entfernt durch Tombstones als gelöscht markierte Kanten und konsolidiert Array-Offsets.

use contextra_types::{DocId, EntityId, TxId};

use super::super::types::{EdgePayload, InternalIndex};
use super::GraphInner;

impl GraphInner {
// ANCHOR[contextra-graph:SRC-CSR-INNER-COMPACTION-001]
    pub fn get_or_create_index(&mut self, id: EntityId) -> InternalIndex {
        if let Some(&idx) = self.id_map.get(&id) {
            idx
        } else {
            let idx = self.reverse_map.len();
            self.id_map.insert(id, idx);
            self.reverse_map.push(id);
            self.out_weight_sums.push(0.0);
            idx
        }
    }

    #[inline]
    pub(crate) fn add_to_out_weight_sum(&mut self, node_idx: usize, weight: f32) {
        if weight > 0.0 {
            let num_nodes = self.reverse_map.len();
            if self.out_weight_sums.len() < num_nodes {
                self.out_weight_sums.resize(num_nodes, 0.0);
            }
            if node_idx < self.out_weight_sums.len() {
                self.out_weight_sums[node_idx] += weight;
            }
        }
    }

    pub(crate) fn recompute_node_out_weight_sum(&mut self, node_idx: usize) {
        let num_nodes = self.reverse_map.len();
        if node_idx >= num_nodes {
            return;
        }
        if self.out_weight_sums.len() < num_nodes {
            self.out_weight_sums.resize(num_nodes, 0.0);
        }
        if self.entity_at(node_idx).is_none() {
            self.out_weight_sums[node_idx] = 0.0;
            return;
        }

        let mut sum = 0.0f32;

        if node_idx < self.offsets.len() - 1 {
            let start = self.offsets.get(node_idx).copied().unwrap_or(0);
            let end = self.offsets.get(node_idx + 1).copied().unwrap_or(0);
            for j in start..end {
                if let (Some(&target), Some(&w)) = (self.targets.get(j), self.weights.get(j)) {
                    if !self.tombstoned_edges.contains(&(node_idx, target))
                        && self.entity_at(target).is_some()
                        && w > 0.0
                    {
                        sum += w;
                    }
                }
            }
        }

        if let Some(pending) = self.pending_edges.get(&node_idx) {
            for edge in pending {
                let target = edge.target;
                if !self.tombstoned_edges.contains(&(node_idx, target))
                    && self.entity_at(target).is_some()
                    && edge.weight > 0.0
                {
                    sum += edge.weight;
                }
            }
        }

        if let Some(slot) = self.out_weight_sums.get_mut(node_idx) {
            *slot = sum;
        }
    }

    fn collect_node_edges_for_compaction(&self, i: usize) -> Vec<EdgePayload> {
        let mut node_edges = Vec::new();

        let old_start = if i < self.offsets.len() - 1 {
            self.offsets.get(i).copied().unwrap_or(0)
        } else {
            0
        };
        let old_end = if i < self.offsets.len() - 1 {
            self.offsets.get(i + 1).copied().unwrap_or(0)
        } else {
            0
        };

        for j in old_start..old_end {
            if let Some(&target) = self.targets.get(j) {
                if !self.tombstoned_edges.contains(&(i, target)) {
                    if let Some(&w) = self.weights.get(j) {
                        node_edges.push(EdgePayload {
                            target,
                            weight: w,
                            tx_valid_from: self.tx_valid_from_at(j),
                            tx_valid_to: self.tx_valid_to_at(j),
                            business_valid_from: self.business_valid_from_at(j),
                            business_valid_to: self.business_valid_to_at(j),
                            source_doc_id: self.source_doc_id_at(j),
                        });
                    }
                }
            }
        }

        if let Some(staged) = self.pending_edges.get(&i) {
            for edge in staged {
                if !self.tombstoned_edges.contains(&(i, edge.target)) {
                    node_edges.push(edge.clone());
                }
            }
        }

        node_edges.sort_by_key(|e| e.target);
        node_edges
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_compacted_arrays(
        &mut self,
        new_offsets: Vec<usize>,
        new_targets: Vec<InternalIndex>,
        new_weights: Vec<f32>,
        new_tx_valid_froms: Vec<TxId>,
        new_tx_valid_tos: Vec<TxId>,
        new_business_valid_froms: Vec<i64>,
        new_business_valid_tos: Vec<i64>,
        new_source_doc_ids: Vec<DocId>,
        num_nodes: usize,
    ) {
        self.offsets = new_offsets;
        self.targets = new_targets;
        self.weights = new_weights;
        self.tx_valid_froms = new_tx_valid_froms;
        self.tx_valid_tos = new_tx_valid_tos;
        self.business_valid_froms = new_business_valid_froms;
        self.business_valid_tos = new_business_valid_tos;
        self.source_doc_ids = new_source_doc_ids;
        self.pending_edges.clear();
        self.tombstoned_edges.clear();
        self.pending_edge_count = 0;
        self.is_dirty = false;
        #[cfg(feature = "edge-reinforcement-learning")]
        self.edge_store.clear();

        self.out_weight_sums.resize(num_nodes, 0.0);
        for i in 0..num_nodes {
            if self.entity_at(i).is_none() {
                if let Some(slot) = self.out_weight_sums.get_mut(i) {
                    *slot = 0.0;
                }
                continue;
            }
            let start = self.offsets.get(i).copied().unwrap_or(0);
            let end = self.offsets.get(i + 1).copied().unwrap_or(0);
            let mut sum = 0.0f32;
            for j in start..end {
                if let (Some(&target), Some(&w)) = (self.targets.get(j), self.weights.get(j)) {
                    if self.entity_at(target).is_some() && w > 0.0 {
                        sum += w;
                    }
                }
            }
            if let Some(slot) = self.out_weight_sums.get_mut(i) {
                *slot = sum;
            }
        }
    }

// ANCHOR[contextra-graph:SRC-CSR-INNER-COMPACTION-002]
    pub fn compact(&mut self) {
        let num_nodes = self.reverse_map.len();

        if self.pending_edges.is_empty() && self.tombstoned_edges.is_empty() {
            while self.offsets.len() < num_nodes + 1 {
                let last = *self.offsets.last().unwrap_or(&0);
                self.offsets.push(last);
            }
            self.pending_edge_count = 0;
            self.is_dirty = false;
            #[cfg(feature = "edge-reinforcement-learning")]
            self.edge_store.clear();
            return;
        }

        let mut new_offsets = Vec::with_capacity(num_nodes + 1);
        let mut new_targets = Vec::with_capacity(self.targets.len() + self.pending_edge_count);
        let mut new_weights = Vec::with_capacity(self.weights.len() + self.pending_edge_count);
        let mut new_tx_valid_froms =
            Vec::with_capacity(self.tx_valid_froms.len() + self.pending_edge_count);
        let mut new_tx_valid_tos =
            Vec::with_capacity(self.tx_valid_tos.len() + self.pending_edge_count);
        let mut new_business_valid_froms =
            Vec::with_capacity(self.business_valid_froms.len() + self.pending_edge_count);
        let mut new_business_valid_tos =
            Vec::with_capacity(self.business_valid_tos.len() + self.pending_edge_count);
        let mut new_source_doc_ids =
            Vec::with_capacity(self.source_doc_ids.len() + self.pending_edge_count);

        let mut current_offset = 0;
        new_offsets.push(current_offset);

        for i in 0..num_nodes {
            let node_edges = self.collect_node_edges_for_compaction(i);

            for edge in node_edges {
                new_targets.push(edge.target);
                new_weights.push(edge.weight);
                new_tx_valid_froms.push(edge.tx_valid_from.unwrap_or(TxId::INVALID));
                new_tx_valid_tos.push(edge.tx_valid_to.unwrap_or(TxId::INVALID));
                new_business_valid_froms.push(edge.business_valid_from.unwrap_or(i64::MIN));
                new_business_valid_tos.push(edge.business_valid_to.unwrap_or(i64::MIN));
                new_source_doc_ids.push(edge.source_doc_id.unwrap_or(DocId::new(0)));
                current_offset += 1;
            }

            new_offsets.push(current_offset);
        }

        while new_offsets.len() < num_nodes + 1 {
            let last = *new_offsets.last().unwrap_or(&0);
            new_offsets.push(last);
        }

        self.apply_compacted_arrays(
            new_offsets,
            new_targets,
            new_weights,
            new_tx_valid_froms,
            new_tx_valid_tos,
            new_business_valid_froms,
            new_business_valid_tos,
            new_source_doc_ids,
            num_nodes,
        );
    }
}
