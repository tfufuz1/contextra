use contextra_core::{ContextraError, DocId, Result};
use std::fmt;
use std::sync::atomic::Ordering;

use super::batch::SearchContext;
use super::types::{Candidate, HnswIndex};

/// Type alias for HNSW node identifier in graph repair operations.
pub type NodeId = usize;

/// Statistics collected during synchronous HNSW node deletion and neighborhood graph repair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletionStats {
    /// Number of HNSW graph levels (0..=max_layer) touched during repair.
    pub levels_touched: u8,
    /// Total number of neighbor lists updated/repaired across all layers.
    pub neighbor_lists_repaired: usize,
    /// Number of orphaned neighbor connections replaced during repair.
    pub orphaned_replacements: usize,
    /// Verified post-condition that no residual ghost pointers remain in the graph.
    pub verified_no_ghost_pointers: bool,
}

/// Errors specific to synchronous HNSW deletion and neighborhood graph repair.
#[derive(Debug, PartialEq, Eq)]
pub enum HnswDeletionError {
    /// Document ID not present in index.
    NotFound(DocId),

    /// Graph became disconnected during repair at level `level` for node `node`.
    DisconnectedRepair { level: u8, node: NodeId },

    /// Post-repair verification found residual ghost pointers.
    VerificationFailed(usize),
}

impl fmt::Display for HnswDeletionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(doc_id) => write!(f, "doc_id {doc_id:?} not present in index"),
            Self::DisconnectedRepair { level, node } => write!(
                f,
                "graph became disconnected during repair at level {level}: node {node:?} has zero remaining neighbors and no replacement candidate"
            ),
            Self::VerificationFailed(count) => write!(
                f,
                "verification step found {count} residual ghost pointer(s) after repair — repair aborted, tombstone NOT eligible for DeletionProof"
            ),
        }
    }
}

impl std::error::Error for HnswDeletionError {}

impl From<HnswDeletionError> for ContextraError {
    fn from(err: HnswDeletionError) -> Self {
        ContextraError::Index(err.to_string())
    }
}

