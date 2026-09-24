use std::borrow::Cow;
use std::sync::atomic::Ordering;

use contextra_core::DocId;

use super::arena::{BacklinkTable, HnswArena};
use super::types::{HnswIndexCore, HnswNode, VectorData};

pub struct PreparedInsert {
    pub doc_id: DocId,
    pub vector_data: VectorData,
    pub new_layer: usize,
    pub new_idx: usize,
    pub final_connections: Vec<Vec<u32>>,
    pub neighbor_backlinks: Vec<NeighborBacklink>,
    pub should_update_entry_point: bool,
    pub should_update_ram_entry_point: bool,
}

/// Back-link connection update for a neighbor node at a specific layer.
#[derive(Debug)]
pub struct NeighborBacklink {
    pub neighbor_ram_idx: usize,
    pub layer: usize,
    pub updated_connections: Vec<u32>,
}

/// Batch tracking context for running state across multi-operation transaction commits.
#[derive(Debug)]
pub struct BatchContext {
    pub running_max_layer: usize,
    pub has_entry_point: bool,
    pub has_ram_entry_point: bool,
    pub backlink_map: BacklinkTable,
}

impl BatchContext {
    pub fn new(core: &HnswIndexCore) -> Self {
        Self {
            running_max_layer: core.hot.max_layer.load(Ordering::SeqCst) as usize,
            has_entry_point: core.hot.get_entry_point().is_some(),
            has_ram_entry_point: core.hot.get_ram_entry_point().is_some(),
            backlink_map: BacklinkTable::new(),
        }
    }
}

pub(super) fn get_neighbor_conns_in_batch(
    core: &HnswIndexCore,
    neighbor_idx: usize,
    layer: usize,
    base_batch_idx: usize,
    mmap_node_count: usize,
    _nodes_read: &[HnswNode],
    prior_prepared: &[PreparedInsert],
    batch_ctx: &BatchContext,
) -> Vec<u32> {
    if neighbor_idx >= base_batch_idx {
        let offset = neighbor_idx - base_batch_idx;
        return prior_prepared
            .get(offset)
            .and_then(|p| p.final_connections.get(layer))
            .cloned()
            .unwrap_or_default();
    }

    if neighbor_idx < mmap_node_count {
        return Vec::new();
    }

    let neighbor_ram_idx = neighbor_idx - mmap_node_count;
    if let Some(conns) = batch_ctx.backlink_map.get(neighbor_ram_idx, layer) {
        return conns.clone();
    }

    core.hot
        .get_ram_node_connections(neighbor_ram_idx, layer, core.cold.config.m)
}

/// Helper for hybrid resolution of nodes (RAM vs Mmap, plus in-flight batch prepared inserts).
pub(super) struct SearchContext<'a> {
    pub(super) nodes: &'a [HnswNode],
    pub(super) mmap: Option<&'a crate::persistence::MmapIndex>,
    pub(super) mmap_node_count: usize,
    pub(super) prior_prepared: &'a [PreparedInsert],
    pub(super) backlink_map: Option<&'a BacklinkTable>,
    pub(super) quantizer: Option<Cow<'a, crate::quantize::ScalarQuantizer>>,
    pub(super) arena: &'a HnswArena,
}
#[allow(dead_code)]
struct _OldSearchContext<'a> {
    nodes: &'a [HnswNode],
    mmap: Option<&'a crate::persistence::MmapIndex>,
    mmap_node_count: usize,
    prior_prepared: &'a [PreparedInsert],
    backlink_map: Option<&'a BacklinkTable>,
    quantizer: Option<Cow<'a, crate::quantize::ScalarQuantizer>>,
    arena: &'a HnswArena,
}
