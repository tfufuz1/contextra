use std::sync::Arc;

use contextra_types::EntityId;

use super::graph_write::CsrGraph;

impl crate::path_rag::PathGraph for CsrGraph {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        let inner = self.inner_read();
        let node_idx = match inner.id_map.get(&node) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };
        if inner.entity_at(node_idx).is_none() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let offsets_len_minus_1 = inner.offsets.len().saturating_sub(1);
        if node_idx < offsets_len_minus_1 {
            let start_edge = *inner.offsets.get(node_idx).unwrap_or(&0);
            let next_idx = node_idx.saturating_add(1);
            let end_edge = *inner.offsets.get(next_idx).unwrap_or(&0);
            for edge_idx in start_edge..end_edge {
                let Some(&neighbor_idx) = inner.targets.get(edge_idx) else { continue };
                let weight = *inner.weights.get(edge_idx).unwrap_or(&1.0);
                if !inner.tombstoned_edges.contains(&(node_idx, neighbor_idx))
                    && inner.entity_at(neighbor_idx).is_some()
                {
                    if let Some(&id) = inner.reverse_map.get(neighbor_idx) {
                        if seen.insert(id) {
                            result.push((id, weight));
                        }
                    }
                }
            }
        }

        if let Some(pending) = inner.pending_edges.get(&node_idx) {
            for edge in pending {
                let neighbor_idx = edge.target;
                if !inner.tombstoned_edges.contains(&(node_idx, neighbor_idx))
                    && inner.entity_at(neighbor_idx).is_some()
                {
                    if let Some(&id) = inner.reverse_map.get(neighbor_idx) {
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
        let inner = self.inner_read();
        let target_idx = match inner.id_map.get(&node) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };
        if inner.entity_at(target_idx).is_none() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let num_nodes = inner.reverse_map.len();
        let offsets_len_minus_1 = inner.offsets.len().saturating_sub(1);

        for u_idx in 0..num_nodes {
            if inner.entity_at(u_idx).is_none() {
                continue;
            }
            let u_id = match inner.reverse_map.get(u_idx) {
                Some(&id) => id,
                None => continue,
            };

            if u_idx < offsets_len_minus_1 {
                let start_edge = *inner.offsets.get(u_idx).unwrap_or(&0);
                let next_idx = u_idx.saturating_add(1);
                let end_edge = *inner.offsets.get(next_idx).unwrap_or(&0);
                for edge_idx in start_edge..end_edge {
                    let weight = *inner.weights.get(edge_idx).unwrap_or(&1.0);
                    if inner.targets.get(edge_idx) == Some(&target_idx)
                        && !inner.tombstoned_edges.contains(&(u_idx, target_idx))
                        && seen.insert(u_id)
                    {
                        result.push((u_id, weight));
                    }
                }
            }

            if let Some(pending) = inner.pending_edges.get(&u_idx) {
                for edge in pending {
                    if edge.target == target_idx
                        && !inner.tombstoned_edges.contains(&(u_idx, target_idx))
                        && seen.insert(u_id)
                    {
                        result.push((u_id, edge.weight));
                    }
                }
            }
        }

        result
    }

    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<crate::hyperedge::HyperEdgeId> {
        self.hyperedges_for_entity(node)
    }

    fn get_hyperedge(
        &self,
        id: crate::hyperedge::HyperEdgeId,
    ) -> Option<Arc<crate::hyperedge::HyperEdge>> {
        self.get_hyperedge(id)
    }
}

impl crate::path_rag::PathGraph for &CsrGraph {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        (*self).neighbors_with_weights(node)
    }
    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        (*self).predecessors_with_weights(node)
    }
    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<crate::hyperedge::HyperEdgeId> {
        (*self).hyperedges_for_entity(node)
    }
    fn get_hyperedge(
        &self,
        id: crate::hyperedge::HyperEdgeId,
    ) -> Option<Arc<crate::hyperedge::HyperEdge>> {
        (*self).get_hyperedge(id)
    }
}

impl crate::path_rag::PathGraph for Arc<CsrGraph> {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.as_ref().neighbors_with_weights(node)
    }
    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.as_ref().predecessors_with_weights(node)
    }
    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<crate::hyperedge::HyperEdgeId> {
        self.as_ref().hyperedges_for_entity(node)
    }
    fn get_hyperedge(
        &self,
        id: crate::hyperedge::HyperEdgeId,
    ) -> Option<Arc<crate::hyperedge::HyperEdge>> {
        self.as_ref().get_hyperedge(id)
    }
}
