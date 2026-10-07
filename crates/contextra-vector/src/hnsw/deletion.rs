// FILE-CONTEXT
// STAND: 2026-10-06
// ZWECK: Synchronous ghost-free node deletion with targeted & complete neighborhood verification for HNSW graph.
// INVARIANTEN: INV-DELETION-2: After Ok(..) of remove_with_graph_repair(doc_id), no neighborhood pointer points to doc_id.
// Lock hierarchy: write_mutex -> entry_point -> nodes / doc_to_node / deleted_nodes.

use ahash::AHashSet;
use contextra_core::{error::HnswDeletionError, ContextraError, DocId, Result};

use super::batch::SearchContext;
use super::types::{Candidate, HnswIndex};
use super::verify::{GhostScan, IncompleteReason};

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
                        let rec = mmap.get_node_record(target_idx)?;
                        mmap.get_connections(&rec, layer)?
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
                            let rec = mmap.get_node_record(neighbor_idx)?;
                            mmap.get_connections(&rec, layer)?
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
                                    let rec = mmap.get_node_record(rem as usize)?;
                                    mmap.get_connections(&rec, layer)?
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
        // Compute actual total degree of deleted node across all layers
        let actual_total_degree = {
            let nodes_read = self.inner.hot.nodes.read();
            let mmap_guard = self.inner.cold.mmap_index.read();
            let mut degree = 0usize;

            for layer in 0..=target_max_layer {
                let conns_len = if target_idx < mmap_node_count {
                    if let Some(mmap) = mmap_guard.as_ref() {
                        if let Ok(rec) = mmap.get_node_record(target_idx) {
                            mmap.get_connections(&rec, layer).map(|c| c.len()).unwrap_or(0)
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                } else {
                    let ram_idx = target_idx - mmap_node_count;
                    if ram_idx < nodes_read.len() {
                        self.inner
                            .hot
                            .arena
                            .get_ram_node_connections(ram_idx, layer, m)
                            .len()
                    } else {
                        0
                    }
                };
                degree += conns_len;
            }
            degree
        };

        let budget = (actual_total_degree.max(1)) * m * 4;
        let (verification_res, scan_result) =
            self.verify_no_ghost_pointers(target_idx as u32, budget, Some(&targeted_nodes));

        repaired_edges += verification_res.repaired_ghost_pointers;

        if !scan_result.is_verified_no_ghost_pointers() {
            let remaining = match scan_result {
                GhostScan::Violated(n) => n as usize,
                GhostScan::Complete(n) => n as usize,
                GhostScan::Incomplete(_) => {
                    verification_res.remaining_ghost_pointers.max(1)
                }
            };
            return Err(ContextraError::GraphRepairFailed(
                HnswDeletionError::VerificationFailed {
                    remaining_pointers: remaining,
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
    /// Returns a tuple of `(VerificationResult, GhostScan)`. Only `GhostScan::Complete(0)`
    /// indicates a verified ghost-pointer-free graph.
    pub fn verify_no_ghost_pointers(
        &self,
        target_idx_u32: u32,
        budget: usize,
        targeted_nodes: Option<&AHashSet<usize>>,
    ) -> (VerificationResult, GhostScan) {
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
                let mut inspected = 0usize;
                for &i in nodes_to_check {
                    inspected += 1;
                    if inspected > budget {
                        let res = VerificationResult {
                            remaining_ghost_pointers,
                            repaired_ghost_pointers,
                            is_complete: false,
                        };
                        return (res, GhostScan::Incomplete(IncompleteReason::BudgetExhausted));
                    }

                    if i >= total_nodes
                        || deleted.contains(i as u64)
                        || i == target_idx_u32 as usize
                    {
                        continue;
                    }

                    let max_layer = if i < mmap_node_count {
                        if let Some(mmap) = mmap_guard.as_ref() {
                            match mmap.get_node_record(i) {
                                Ok(r) => r.max_layer as usize,
                                Err(_) => {
                                    let res = VerificationResult {
                                        remaining_ghost_pointers,
                                        repaired_ghost_pointers,
                                        is_complete: false,
                                    };
                                    return (res, GhostScan::Incomplete(IncompleteReason::ReadError));
                                }
                            }
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
                                let rec = match mmap.get_node_record(i) {
                                    Ok(r) => r,
                                    Err(_) => {
                                        let res = VerificationResult {
                                            remaining_ghost_pointers,
                                            repaired_ghost_pointers,
                                            is_complete: false,
                                        };
                                        return (res, GhostScan::Incomplete(IncompleteReason::ReadError));
                                    }
                                };
                                match mmap.get_connections(&rec, layer) {
                                    Ok(c) => c,
                                    Err(_) => {
                                        let res = VerificationResult {
                                            remaining_ghost_pointers,
                                            repaired_ghost_pointers,
                                            is_complete: false,
                                        };
                                        return (res, GhostScan::Incomplete(IncompleteReason::ReadError));
                                    }
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

                let total_ghosts = (remaining_ghost_pointers + repaired_ghost_pointers) as u64;
                let scan = if total_ghosts == 0 {
                    GhostScan::Complete(0)
                } else {
                    GhostScan::Violated(total_ghosts)
                };

                let res = VerificationResult {
                    remaining_ghost_pointers,
                    repaired_ghost_pointers,
                    is_complete: true,
                };
                return (res, scan);
            }
        }

        // Fallback: Full scan over all nodes 0..total_nodes
        if total_nodes > budget {
            let res = VerificationResult {
                remaining_ghost_pointers,
                repaired_ghost_pointers,
                is_complete: false,
            };
            return (res, GhostScan::Incomplete(IncompleteReason::BudgetExhausted));
        }

        let mut inspected = 0usize;
        for i in 0..total_nodes {
            inspected += 1;
            if inspected > budget {
                let res = VerificationResult {
                    remaining_ghost_pointers,
                    repaired_ghost_pointers,
                    is_complete: false,
                };
                return (res, GhostScan::Incomplete(IncompleteReason::BudgetExhausted));
            }

            if deleted.contains(i as u64) || i == target_idx_u32 as usize {
                continue;
            }

            let max_layer = if i < mmap_node_count {
                if let Some(mmap) = mmap_guard.as_ref() {
                    match mmap.get_node_record(i) {
                        Ok(r) => r.max_layer as usize,
                        Err(_) => {
                            let res = VerificationResult {
                                remaining_ghost_pointers,
                                repaired_ghost_pointers,
                                is_complete: false,
                            };
                            return (res, GhostScan::Incomplete(IncompleteReason::ReadError));
                        }
                    }
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
                        let rec = match mmap.get_node_record(i) {
                            Ok(r) => r,
                            Err(_) => {
                                let res = VerificationResult {
                                    remaining_ghost_pointers,
                                    repaired_ghost_pointers,
                                    is_complete: false,
                                };
                                return (res, GhostScan::Incomplete(IncompleteReason::ReadError));
                            }
                        };
                        match mmap.get_connections(&rec, layer) {
                            Ok(c) => c,
                            Err(_) => {
                                let res = VerificationResult {
                                    remaining_ghost_pointers,
                                    repaired_ghost_pointers,
                                    is_complete: false,
                                };
                                return (res, GhostScan::Incomplete(IncompleteReason::ReadError));
                            }
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

        let total_ghosts = (remaining_ghost_pointers + repaired_ghost_pointers) as u64;
        let scan = if total_ghosts == 0 {
            GhostScan::Complete(0)
        } else {
            GhostScan::Violated(total_ghosts)
        };

        let res = VerificationResult {
            remaining_ghost_pointers,
            repaired_ghost_pointers,
            is_complete: true,
        };
        (res, scan)
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

    #[tokio::test]
    async fn test_verifier_ghost_scan_outcomes() -> Result<()> {
        let dim = 8;
        let config = HnswConfig {
            dimension: dim,
            m: 8,
            ef_construction: 32,
            ..Default::default()
        };

        let index = HnswIndex::try_new(config.clone())?;

        for i in 1..=20 {
            let doc_id = DocId::new(i as u64);
            let vector = vec![(i as f32) / 20.0; dim];
            index.insert(TxId::new(i as u64), doc_id, &vector).await?;
            index.commit(TxId::new(i as u64)).await?;
        }

        // Delete doc 1 cleanly
        let mut mut_index = index;
        let target_doc = DocId::new(1);
        let stats = mut_index.remove_with_graph_repair(target_doc)?;
        assert!(stats.verified_no_ghost_pointers);

        // 1. Clean graph scan for deleted doc -> Complete(0)
        let target_idx_u32 = 0u32;
        let (_res, scan_clean) = mut_index.verify_no_ghost_pointers(target_idx_u32, 1000, None);
        assert_eq!(scan_clean, GhostScan::Complete(0));
        assert!(scan_clean.is_verified_no_ghost_pointers());

        // 2. Budget exhaustion -> Incomplete(BudgetExhausted)
        let (_res, scan_budget) = mut_index.verify_no_ghost_pointers(target_idx_u32, 2, None);
        assert_eq!(
            scan_budget,
            GhostScan::Incomplete(IncompleteReason::BudgetExhausted)
        );
        assert!(!scan_budget.is_verified_no_ghost_pointers());

        // 3. Unrepairable Ghost pointer on read-only / mmap node -> Violated(n)
        // Clear deleted set so mmap node index 0 is evaluated as active
        mut_index.inner.cold.deleted_nodes.write().clear();

        let header = crate::persistence::HnswHeader::new_v2_with_bias(
            dim as u32, 8, 0, 0, 0.0, 1.0, 1, 0, 80, 200, 1, 0, 0, 0.0, 0.0,
        );
        let mut fake_mmap = memmap2::MmapMut::map_anon(300).unwrap();
        // Write header
        fake_mmap[..crate::persistence::HnswHeader::SIZE].copy_from_slice(&header.to_bytes());
        // Node 0 record at offset 80
        let rec0 = crate::persistence::NodeRecord {
            doc_id: 100,
            max_layer: 0,
            vector_offset: 120,
            connections_offset: 200,
        };
        fake_mmap[80..80 + crate::persistence::NodeRecord::SIZE]
            .copy_from_slice(&rec0.to_bytes());
        // Connections at offset 200: 1 layer, 1 connection pointing to 999
        fake_mmap[200] = 1; // 1 layer
        fake_mmap[201..205].copy_from_slice(&1u32.to_le_bytes()); // len 1
        fake_mmap[205..209].copy_from_slice(&999u32.to_le_bytes()); // conn 999

        let mmap_file = tempfile::NamedTempFile::new().unwrap();
        let mmap_index = crate::persistence::MmapIndex {
            mmap: std::sync::Arc::new(fake_mmap.make_read_only().unwrap()),
            file_handle: std::sync::Arc::new(mmap_file.reopen().unwrap()),
            header,
        };
        *mut_index.inner.cold.mmap_index.write() = Some(mmap_index);

        let (_res, scan_violated) = mut_index.verify_no_ghost_pointers(999, 1000, None);
        assert_eq!(scan_violated, GhostScan::Violated(1));
        assert!(!scan_violated.is_verified_no_ghost_pointers());

        Ok(())
    }

    #[tokio::test]
    async fn test_verifier_mmap_read_error_incomplete() -> Result<()> {
        let config = HnswConfig {
            dimension: 4,
            m: 8,
            ..Default::default()
        };

        let index = HnswIndex::try_new(config)?;

        let header = crate::persistence::HnswHeader::new_v2_with_bias(
            4, 8, 0, 0, 0.0, 1.0, 10, 0, 80, 200, 1, 0, 0, 0.0, 0.0,
        );
        let fake_mmap = memmap2::MmapMut::map_anon(100).unwrap().make_read_only().unwrap();

        let mmap_file = tempfile::NamedTempFile::new().unwrap();
        let mmap_index = crate::persistence::MmapIndex {
            mmap: std::sync::Arc::new(fake_mmap),
            file_handle: std::sync::Arc::new(mmap_file.reopen().unwrap()),
            header,
        };

        *index.inner.cold.mmap_index.write() = Some(mmap_index);

        let (_res, scan) = index.verify_no_ghost_pointers(0, 1000, None);
        assert_eq!(scan, GhostScan::Incomplete(IncompleteReason::ReadError));
        assert!(!scan.is_verified_no_ghost_pointers());

        Ok(())
    }
}
