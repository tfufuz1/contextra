// FILE-CONTEXT
// ZWECK: Schichteigener rein lesender Residue-Scanner für HNSW (Vector).
// INVARIANTEN: Pure read path — haelt NIE den globalen write_mutex (INV-D9).
// Keine Locks über .await. Lock-Reihenfolge einhalten.

use contextra_core::{DocId, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::types::{HnswIndex, VectorData};

/// Scan depth for HNSW residue analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResDepth {
    /// O(Footprint · M) via backlinks/direct neighborhood.
    Footprint,
    /// Vollständiger Durchlauf aller Nachbarlisten aller Ebenen.
    FullSweep,
}

impl Default for ResDepth {
    fn default() -> Self {
        ResDepth::Footprint
    }
}

/// Structural residue evidence collected during HNSW scan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResHnswEvidence {
    pub nodes_checked: usize,
    pub vector_copies_zeroed: usize,
    pub vector_copies_zeroed_f32: usize,
    pub vector_copies_zeroed_u8: usize,
    pub vector_copies_zeroed_codes: usize,
    pub nonzero_vector_copies: usize,
    pub incoming_edges: usize,
    pub entry_point_valid: bool,
    pub pinned_by_reader: bool,
    pub persisted_snapshots_present: bool,
    pub depth: ResDepth,
}

/// Evidence report produced by directly inspecting internal HNSW structures for residual data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HnswResidueReport {
    /// True if the node for `doc_id` exists in `doc_to_node` or has a matching non-deleted/non-zeroed entry in `nodes`.
    pub node_found: bool,
    /// Number of non-zero vector bytes found in the node's vector representation.
    pub non_zero_vector_bytes: usize,
    /// Number of incoming graph edges from other active nodes pointing to this node.
    pub incoming_edges: usize,
    /// Number of outgoing graph edges originating from this node.
    pub outgoing_edges: usize,
    /// True if the index sequence log currently has active reader snapshot pins.
    pub pinned_by_reader: bool,
    /// True if the current entry point or RAM entry point matches this document's node index.
    pub entry_point_is_doc: bool,
    /// True if quantized (U8) vector data is present and non-zero for this document.
    pub quantized_code_found: bool,
}

