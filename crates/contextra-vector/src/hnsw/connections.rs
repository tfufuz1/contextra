// FILE-CONTEXT
// STAND: 2026-10-07
// ZWECK: In-memory backlink overlay for mmap-resident nodes and centralized graph connection access.
// INVARIANTEN: Zero-Panic, lock-free/shared-read access during concurrent graph traversal.

use ahash::AHashMap;
use contextra_core::Result;
use parking_lot::RwLock;

use super::types::HnswIndexCore;

/// In-memory backlink overlay for mmap-resident nodes.
///
/// Stores backlink repairs made to mmap-resident nodes in RAM (Key: `(global_idx, layer)` -> Value: `Vec<u32>`).
/// Overrides are transient and non-persistent across index reload or system restarts.
#[derive(Default)]
pub struct MmapBacklinkOverlay {
    overrides: RwLock<AHashMap<(usize, usize), Vec<u32>>>,
}

impl MmapBacklinkOverlay {
    /// Creates a new empty overlay.
    pub fn new() -> Self {
        Self {
            overrides: RwLock::new(AHashMap::new()),
        }
    }

    /// Gets overridden connections for a given global node index and layer, if present.
    pub fn get_override(&self, global_idx: usize, layer: usize) -> Option<Vec<u32>> {
        self.overrides.read().get(&(global_idx, layer)).cloned()
    }

    /// Sets an overridden list of backlink connections for a given global node index and layer.
    pub fn set_override(&self, global_idx: usize, layer: usize, connections: Vec<u32>) {
        self.overrides
            .write()
            .insert((global_idx, layer), connections);
    }

    /// Clears all overlay overrides.
    pub fn clear(&self) {
        self.overrides.write().clear();
    }

    /// Returns the number of active overrides stored in the overlay.
    pub fn len(&self) -> usize {
        self.overrides.read().len()
    }

    /// Returns true if the overlay is empty.
    pub fn is_empty(&self) -> bool {
        self.overrides.read().is_empty()
    }
}

impl HnswIndexCore {
    /// Centralized accessor to retrieve connections for any node (RAM or mmap-resident) at a specific layer.
    ///
    /// Resolution order:
    /// 1. RAM node (`global_idx >= mmap_node_count`): Fetch from `arena.get_ram_node_connections(...)`.
    /// 2. Overlay: Check `mmap_backlink_overlay` for active RAM override.
    /// 3. Raw mmap read: Fetch from read-only `mmap_index`.
    pub fn get_node_connections(&self, global_idx: usize, layer: usize) -> Result<Vec<u32>> {
        let mmap_guard = self.cold.mmap_index.read();
        let mmap_node_count = mmap_guard
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        if global_idx >= mmap_node_count {
            let ram_idx = global_idx - mmap_node_count;
            return Ok(self
                .hot
                .get_ram_node_connections(ram_idx, layer, self.cold.config.m));
        }

        if let Some(overridden) = self
            .cold
            .mmap_backlink_overlay
            .get_override(global_idx, layer)
        {
            return Ok(overridden);
        }

        if let Some(mmap) = mmap_guard.as_ref() {
            let record = mmap.get_node_record(global_idx)?;
            return mmap.get_connections(&record, layer);
        }

        Ok(Vec::new())
    }
}
