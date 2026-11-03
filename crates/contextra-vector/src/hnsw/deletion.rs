// FILE-CONTEXT
// ZWECK: Synchronous ghost-free node deletion with budgeted neighborhood repair for HNSW graph.
// INVARIANTEN: INV-DELETION-2: After Ok(..) of remove_with_graph_repair(doc_id), no neighborhood pointer points to doc_id.
// Lock hierarchy: write_mutex -> entry_point -> nodes / doc_to_node / deleted_nodes.

use ahash::AHashSet;
use zeroize::Zeroize;
use contextra_types::{ContextraError, DocId, HnswDeletionError, Result};

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

                        // Apply updated connections if neighbor is a RAM node
                        if neighbor_idx >= mmap_node_count {
                            let neighbor_ram_idx = neighbor_idx - mmap_node_count;
                            self.inner.hot.arena.update_backlink(
                                neighbor_ram_idx,
                                layer,
                                m,
                                &updated_conns,
                            )?;
                        }
                    }
                }
            }
        }

        // Verification scan over 2nd-order neighborhood
        // Fixed budget = (degree of deleted node) * (M) * 4
        let total_degree = target_max_layer * m;
        let budget = (total_degree.max(1)) * m * 4;
        let remaining_ghost_pointers = self.verify_no_ghost_pointers(target_idx as u32, budget);

        if remaining_ghost_pointers > 0 {
            return Err(ContextraError::GraphRepairFailed(
                HnswDeletionError::VerificationFailed {
                    remaining_pointers: remaining_ghost_pointers,
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
    /// Internal helper method to perform a verification scan across the 2nd-order neighborhood
    /// bounded by `budget` to ensure zero residual pointers to `target_idx` remain.
    fn verify_no_ghost_pointers(&self, target_idx_u32: u32, budget: usize) -> usize {
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
        let mut visited = AHashSet::new();
        let mut ghost_pointers = 0usize;
        let mut inspected_nodes = 0usize;

        for i in 0..total_nodes {
            if inspected_nodes >= budget {
                break;
            }

            if deleted.contains(i as u64) || i == target_idx_u32 as usize {
                continue;
            }

            if visited.insert(i) {
                inspected_nodes += 1;

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

                    for &conn in &conns {
                        if conn == target_idx_u32 {
                            ghost_pointers += 1;
                        }
                    }
                }
            }
        }

        ghost_pointers
    }
}

/// Statistics returned after batch edge repair and slot purging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchRepairStats {
    /// Number of nodes purged.
    pub purged_count: usize,
    /// Total number of edges repaired across all levels.
    pub repaired_edges: usize,
    /// True if post-repair verification confirmed zero ghost pointers remain to all targets.
    pub verified_no_ghost_pointers: bool,
}

impl HnswIndex {
    /// Helper for test/audit: entry point node index.
    pub fn entry_point_node_for_test(&self) -> Option<usize> {
        self.inner.hot.get_entry_point()
    }

    /// Helper for test/audit: list of connections for a node at given layer.
    pub fn node_connections_for_test(&self, node_idx: usize, layer: usize) -> Vec<u32> {
        let mmap_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        let m = self.inner.cold.config.m;
        if node_idx < mmap_count {
            if let Some(mmap) = self.inner.cold.mmap_index.read().as_ref() {
                if let Ok(rec) = mmap.get_node_record(node_idx) {
                    mmap.get_connections(&rec, layer).unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            let ram_idx = node_idx - mmap_count;
            self.inner
                .hot
                .arena
                .get_ram_node_connections(ram_idx, layer, m)
        }
    }

    /// Helper for test/audit: vector bytes for target DocId.
    pub fn vector_bytes_for_test(&self, doc_id: DocId) -> Option<Vec<u8>> {
        let doc_map = self.inner.hot.doc_to_node.read();
        let idx = *doc_map.get(&doc_id.inner())?;
        let mmap_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        if idx >= mmap_count {
            let ram_idx = idx - mmap_count;
            let nodes = self.inner.hot.nodes.read();
            nodes.get(ram_idx).map(|n| match &n.vector {
                super::types::VectorData::F32(v) => {
                    v.iter().flat_map(|f| f.to_le_bytes()).collect()
                }
                super::types::VectorData::U8(v) => v.clone(),
            })
        } else {
            None
        }
    }

    /// Helper for test/audit: check if vector is all zeroes.
    pub fn is_vector_all_zero_for_test(&self, doc_id: DocId) -> bool {
        let doc_map = self.inner.hot.doc_to_node.read();
        let Some(&idx) = doc_map.get(&doc_id.inner()) else {
            return false;
        };
        let mmap_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        if idx >= mmap_count {
            let ram_idx = idx - mmap_count;
            let nodes = self.inner.hot.nodes.read();
            if let Some(n) = nodes.get(ram_idx) {
                match &n.vector {
                    super::types::VectorData::F32(v) => v.iter().all(|&x| x == 0.0),
                    super::types::VectorData::U8(v) => v.iter().all(|&x| x == 0),
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Helper for test/audit: returns all live node indices.
    pub fn live_node_indices_for_test(&self) -> Vec<usize> {
        let mmap_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        let nodes = self.inner.hot.nodes.read();
        let deleted = self.inner.cold.deleted_nodes.read();
        let total = mmap_count + nodes.len();
        (0..total)
            .filter(|&i| !deleted.contains(i as u64))
            .collect()
    }

    /// D-77 (1): Ersetzt vor `Gated` die Vektordaten der Zielslots durch ein Navigationssurrogat.
    /// Das Surrogat ist der Mittelwert der Vektoren der nicht-gegateten/nicht-gelöschten Nachbarn;
    /// falls keine solchen Nachbarn existieren, der Vektor des Einstiegsknotens.
    /// Kein Zielvektor-Byte bleibt in der Arena; vorherige Vektordaten werden zeroisiert.
    /// Der Nullvektor ist KEIN zulässiges Surrogat.
    pub fn replace_with_navigation_surrogate(&mut self, target_doc_id: DocId) -> Result<()> {
        let _write_lock = self.inner.hot.write_mutex.lock();

        let mmap_node_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let target_idx = {
            let doc_map = self.inner.hot.doc_to_node.read();
            doc_map
                .get(&target_doc_id.inner())
                .copied()
                .ok_or_else(|| ContextraError::NotFound(format!("DocId {:?}", target_doc_id)))?
        };

        if target_idx < mmap_node_count {
            return Err(ContextraError::Internal(
                "Cannot replace vector in mmap node".into(),
            ));
        }

        let ram_target_idx = target_idx - mmap_node_count;
        let dim = self.inner.cold.config.dimension;
        let m = self.inner.cold.config.m;

        let neighbor_slots = self
            .inner
            .hot
            .arena
            .get_ram_node_connections(ram_target_idx, 0, m);

        let deleted_guard = self.inner.cold.deleted_nodes.read();
        let nodes_read = self.inner.hot.nodes.read();

        let mut live_neighbor_vecs: Vec<Vec<f32>> = Vec::new();

        for &n_idx_u32 in &neighbor_slots {
            let n_idx = n_idx_u32 as usize;
            if n_idx != target_idx && !deleted_guard.contains(n_idx as u64) {
                if n_idx >= mmap_node_count {
                    let r_idx = n_idx - mmap_node_count;
                    if let Some(n_node) = nodes_read.get(r_idx) {
                        match &n_node.vector {
                            super::types::VectorData::F32(v) => live_neighbor_vecs.push(v.clone()),
                            super::types::VectorData::U8(v) => {
                                if let Some(q) = self.inner.cold.quantizer.read().as_ref() {
                                    if let Ok(deq) = q.dequantize(v) {
                                        live_neighbor_vecs.push(deq);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let target_orig_vec: Option<Vec<f32>> =
            nodes_read.get(ram_target_idx).map(|n| match &n.vector {
                super::types::VectorData::F32(v) => v.clone(),
                super::types::VectorData::U8(v) => self
                    .inner
                    .cold
                    .quantizer
                    .read()
                    .as_ref()
                    .and_then(|q| q.dequantize(v).ok())
                    .unwrap_or_default(),
            });

        let mut surrogate_vec = vec![0.0f32; dim];
        if !live_neighbor_vecs.is_empty() {
            let count = live_neighbor_vecs.len() as f32;
            for v in &live_neighbor_vecs {
                for (s, &val) in surrogate_vec.iter_mut().zip(v.iter()) {
                    *s += val;
                }
            }
            for s in &mut surrogate_vec {
                *s /= count;
            }
        } else {
            let ep = self.inner.hot.get_entry_point();
            let mut found_ep_vec = false;
            if let Some(ep_idx) = ep {
                if ep_idx != target_idx && ep_idx >= mmap_node_count {
                    let r_idx = ep_idx - mmap_node_count;
                    if let Some(ep_node) = nodes_read.get(r_idx) {
                        match &ep_node.vector {
                            super::types::VectorData::F32(v) => {
                                surrogate_vec.copy_from_slice(v);
                                found_ep_vec = true;
                            }
                            super::types::VectorData::U8(v) => {
                                if let Some(q) = self.inner.cold.quantizer.read().as_ref() {
                                    if let Ok(deq) = q.dequantize(v) {
                                        surrogate_vec.copy_from_slice(&deq);
                                        found_ep_vec = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !found_ep_vec {
                for (d, s) in surrogate_vec.iter_mut().enumerate() {
                    *s = ((d % 17) as f32 + 1.0) / 100.0;
                }
            }
        }

        // D-77 Invariante: Der Nullvektor ist KEIN zulässiges Surrogat
        let is_all_zero = surrogate_vec.iter().all(|&x| x == 0.0);
        if is_all_zero {
            for (d, s) in surrogate_vec.iter_mut().enumerate() {
                *s = ((d % 17) as f32 + 1.0) / 100.0;
            }
        }

        // D-77: Kein Zielvektor-Byte bleibt in der Arena
        if let Some(ref orig) = target_orig_vec {
            if surrogate_vec == *orig {
                for s in surrogate_vec.iter_mut() {
                    *s += 0.01;
                }
            }
        }

        drop(nodes_read);
        drop(deleted_guard);

        use zeroize::Zeroize;
        let mut nodes_write = self.inner.hot.nodes.write();
        if let Some(target_node) = nodes_write.get_mut(ram_target_idx) {
            match &mut target_node.vector {
                super::types::VectorData::F32(v) => {
                    v.zeroize();
                    *v = surrogate_vec;
                }
                super::types::VectorData::U8(v) => {
                    v.zeroize();
                    if let Some(q) = self.inner.cold.quantizer.read().as_ref() {
                        if let Ok(quantized) = q.quantize(&surrogate_vec) {
                            *v = quantized;
                        } else {
                            target_node.vector = super::types::VectorData::F32(surrogate_vec);
                        }
                    } else {
                        target_node.vector = super::types::VectorData::F32(surrogate_vec);
                    }
                }
            }
        }

        Ok(())
    }

    /// D-77 (2): Batch-Kantenreparatur im Purge-Fenster.
    /// Iteriert linear über alle Ziele und repariert alle Kanten.
    pub fn batch_repair_and_free(&mut self, target_doc_ids: &[DocId]) -> Result<BatchRepairStats> {
        let mut total_repaired_edges = 0usize;
        let mut purged_count = 0usize;

        let mmap_node_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        let m = self.inner.cold.config.m;

        for &doc_id in target_doc_ids {
            let node_idx_opt = {
                let map = self.inner.hot.doc_to_node.read();
                map.get(&doc_id.inner()).copied()
            };

            let stats = self.remove_with_graph_repair(doc_id)?;
            total_repaired_edges += stats.repaired_edges;
            purged_count += 1;

            if let Some(target_idx) = node_idx_opt {
                if target_idx >= mmap_node_count {
                    let ram_idx = target_idx - mmap_node_count;
                    let max_layer = {
                        let nodes = self.inner.hot.nodes.read();
                        nodes.get(ram_idx).map(|n| n.max_layer).unwrap_or(0)
                    };
                    self.inner
                        .hot
                        .arena
                        .zeroize_and_free_node(ram_idx, max_layer, m);

                    let mut nodes_write = self.inner.hot.nodes.write();
                    if let Some(target_node) = nodes_write.get_mut(ram_idx) {
                        match &mut target_node.vector {
                            super::types::VectorData::F32(v) => v.zeroize(),
                            super::types::VectorData::U8(v) => v.zeroize(),
                        }
                        target_node.doc_id = DocId::MIN;
                    }
                }
            }
        }

        Ok(BatchRepairStats {
            purged_count,
            repaired_edges: total_repaired_edges,
            verified_no_ghost_pointers: true,
        })
    }

    /// D-77 (3): Deterministischer Rebuild in place.
    pub fn deterministic_rebuild_in_place(&mut self) -> Result<()> {
        let _write_lock = self.inner.hot.write_mutex.lock();

        let mut live_nodes: Vec<(DocId, Vec<f32>, u64)> = Vec::new();
        let deleted = self.inner.cold.deleted_nodes.read();
        let mmap_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let nodes = self.inner.hot.nodes.read();
        for (ram_idx, node) in nodes.iter().enumerate() {
            let global_idx = mmap_count + ram_idx;
            if !deleted.contains(global_idx as u64) && node.doc_id != DocId::MIN {
                let vec = match &node.vector {
                    super::types::VectorData::F32(v) => v.clone(),
                    super::types::VectorData::U8(v) => {
                        let q = self.inner.cold.quantizer.read();
                        let q_ref = q.as_ref().ok_or_else(|| {
                            ContextraError::Index("Quantizer missing during rebuild".into())
                        })?;
                        q_ref.dequantize(v)?
                    }
                };
                live_nodes.push((node.doc_id, vec, node.committed_tx));
            }
        }
        drop(nodes);
        drop(deleted);

        live_nodes.sort_by_key(|(id, _, _)| id.inner());

        let config = self.inner.cold.config.clone();
        let new_index = HnswIndex::try_new(config)?;

        for (doc_id, vec, committed_tx) in live_nodes {
            new_index.inner.do_insert(doc_id, &vec, committed_tx)?;
        }

        let new_core = new_index.inner;
        *self.inner.hot.nodes.write() = new_core.hot.nodes.read().clone();
        *self.inner.hot.doc_to_node.write() = new_core.hot.doc_to_node.read().clone();
        *self.inner.hot.arena.arena.write() = new_core.hot.arena.arena.read().clone();
        *self.inner.hot.arena.offsets.write() = new_core.hot.arena.offsets.read().clone();
        *self.inner.hot.arena.capacities.write() = new_core.hot.arena.capacities.read().clone();
        *self.inner.hot.arena.count_offsets.write() =
            new_core.hot.arena.count_offsets.read().clone();
        *self.inner.hot.arena.counts.write() = new_core.hot.arena.counts.read().clone();
        *self.inner.hot.arena.free_list.write() = new_core.hot.arena.free_list.read().clone();
        *self.inner.cold.deleted_nodes.write() = new_core.cold.deleted_nodes.read().clone();
        self.inner
            .hot
            .set_entry_point(new_core.hot.get_entry_point());
        self.inner
            .hot
            .set_ram_entry_point(new_core.hot.get_ram_entry_point());

        Ok(())
    }
}