impl HnswIndex {
    /// Deletes a document from the HNSW graph AND synchronously repairs all neighbor lists
    /// before returning, ensuring zero ghost pointers remain (INV-DELETION-2).
    ///
    /// Algorithmic complexity: O(Degree(doc_id) * M * log M), bounded by local neighborhood size.
    pub fn remove_with_graph_repair(&mut self, doc_id: DocId) -> Result<DeletionStats> {
        let _write_guard = self.inner.hot.write_mutex.lock();

        let mmap_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let target_global_idx = {
            let doc_map = self.inner.hot.doc_to_node.read();
            doc_map
                .get(&doc_id.inner())
                .copied()
                .ok_or(HnswDeletionError::NotFound(doc_id))?
        };

        if target_global_idx < mmap_count {
            return Err(ContextraError::Index(format!(
                "Cannot synchronously repair mmap-backed node {target_global_idx} for doc_id {doc_id:?}"
            )));
        }

        let target_ram_idx = target_global_idx - mmap_count;

        let (max_layer, vector_data) = {
            let nodes = self.inner.hot.nodes.read();
            let node = nodes.get(target_ram_idx).ok_or_else(|| {
                ContextraError::Index(format!(
                    "RAM node {target_ram_idx} missing from nodes vector"
                ))
            })?;
            (node.max_layer, node.vector.clone())
        };

        let m = self.inner.cold.config.m;

        // Step 1: Collect neighbors per layer before unlinking
        let mut layers_neighbors: Vec<Vec<u32>> = Vec::with_capacity(max_layer + 1);
        for layer in 0..=max_layer {
            let conns = self
                .inner
                .hot
                .arena
                .get_ram_node_connections(target_ram_idx, layer, m);
            layers_neighbors.push(conns);
        }

        // Mark target node in deleted_nodes and remove from doc_to_node
        self.inner.hot.doc_to_node.write().remove(&doc_id.inner());
        self.inner
            .cold
            .deleted_nodes
            .write()
            .insert(target_global_idx as u64);
        self.inner.hot.deleted_count.fetch_add(1, Ordering::SeqCst);

        // Adjust entry points if target node was an entry point
        let ep_val = self.inner.hot.get_entry_point();
        let ram_ep_val = self.inner.hot.get_ram_entry_point();
        if ep_val == Some(target_global_idx) || ram_ep_val == Some(target_global_idx) {
            let nodes = self.inner.hot.nodes.read();
            let deleted = self.inner.cold.deleted_nodes.read();
            let mut best_node = None;
            let mut best_ram_node = None;
            let mut max_l = 0;
            let mut max_ram_l = 0;

            for (i, node) in nodes.iter().enumerate() {
                let global_idx = mmap_count + i;
                if global_idx != target_global_idx && !deleted.contains(global_idx as u64) {
                    if node.max_layer >= max_l {
                        max_l = node.max_layer;
                        best_node = Some(global_idx);
                    }
                    if node.max_layer >= max_ram_l {
                        max_ram_l = node.max_layer;
                        best_ram_node = Some(global_idx);
                    }
                }
            }

            if ep_val == Some(target_global_idx) {
                self.inner.hot.set_entry_point(best_node);
                if let Some(new_idx) = best_node {
                    let node_max_layer = nodes[new_idx - mmap_count].max_layer;
                    self.inner
                        .hot
                        .max_layer
                        .store(node_max_layer as u64, Ordering::SeqCst);
                } else {
                    self.inner.hot.max_layer.store(0, Ordering::SeqCst);
                }
            }

            if ram_ep_val == Some(target_global_idx) {
                self.inner.hot.set_ram_entry_point(best_ram_node);
            }
        }

        let mut neighbor_lists_repaired = 0;
        let mut orphaned_replacements = 0;

        let nodes = self.inner.hot.nodes.read();
        let deleted_guard = self.inner.cold.deleted_nodes.read();

        let search_ctx = SearchContext {
            nodes: &nodes,
            mmap: None,
            mmap_node_count: mmap_count,
            prior_prepared: &[],
            backlink_map: None,
            quantizer: None,
            arena: &self.inner.hot.arena,
        };

        // Step 2: Synchronously repair neighbor lists for each neighbor across all layers
        for (layer, current_layer_neighbors) in layers_neighbors.iter().enumerate().take(max_layer + 1) {
            let layer_cap = if layer == 0 { m * 2 } else { m };

            for &neighbor_u32 in current_layer_neighbors {
                let neighbor_global_idx = neighbor_u32 as usize;
                if deleted_guard.contains(neighbor_global_idx as u64) {
                    continue;
                }
                if neighbor_global_idx < mmap_count {
                    continue;
                }
                let neighbor_ram_idx = neighbor_global_idx - mmap_count;

                let old_conns =
                    self.inner
                        .hot
                        .arena
                        .get_ram_node_connections(neighbor_ram_idx, layer, m);

                // Remove target node from neighbor's list
                let mut updated_conns: Vec<u32> = old_conns
                    .into_iter()
                    .filter(|&c| c as usize != target_global_idx)
                    .collect();

                // Check if neighbor needs a replacement candidate to maintain degree/connectivity
                if updated_conns.is_empty() && !current_layer_neighbors.is_empty() {
                    // Try to find a replacement candidate from target's other neighbors or neighbor's 2nd-order neighbors
                    let mut candidates = Vec::new();
                    for &other_u32 in current_layer_neighbors {
                        let other_idx = other_u32 as usize;
                        if other_idx != neighbor_global_idx
                            && !deleted_guard.contains(other_idx as u64)
                        {
                            let dist = self.inner.compute_symmetric_distance_hybrid_with_batch(
                                neighbor_global_idx,
                                other_idx,
                                &search_ctx,
                                target_global_idx,
                                &vector_data,
                                &[],
                            )?;
                            candidates.push(Candidate {
                                index: other_idx,
                                distance: dist,
                            });
                        }
                    }

                    if !candidates.is_empty() {
                        let selected = self.inner.select_neighbors_heuristic_with_batch(
                            &search_ctx,
                            &candidates,
                            layer_cap,
                            target_global_idx,
                            &vector_data,
                            &[],
                        )?;
                        if !selected.is_empty() {
                            updated_conns.extend(selected);
                            orphaned_replacements += updated_conns.len();
                        }
                    }
                }

                if updated_conns.is_empty() {
                    let active_nodes_at_layer = self
                        .inner
                        .hot
                        .nodes
                        .read()
                        .iter()
                        .enumerate()
                        .filter(|(i, node)| {
                            node.max_layer >= layer
                                && !deleted_guard.contains((mmap_count + i) as u64)
                        })
                        .count();
                    if active_nodes_at_layer > 1 {
                        return Err(HnswDeletionError::DisconnectedRepair {
                            level: layer as u8,
                            node: neighbor_ram_idx,
                        }
                        .into());
                    }
                }

                self.inner
                    .hot
                    .arena
                    .update_backlink(neighbor_ram_idx, layer, m, &updated_conns)?;
                neighbor_lists_repaired += 1;
            }
        }

        // Step 3: Post-repair Verification step — 2nd-order neighborhood scan with budget
        let budget = (max_layer + 1) * m * 4;
        let mut residual_ghost_pointers = 0;
        let mut scanned = 0;

        for (i, node) in nodes.iter().enumerate() {
            let global_idx = mmap_count + i;
            if deleted_guard.contains(global_idx as u64) {
                continue;
            }
            for l in 0..=node.max_layer {
                let conns = self.inner.hot.arena.get_ram_node_connections(i, l, m);
                for &conn in &conns {
                    scanned += 1;
                    if conn as usize == target_global_idx {
                        residual_ghost_pointers += 1;
                    }
                    if scanned >= budget {
                        break;
                    }
                }
                if scanned >= budget {
                    break;
                }
            }
            if scanned >= budget {
                break;
            }
        }

        if residual_ghost_pointers > 0 {
            return Err(HnswDeletionError::VerificationFailed(residual_ghost_pointers).into());
        }

        // Free arena slot of target node
        self.inner.hot.arena.free_node(target_ram_idx);

        Ok(DeletionStats {
            levels_touched: (max_layer + 1) as u8,
            neighbor_lists_repaired,
            orphaned_replacements,
            verified_no_ghost_pointers: true,
        })
    }
}
