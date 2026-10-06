// FILE-CONTEXT
// STAND: 2026-10-06
// ZWECK: Synchronous ghost-free node deletion with targeted & complete neighborhood verification for HNSW graph.
// INVARIANTEN: INV-DELETION-2: After Ok(..) of remove_with_graph_repair(doc_id), no neighborhood pointer points to doc_id.
// Lock hierarchy: write_mutex -> entry_point -> nodes / doc_to_node / deleted_nodes.

use ahash::AHashSet;
use contextra_core::{error::HnswDeletionError, ContextraError, DocId, Result};

use super::batch::SearchContext;
use super::types::{Candidate, HnswIndex};

/// Statistics returned after a ghost-free node deletion and neighborhood repair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletionStats {
    /// Document ID that was deleted.
    pub doc_id: DocId,
    /// Number of neighborhood edges repaired (backlinks removed/replaced).
    pub repaired_edges: usize,
    /// Number of replacement attempts where no valid replacement candidate existed.
    pub orphaned_replacements: usize,
    /// True if post-repair verification confirmed zero ghost pointers remain to doc_id.
    pub verified_no_ghost_pointers: bool,
}

/// Result of ghost pointer verification scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerificationResult {
    /// Number of residual ghost pointers that could not be removed.
    pub remaining_ghost_pointers: usize,
    /// Number of ghost pointers found during verification and successfully repaired.
    pub repaired_ghost_pointers: usize,
    /// True if all candidate/index nodes were verified completely.
    pub is_complete: bool,
}

/// Trait providing ghost-free vector index deletion with synchronous neighborhood graph repair.
pub trait GhostFreeVectorIndex {
    /// Deletes `doc_id` and synchronously repairs all neighborhood pointers across all layers
    /// before returning.
    ///
    /// # Invariant
    /// `INV-DELETION-2`: After `Ok(stats)` returns, no neighborhood pointer in the entire index
    /// references `doc_id`.
    fn remove_with_graph_repair(&mut self, doc_id: DocId) -> Result<DeletionStats>;
}

