// FILE-CONTEXT
// ZWECK: Schichteigener rein lesender Residue-Scanner für HNSW (Vector).
// INVARIANTEN: Pure read path — haelt NIE den globalen write_mutex (INV-D9).
// Keine Locks über .await. Lock-Reihenfolge einhalten.

use contextra_types::{DocId, Result};
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

impl Default for ResDepth {
    fn default() -> Self {
        ResDepth::Footprint
    }
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
                                    .unwrap_or_default()
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
                                    .unwrap_or_default()
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
                                    .unwrap_or_default()
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
}