impl HnswIndex {
    /// Scans the HNSW index for residual state associated with `doc_ids`.
    ///
    /// ## Invariants
    /// - **INV-D9**: Pure read-path. NEVER acquires `write_mutex`.
    /// - Non-blocking: acquires short read locks without `.await` points.
    pub fn scan_residue(&self, doc_ids: &[DocId], depth: ResDepth) -> Result<ResHnswEvidence> {
        let mut evidence = ResHnswEvidence {
            nodes_checked: doc_ids.len(),
            depth,
            entry_point_valid: true,
            ..Default::default()
        };

        // 1. Check persistence/mmap presence (AP-07)
        let mmap_guard = self.inner.cold.mmap_index.read();
        evidence.persisted_snapshots_present = mmap_guard.is_some();

        let mmap_node_count = mmap_guard
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let nodes = self.inner.hot.nodes.read();
        let deleted = self.inner.cold.deleted_nodes.read();
        let seq_log = self.inner.cold.seq_log.read();
        let total_nodes = mmap_node_count + nodes.len();

        // 2. Entry point validity check
        let ep_opt = self.inner.hot.get_entry_point();
        let ram_ep_opt = self.inner.hot.get_ram_entry_point();

        let live_nodes_count = total_nodes.saturating_sub(deleted.len() as usize);

        if let Some(ep) = ep_opt {
            if ep >= total_nodes || deleted.contains(ep as u64) {
                evidence.entry_point_valid = false;
            }
        } else if live_nodes_count > 0 {
            evidence.entry_point_valid = false;
        }

        if let Some(ram_ep) = ram_ep_opt {
            if ram_ep >= total_nodes || deleted.contains(ram_ep as u64) {
                evidence.entry_point_valid = false;
            }
        }

        // 3. Reader pinning detection
        if seq_log.min_retention_seq().is_some() {
            evidence.pinned_by_reader = true;
        }

        // 4. Locate target slots for target doc_ids
        let mut target_slots = HashSet::new();
        {
            let doc_map = self.inner.hot.doc_to_node.read();
            for &doc_id in doc_ids {
                if let Some(&idx) = doc_map.get(&doc_id.inner()) {
                    target_slots.insert(idx);
                }
            }
        }

        // Also check RAM nodes array directly for any node matching doc_id
        for (ram_i, node) in nodes.iter().enumerate() {
            if doc_ids.contains(&node.doc_id) {
                let global_idx = mmap_node_count + ram_i;
                target_slots.insert(global_idx);
            }
        }

        // 5. Inspect vector memory zeroing
        for &target_idx in &target_slots {
            if target_idx >= mmap_node_count {
                let ram_idx = target_idx - mmap_node_count;
                if let Some(node) = nodes.get(ram_idx) {
                    match &node.vector {
                        VectorData::F32(v) => {
                            if v.iter().all(|&x| x == 0.0) {
                                evidence.vector_copies_zeroed += 1;
                                evidence.vector_copies_zeroed_f32 += 1;
                            } else {
                                evidence.nonzero_vector_copies += 1;
                            }
                        }
                        VectorData::U8(v) => {
                            if v.iter().all(|&x| x == 0) {
                                evidence.vector_copies_zeroed += 1;
                                evidence.vector_copies_zeroed_u8 += 1;
                            } else {
                                evidence.nonzero_vector_copies += 1;
                            }
                        }
                    }
                }
            }
        }

        // 6. Inspect incoming edges (Footprint vs FullSweep)
        let m = self.inner.cold.config.m;
        let max_layer = self
            .inner
            .hot
            .max_layer
            .load(std::sync::atomic::Ordering::Acquire) as usize;

        if !target_slots.is_empty() {
            match depth {
                ResDepth::Footprint => {
                    // Check direct neighbors of target slots across layers
                    let mut neighborhood = HashSet::new();
                    for &target_idx in &target_slots {
                        for layer in 0..=max_layer {
                            let conns = if target_idx < mmap_node_count {
                                self.inner
                                    .get_node_connections(target_idx, layer)
                            } else {
                                let ram_idx = target_idx - mmap_node_count;
                                self.inner
                                    .hot
                                    .arena
                                    .get_ram_node_connections(ram_idx, layer, m)
                            };
                            for &c in &conns {
                                if !target_slots.contains(&(c as usize)) {
                                    neighborhood.insert(c as usize);
                                }
                            }
                        }
                    }

                    for &neighbor_idx in &neighborhood {
                        let neighbor_max_layer = if neighbor_idx < mmap_node_count {
                            mmap_guard
                                .as_ref()
                                .and_then(|mmap| mmap.get_node_record(neighbor_idx).ok())
                                .map(|r| r.max_layer as usize)
                                .unwrap_or(0)
                        } else {
                            let ram_i = neighbor_idx - mmap_node_count;
                            nodes.get(ram_i).map(|n| n.max_layer).unwrap_or(0)
                        };

                        for layer in 0..=neighbor_max_layer {
                            let conns = if neighbor_idx < mmap_node_count {
                                self.inner
                                    .get_node_connections(neighbor_idx, layer)
                            } else {
                                let ram_idx = neighbor_idx - mmap_node_count;
                                self.inner
                                    .hot
                                    .arena
                                    .get_ram_node_connections(ram_idx, layer, m)
                            };

                            for &target_idx in &target_slots {
                                if conns.contains(&(target_idx as u32)) {
                                    evidence.incoming_edges += 1;
                                }
                            }
                        }
                    }
                }
                ResDepth::FullSweep => {
                    // Complete traversal of all neighbor lists of all levels
                    for i in 0..total_nodes {
                        if target_slots.contains(&i) {
                            continue;
                        }

                        let node_max_layer = if i < mmap_node_count {
                            mmap_guard
                                .as_ref()
                                .and_then(|mmap| mmap.get_node_record(i).ok())
                                .map(|r| r.max_layer as usize)
                                .unwrap_or(0)
                        } else {
                            let ram_i = i - mmap_node_count;
                            nodes.get(ram_i).map(|n| n.max_layer).unwrap_or(0)
                        };

                        for layer in 0..=node_max_layer {
                            let conns = if i < mmap_node_count {
                                self.inner
                                    .get_node_connections(i, layer)
                            } else {
                                let ram_i = i - mmap_node_count;
                                self.inner
                                    .hot
                                    .arena
                                    .get_ram_node_connections(ram_i, layer, m)
                            };

                            for &target_idx in &target_slots {
                                if conns.contains(&(target_idx as u32)) {
                                    evidence.incoming_edges += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(evidence)
    }

    /// Directly scans HNSW internal data structures for residual traces of a single `doc_id`.
    ///
    /// Unlike search-based paths, this scanner inspects low-level `doc_to_node` maps,
    /// `nodes` arrays, `arena` connection tables, and entry points directly so that search path
    /// bugs cannot mask residual data.
    pub fn scan_residue_report(&self, doc_id: DocId) -> HnswResidueReport {
        let mmap_guard = self.inner.cold.mmap_index.read();
        let mmap_node_count = mmap_guard
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let doc_map = self.inner.hot.doc_to_node.read();
        let _deleted_guard = self.inner.cold.deleted_nodes.read();
        let nodes_read = self.inner.hot.nodes.read();
        let m = self.inner.cold.config.m;

        // 1. Locate node index and determine node presence
        let mut target_idx = doc_map.get(&doc_id.inner()).copied();

        // If doc_map doesn't contain doc_id, search RAM nodes for matching doc_id (e.g. unzeroed node residual)
        if target_idx.is_none() {
            for (ram_idx, node) in nodes_read.iter().enumerate() {
                if node.doc_id == doc_id {
                    let global_idx = mmap_node_count + ram_idx;
                    target_idx = Some(global_idx);
                    break;
                }
            }
        }

        let node_found = target_idx.is_some();

        // 2. Inspect vector data zeroization
        let mut non_zero_vector_bytes = 0usize;
        let mut quantized_code_found = false;

        if let Some(t_idx) = target_idx {
            if t_idx >= mmap_node_count {
                let ram_idx = t_idx - mmap_node_count;
                if let Some(node) = nodes_read.get(ram_idx) {
                    match &node.vector {
                        VectorData::F32(vec) => {
                            for &val in vec {
                                if val != 0.0 {
                                    non_zero_vector_bytes += 4;
                                }
                            }
                        }
                        VectorData::U8(code) => {
                            for &byte in code {
                                if byte != 0 {
                                    non_zero_vector_bytes += 1;
                                    quantized_code_found = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Inspect entry point
        let entry_point_is_doc = if let Some(t_idx) = target_idx {
            let ep = self.inner.hot.get_entry_point();
            let ram_ep = self.inner.hot.get_ram_entry_point();
            ep == Some(t_idx) || ram_ep == Some(t_idx)
        } else {
            false
        };

        // 4. Inspect outgoing edges
        let mut outgoing_edges = 0usize;
        if let Some(t_idx) = target_idx {
            if t_idx < mmap_node_count {
                if let Some(mmap) = mmap_guard.as_ref() {
                    if let Ok(rec) = mmap.get_node_record(t_idx) {
                        for layer in 0..=rec.max_layer as usize {
                            if let Ok(conns) = mmap.get_connections(&rec, layer) {
                                outgoing_edges += conns.len();
                            }
                        }
                    }
                }
            } else {
                let ram_idx = t_idx - mmap_node_count;
                let max_layer = nodes_read.get(ram_idx).map(|n| n.max_layer).unwrap_or(0);
                for layer in 0..=max_layer {
                    outgoing_edges += self
                        .inner
                        .hot
                        .arena
                        .get_ram_node_connections(ram_idx, layer, m)
                        .len();
                }
            }
        }

        // 5. Inspect incoming edges
        let mut incoming_edges = 0usize;
        if let Some(t_idx) = target_idx {
            let total_nodes = mmap_node_count + nodes_read.len();
            for node_idx in 0..total_nodes {
                if node_idx == t_idx {
                    continue;
                }
                let node_max_layer = if node_idx < mmap_node_count {
                    if let Some(mmap) = mmap_guard.as_ref() {
                        mmap.get_node_record(node_idx)
                            .map(|r| r.max_layer as usize)
                            .unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    let ram_idx = node_idx - mmap_node_count;
                    nodes_read.get(ram_idx).map(|n| n.max_layer).unwrap_or(0)
                };

                for layer in 0..=node_max_layer {
                    let conns = if node_idx < mmap_node_count {
                        if let Some(mmap) = mmap_guard.as_ref() {
                            if let Ok(rec) = mmap.get_node_record(node_idx) {
                                mmap.get_connections(&rec, layer).unwrap_or_default()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        }
                    } else {
                        let ram_idx = node_idx - mmap_node_count;
                        self.inner
                            .hot
                            .arena
                            .get_ram_node_connections(ram_idx, layer, m)
                    };
                    if conns.contains(&(t_idx as u32)) {
                        incoming_edges += 1;
                    }
                }
            }
        }

        let pinned_by_reader = self.inner.cold.seq_log.read().min_retention_seq().is_some();

        HnswResidueReport {
            node_found,
            non_zero_vector_bytes,
            incoming_edges,
            outgoing_edges,
            pinned_by_reader,
            entry_point_is_doc,
            quantized_code_found,
        }
    }
}