impl GhostFreeVectorIndex for HnswIndex {
    fn remove_with_graph_repair(&mut self, doc_id: DocId) -> Result<DeletionStats> {
        // Lock hierarchy: 1. write_mutex
        let _write_lock = self.inner.hot.write_mutex.lock();

        let mmap_node_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        // Find target node index from doc_to_node
        let target_idx = {
            let doc_map = self.inner.hot.doc_to_node.read();
            match doc_map.get(&doc_id.inner()).copied() {
                Some(idx) => idx,
                None => {
                    return Err(ContextraError::GraphRepairFailed(
                        HnswDeletionError::NodeNotFound(doc_id),
                    ))
                }
            }
        };

        // Check if already tombstoned/deleted
        if self
            .inner
            .cold
            .deleted_nodes
            .read()
            .contains(target_idx as u64)
        {
            return Err(ContextraError::GraphRepairFailed(
                HnswDeletionError::NodeNotFound(doc_id),
            ));
        }

        let m = self.inner.cold.config.m;

        // Determine target_max_layer
        let target_max_layer = {
            let nodes_read = self.inner.hot.nodes.read();
            if target_idx < mmap_node_count {
                let mmap_guard = self.inner.cold.mmap_index.read();
                if let Some(mmap) = mmap_guard.as_ref() {
                    mmap.get_node_record(target_idx)
                        .map(|r| r.max_layer as usize)
                        .unwrap_or(0)
                } else {
                    0
                }
            } else {
                let ram_idx = target_idx - mmap_node_count;
                if ram_idx < nodes_read.len() {
                    nodes_read[ram_idx].max_layer
                } else {
                    0
                }
            }
        };

        let mut repaired_edges = 0usize;
        let mut orphaned_replacements = 0usize;
        let mut targeted_nodes = AHashSet::new();

        // Include target's own direct neighbors across all layers into targeted verification set
        {
            let nodes_read = self.inner.hot.nodes.read();
            let mmap_guard = self.inner.cold.mmap_index.read();

            for layer in 0..=target_max_layer {
                let target_conns = if target_idx < mmap_node_count {
                    if let Some(mmap) = mmap_guard.as_ref() {
                        if let Ok(rec) = mmap.get_node_record(target_idx) {
                            mmap.get_connections(&rec, layer).unwrap_or_default()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    let ram_idx = target_idx - mmap_node_count;
                    if ram_idx < nodes_read.len() {
                        self.inner
                            .hot
                            .arena
                            .get_ram_node_connections(ram_idx, layer, m)
                    } else {
                        Vec::new()
                    }
                };

                for &c in &target_conns {
                    targeted_nodes.insert(c as usize);
                }
            }
        }

        // Perform neighborhood repairs across layers
        {
            let nodes_read = self.inner.hot.nodes.read();
            let mmap_guard = self.inner.cold.mmap_index.read();
            let total_nodes = mmap_node_count + nodes_read.len();

            let ctx = SearchContext {
                nodes: &nodes_read,
                mmap: mmap_guard.as_ref(),
                mmap_node_count,
                prior_prepared: &[],
                backlink_map: None,
                quantizer: None,
                arena: &self.inner.hot.arena,
            };

            for layer in 0..=target_max_layer {
                let layer_max_m = if layer == 0 { m * 2 } else { m };

                // Find all nodes in graph that have a connection pointing to target_idx at this layer
                for neighbor_idx in 0..total_nodes {
                    if neighbor_idx == target_idx {
                        continue;
                    }

                    let existing_conns = if neighbor_idx < mmap_node_count {
                        if let Some(mmap) = mmap_guard.as_ref() {
                            if let Ok(rec) = mmap.get_node_record(neighbor_idx) {
                                mmap.get_connections(&rec, layer).unwrap_or_default()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        }
                    } else {
                        let neighbor_ram_idx = neighbor_idx - mmap_node_count;
                        self.inner
                            .hot
                            .arena
                            .get_ram_node_connections(neighbor_ram_idx, layer, m)
                    };

                    // Remove backlink to target_idx
                    if existing_conns.contains(&(target_idx as u32)) {
                        repaired_edges += 1;
                        targeted_nodes.insert(neighbor_idx);
                        let neighbor_u32 = neighbor_idx as u32;

                        let remaining: Vec<u32> = existing_conns
                            .into_iter()
                            .filter(|&c| c != target_idx as u32)
                            .collect();

                        // Collect 2nd-order candidates from remaining neighbors' connections
                        let mut candidate_set: AHashSet<u32> = AHashSet::new();
                        for &rem in &remaining {
                            if rem != target_idx as u32 && rem != neighbor_u32 {
                                candidate_set.insert(rem);
                                targeted_nodes.insert(rem as usize);
                            }
                            let rem_conns = if (rem as usize) < mmap_node_count {
                                if let Some(mmap) = mmap_guard.as_ref() {
                                    if let Ok(rec) = mmap.get_node_record(rem as usize) {
                                        mmap.get_connections(&rec, layer).unwrap_or_default()
                                    } else {
                                        Vec::new()
                                    }
                                } else {
                                    Vec::new()
                                }
                            } else {
                                let rem_ram_idx = (rem as usize) - mmap_node_count;
                                self.inner
                                    .hot
                                    .arena
                                    .get_ram_node_connections(rem_ram_idx, layer, m)
                            };

                            for &c_u32 in &rem_conns {
                                if c_u32 != target_idx as u32
                                    && c_u32 != neighbor_u32
                                    && !self.inner.cold.deleted_nodes.read().contains(c_u32 as u64)
                                {
                                    candidate_set.insert(c_u32);
                                    targeted_nodes.insert(c_u32 as usize);
                                }
                            }
                        }

                        if candidate_set.is_empty() {
                            orphaned_replacements += 1;
                        }

                        // Compute candidates distances to neighbor_idx
                        let mut cand_objs = Vec::with_capacity(candidate_set.len());
                        for &cand_u32 in &candidate_set {
                            let cand_idx = cand_u32 as usize;
                            let dummy_vec = super::types::VectorData::F32(Vec::new());
                            let dist = self
                                .inner
                                .compute_symmetric_distance_hybrid_with_batch(
                                    neighbor_idx,
                                    cand_idx,
                                    &ctx,
                                    usize::MAX,
                                    &dummy_vec,
                                    &[],
                                )
                                .unwrap_or(f32::MAX);

                            cand_objs.push(Candidate {
                                index: cand_idx,
                                distance: dist,
                            });
                        }

                        // Use existing heuristic neighbor selection
                        let dummy_vec = super::types::VectorData::F32(Vec::new());
                        let updated_conns = if !cand_objs.is_empty() {
                            self.inner
                                .select_neighbors_heuristic_with_batch(
                                    &ctx,
                                    &cand_objs,
                                    layer_max_m,
                                    usize::MAX,
                                    &dummy_vec,
                                    &[],
                                )
                                .unwrap_or(remaining.clone())
                        } else {
                            remaining.clone()
                        };

                        for &u in &updated_conns {
                            targeted_nodes.insert(u as usize);
                        }

                        // Apply updated connections if neighbor is a RAM node
                        if neighbor_idx >= mmap_node_count {
                            let neighbor_ram_idx = neighbor_idx - mmap_node_count;
                            let _ = self.inner.hot.arena.update_backlink(
                                neighbor_ram_idx,
                                layer,
                                m,
                                &updated_conns,
                            );
                        }
                    }
                }
            }
        }

        // Verification scan over targeted neighborhood or full fallback
        // Cost cap budget = (degree of deleted node) * (M) * 4
        let total_degree = target_max_layer * m;
        let budget = (total_degree.max(1)) * m * 4;
        let verification_res =
            self.verify_no_ghost_pointers(target_idx as u32, budget, Some(&targeted_nodes));

        repaired_edges += verification_res.repaired_ghost_pointers;

        if verification_res.remaining_ghost_pointers > 0 || !verification_res.is_complete {
            return Err(ContextraError::GraphRepairFailed(
                HnswDeletionError::VerificationFailed {
                    remaining_pointers: verification_res.remaining_ghost_pointers,
                },
            ));
        }

        // Mark doc_id as deleted in HNSW structure (updates doc_to_node, deleted_nodes, entry_point)
        self.inner.do_delete(doc_id)?;

        Ok(DeletionStats {
            doc_id,
            repaired_edges,
            orphaned_replacements,
            verified_no_ghost_pointers: true,
        })
    }
}

impl HnswIndex {
    /// Internal helper method to perform a verification scan for residual ghost pointers
    /// pointing to `target_idx_u32`.
    ///
    /// If `targeted_nodes` is provided and its size is within `budget`, inspects all targeted
    /// 1st and 2nd order neighbor nodes. If `targeted_nodes` is `None` or its size exceeds `budget`,
    /// performs a complete fallback scan over all `0..total_nodes`.
    pub fn verify_no_ghost_pointers(
        &self,
        target_idx_u32: u32,
        budget: usize,
        targeted_nodes: Option<&AHashSet<usize>>,
    ) -> VerificationResult {
        let mmap_node_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let nodes_read = self.inner.hot.nodes.read();
        let mmap_guard = self.inner.cold.mmap_index.read();
        let deleted = self.inner.cold.deleted_nodes.read();
        let m = self.inner.cold.config.m;

        let total_nodes = mmap_node_count + nodes_read.len();

        let mut remaining_ghost_pointers = 0usize;
        let mut repaired_ghost_pointers = 0usize;

        let execute_targeted = if let Some(nodes_to_check) = targeted_nodes {
            nodes_to_check.len() <= budget
        } else {
            false
        };

        if execute_targeted {
            if let Some(nodes_to_check) = targeted_nodes {
                for &i in nodes_to_check {
                    if i >= total_nodes
                        || deleted.contains(i as u64)
                        || i == target_idx_u32 as usize
                    {
                        continue;
                    }

                    let max_layer = if i < mmap_node_count {
                        if let Some(mmap) = mmap_guard.as_ref() {
                            mmap.get_node_record(i)
                                .map(|r| r.max_layer as usize)
                                .unwrap_or(0)
                        } else {
                            0
                        }
                    } else {
                        let ram_idx = i - mmap_node_count;
                        nodes_read.get(ram_idx).map(|n| n.max_layer).unwrap_or(0)
                    };

                    for layer in 0..=max_layer {
                        let conns = if i < mmap_node_count {
                            if let Some(mmap) = mmap_guard.as_ref() {
                                if let Ok(rec) = mmap.get_node_record(i) {
                                    mmap.get_connections(&rec, layer).unwrap_or_default()
                                } else {
                                    Vec::new()
                                }
                            } else {
                                Vec::new()
                            }
                        } else {
                            let ram_idx = i - mmap_node_count;
                            self.inner
                                .hot
                                .arena
                                .get_ram_node_connections(ram_idx, layer, m)
                        };

                        if conns.contains(&target_idx_u32) {
                            if i >= mmap_node_count {
                                let ram_idx = i - mmap_node_count;
                                let cleaned: Vec<u32> = conns
                                    .into_iter()
                                    .filter(|&c| c != target_idx_u32)
                                    .collect();
                                if self
                                    .inner
                                    .hot
                                    .arena
                                    .update_backlink(ram_idx, layer, m, &cleaned)
                                    .is_ok()
                                {
                                    repaired_ghost_pointers += 1;
                                } else {
                                    remaining_ghost_pointers += 1;
                                }
                            } else {
                                remaining_ghost_pointers += 1;
                            }
                        }
                    }
                }

                return VerificationResult {
                    remaining_ghost_pointers,
                    repaired_ghost_pointers,
                    is_complete: true,
                };
            }
        }

        // Fallback: Full scan over all nodes 0..total_nodes
        for i in 0..total_nodes {
            if deleted.contains(i as u64) || i == target_idx_u32 as usize {
                continue;
            }

            let max_layer = if i < mmap_node_count {
                if let Some(mmap) = mmap_guard.as_ref() {
                    mmap.get_node_record(i)
                        .map(|r| r.max_layer as usize)
                        .unwrap_or(0)
                } else {
                    0
                }
            } else {
                let ram_idx = i - mmap_node_count;
                nodes_read.get(ram_idx).map(|n| n.max_layer).unwrap_or(0)
            };

            for layer in 0..=max_layer {
                let conns = if i < mmap_node_count {
                    if let Some(mmap) = mmap_guard.as_ref() {
                        if let Ok(rec) = mmap.get_node_record(i) {
                            mmap.get_connections(&rec, layer).unwrap_or_default()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    let ram_idx = i - mmap_node_count;
                    self.inner
                        .hot
                        .arena
                        .get_ram_node_connections(ram_idx, layer, m)
                };

                if conns.contains(&target_idx_u32) {
                    if i >= mmap_node_count {
                        let ram_idx = i - mmap_node_count;
                        let cleaned: Vec<u32> =
                            conns.into_iter().filter(|&c| c != target_idx_u32).collect();
                        if self
                            .inner
                            .hot
                            .arena
                            .update_backlink(ram_idx, layer, m, &cleaned)
                            .is_ok()
                        {
                            repaired_ghost_pointers += 1;
                        } else {
                            remaining_ghost_pointers += 1;
                        }
                    } else {
                        remaining_ghost_pointers += 1;
                    }
                }
            }
        }

        VerificationResult {
            remaining_ghost_pointers,
            repaired_ghost_pointers,
            is_complete: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hnsw::config::HnswConfig;
    use contextra_core::{TxId, VectorIndex};

    #[tokio::test]
    async fn test_injected_residual_ghost_pointer_detected_and_repaired() -> Result<()> {
        let dimension = 8;
        let m = 8;
        let config = HnswConfig {
            dimension,
            m,
            ef_construction: 32,
            ..Default::default()
        };

        let index = HnswIndex::try_new(config)?;

        // Insert 100 documents
        for i in 1..=100 {
            let doc_id = DocId::new(i as u64);
            let vector = vec![(i as f32) / 100.0; dimension];
            index.insert(TxId::new(i as u64), doc_id, &vector).await?;
            index.commit(TxId::new(i as u64)).await?;
        }

        // Target to delete is doc 1 (target_idx = 0)
        let target_doc = DocId::new(1);
        let target_idx_u32 = 0u32;
        let ram_idx_80 = 80usize;

        // Inject ghost pointer at node index 80 pointing to 0
        let existing = index
            .inner
            .hot
            .arena
            .get_ram_node_connections(ram_idx_80, 0, m);
        let mut injected = existing.clone();
        if !injected.contains(&target_idx_u32) {
            injected.push(target_idx_u32);
        }
        index
            .inner
            .hot
            .arena
            .update_backlink(ram_idx_80, 0, m, &injected)?;

        let mut mut_index = index;
        let stats = mut_index.remove_with_graph_repair(target_doc)?;

        assert_eq!(stats.doc_id, target_doc);
        assert!(stats.verified_no_ghost_pointers);

        // Check node 80 is clean
        let after = mut_index
            .inner
            .hot
            .arena
            .get_ram_node_connections(ram_idx_80, 0, m);
        assert!(!after.contains(&target_idx_u32));

        Ok(())
    }
}
