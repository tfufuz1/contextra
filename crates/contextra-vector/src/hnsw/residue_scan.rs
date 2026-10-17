use contextra_core::DocId;
use serde::{Deserialize, Serialize};

use super::types::{HnswIndex, VectorData};

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
    /// Directly scans HNSW internal data structures for residual traces of `doc_id`.
    ///
    /// Unlike search-based paths, this scanner inspects low-level `doc_to_node` maps,
    /// `nodes` arrays, `arena` connection tables, and entry points directly so that search path
    /// bugs cannot mask residual data.
    pub fn scan_residue(&self, doc_id: DocId) -> HnswResidueReport {
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

        let mut non_zero_vector_bytes = 0;
        let mut quantized_code_found = false;
        let mut outgoing_edges = 0;
        let mut incoming_edges = 0;
        let mut entry_point_is_doc = false;

        if let Some(t_idx) = target_idx {
            // Check entry point
            let ep_val = self.inner.hot.get_entry_point();
            let ram_ep_val = self.inner.hot.get_ram_entry_point();
            if ep_val == Some(t_idx) || ram_ep_val == Some(t_idx) {
                entry_point_is_doc = true;
            }

            // Inspect vector bytes if in RAM
            if t_idx >= mmap_node_count {
                let ram_idx = t_idx - mmap_node_count;
                if let Some(node) = nodes_read.get(ram_idx) {
                    match &node.vector {
                        VectorData::F32(v) => {
                            for f in v {
                                for &b in &f.to_le_bytes() {
                                    if b != 0 {
                                        non_zero_vector_bytes += 1;
                                    }
                                }
                            }
                        }
                        VectorData::U8(v) => {
                            for &b in v {
                                if b != 0 {
                                    non_zero_vector_bytes += 1;
                                    quantized_code_found = true;
                                }
                            }
                        }
                    }
                }
            } else if let Some(mmap) = mmap_guard.as_ref() {
                if let Ok(rec) = mmap.get_node_record(t_idx) {
                    if let Ok(bytes) = mmap.get_vector(&rec) {
                        for &b in bytes {
                            if b != 0 {
                                non_zero_vector_bytes += 1;
                            }
                        }
                        if mmap.header.is_quantized() && non_zero_vector_bytes > 0 {
                            quantized_code_found = true;
                        }
                    }
                }
            }

            // Inspect outgoing edges
            let target_max_layer = if t_idx < mmap_node_count {
                if let Some(mmap) = mmap_guard.as_ref() {
                    mmap.get_node_record(t_idx)
                        .map(|r| r.max_layer as usize)
                        .unwrap_or(0)
                } else {
                    0
                }
            } else {
                let ram_idx = t_idx - mmap_node_count;
                nodes_read.get(ram_idx).map(|n| n.max_layer).unwrap_or(0)
            };

            for layer in 0..=target_max_layer {
                let conns = if t_idx < mmap_node_count {
                    if let Some(mmap) = mmap_guard.as_ref() {
                        if let Ok(rec) = mmap.get_node_record(t_idx) {
                            mmap.get_connections(&rec, layer).unwrap_or_default()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    let ram_idx = t_idx - mmap_node_count;
                    self.inner
                        .hot
                        .arena
                        .get_ram_node_connections(ram_idx, layer, m)
                };
                outgoing_edges += conns.len();
            }

            // Inspect incoming edges from all active nodes
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
