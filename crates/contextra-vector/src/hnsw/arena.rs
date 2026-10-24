// FILE-CONTEXT
// ZWECK: Arena-Allocator (HnswArena) für HNSW-Nachbarlisten mit 64-Byte-Alignment, Slot-Freilisten und CAS-Relinking.
// INVARIANTEN: Zero-Panic Doctrine (ContextraError bei Kapazität/Grenzen); SIMD-Cacheline Alignment (64 Bytes).

//! Arena Allocator for HNSW neighbor lists and O(1) Backlink management.

use ahash::AHashMap;
use contextra_core::{ContextraError, Result};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// Slot alignment in bytes (64-byte alignment = 16 u32 elements for SIMD cache-line compatibility).
pub const ARENA_ALIGNMENT_BYTES: usize = 64;
/// Alignment in u32 elements (64 bytes / 4 bytes = 16 u32 elements).
pub const ARENA_ALIGNMENT_U32: usize = ARENA_ALIGNMENT_BYTES / std::mem::size_of::<u32>();

/// Composite key (neighbor_ram_idx, layer) for backlink lookups.
pub type BacklinkKey = (usize, usize);

/// Thread-safe $O(1)$ backlink connection table using `AHashMap`.
#[derive(Debug, Default, Clone)]
pub struct BacklinkTable {
    pub map: AHashMap<BacklinkKey, Vec<u32>>,
}

impl BacklinkTable {
    /// Creates a new `BacklinkTable`.
    pub fn new() -> Self {
        Self {
            map: AHashMap::new(),
        }
    }

    /// Gets connection list for (neighbor_ram_idx, layer) in O(1) time.
    #[inline]
    pub fn get(&self, neighbor_ram_idx: usize, layer: usize) -> Option<&Vec<u32>> {
        self.map.get(&(neighbor_ram_idx, layer))
    }

    /// Inserts or updates connection list for (neighbor_ram_idx, layer) in O(1) time.
    #[inline]
    pub fn insert(&mut self, neighbor_ram_idx: usize, layer: usize, connections: Vec<u32>) {
        self.map.insert((neighbor_ram_idx, layer), connections);
    }

    /// Clears all stored backlinks.
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

/// Contiguous 64-byte aligned arena allocator for HNSW node connections with free-list slot reuse.
#[derive(Debug)]
pub struct HnswArena {
    pub arena: RwLock<Vec<u32>>,
    pub offsets: RwLock<Vec<usize>>,
    pub capacities: RwLock<Vec<usize>>,
    pub count_offsets: RwLock<Vec<usize>>,
    pub counts: RwLock<Vec<u8>>,
    pub free_list: RwLock<Vec<usize>>,
    pub total_allocated: AtomicU64,
}

impl Default for HnswArena {
    fn default() -> Self {
        Self::new()
    }
}

impl HnswArena {
    /// Creates a new `HnswArena`.
    pub fn new() -> Self {
        Self {
            arena: RwLock::new(Vec::new()),
            offsets: RwLock::new(Vec::new()),
            capacities: RwLock::new(Vec::new()),
            count_offsets: RwLock::new(Vec::new()),
            counts: RwLock::new(Vec::new()),
            free_list: RwLock::new(Vec::new()),
            total_allocated: AtomicU64::new(0),
        }
    }

    /// Calculates layer offset within a node's allocated arena block.
    #[inline]
    pub fn layer_offset(node_offset: usize, layer: usize, m: usize) -> usize {
        if layer == 0 {
            node_offset
        } else {
            node_offset + (m * 2) + (layer - 1) * m
        }
    }

    /// Rounds up an allocation size in u32s to the nearest 64-byte (16 x u32) boundary.
    #[inline]
    pub fn align_capacity(raw_capacity: usize) -> usize {
        (raw_capacity + ARENA_ALIGNMENT_U32 - 1) & !(ARENA_ALIGNMENT_U32 - 1)
    }

    /// Allocates or reuses a slot for a node's neighbor lists across all its layers.
    /// Ensures 64-byte alignment and validates capacity on slot reuse.
    pub fn allocate_node(
        &self,
        max_layer: usize,
        m: usize,
        final_connections: &[Vec<u32>],
    ) -> Result<usize> {
        let raw_capacity = (m * 2) + max_layer * m;
        if raw_capacity == 0 {
            return Err(ContextraError::invalid_input(
                "Node connection capacity cannot be 0",
            ));
        }
        let node_capacity = Self::align_capacity(raw_capacity);

        let mut offsets = self.offsets.write();
        let mut capacities = self.capacities.write();
        let mut count_offsets = self.count_offsets.write();
        let mut counts = self.counts.write();
        let mut arena = self.arena.write();
        let mut free_list = self.free_list.write();

        // Search free_list for a slot with sufficient capacity
        let mut found_free_index = None;
        for (i, &freed_idx) in free_list.iter().enumerate() {
            if freed_idx < capacities.len() && capacities[freed_idx] >= node_capacity {
                found_free_index = Some(i);
                break;
            }
        }

        let (ram_idx, start_offset) = if let Some(free_pos) = found_free_index {
            let reused_idx = free_list.remove(free_pos);
            (reused_idx, offsets[reused_idx])
        } else {
            // Align start offset to 64-byte boundary (16 x u32 elements)
            let current_len = arena.len();
            let aligned_start = Self::align_capacity(current_len);
            if aligned_start > current_len {
                arena.resize(aligned_start, 0);
            }
            let start = arena.len();
            arena.resize(start + node_capacity, 0);
            offsets.push(start);
            capacities.push(node_capacity);
            (offsets.len() - 1, start)
        };

        if ram_idx >= count_offsets.len() {
            let count_start = counts.len();
            count_offsets.push(count_start);
            for (layer, layer_conns) in final_connections.iter().enumerate() {
                let l_offset = Self::layer_offset(start_offset, layer, m);
                let layer_cap = if layer == 0 { m * 2 } else { m };
                let len = layer_conns.len().min(layer_cap);
                if l_offset + len > arena.len() {
                    arena.resize(l_offset + len, 0);
                }
                arena[l_offset..l_offset + len].copy_from_slice(&layer_conns[..len]);
                counts.push(len as u8);
            }
        } else {
            let count_start = count_offsets[ram_idx];
            for (layer, layer_conns) in final_connections.iter().enumerate() {
                let l_offset = Self::layer_offset(start_offset, layer, m);
                let layer_cap = if layer == 0 { m * 2 } else { m };
                let len = layer_conns.len().min(layer_cap);
                if l_offset + len > arena.len() {
                    arena.resize(l_offset + len, 0);
                }
                arena[l_offset..l_offset + len].copy_from_slice(&layer_conns[..len]);
                if count_start + layer < counts.len() {
                    counts[count_start + layer] = len as u8;
                }
            }
        }

        self.total_allocated.fetch_add(1, Ordering::SeqCst);
        Ok(ram_idx)
    }

    /// Gets connection list for a node at a specific layer without panicking.
    pub fn get_ram_node_connections(&self, ram_idx: usize, layer: usize, m: usize) -> Vec<u32> {
        let offsets = self.offsets.read();
        let count_offsets = self.count_offsets.read();
        let counts = self.counts.read();

        if ram_idx >= offsets.len() || ram_idx >= count_offsets.len() {
            return Vec::new();
        }

        let count_start = count_offsets[ram_idx];
        if count_start + layer >= counts.len() {
            return Vec::new();
        }

        let count_end = if ram_idx + 1 < count_offsets.len() {
            count_offsets[ram_idx + 1]
        } else {
            counts.len()
        };

        if count_start + layer >= count_end {
            return Vec::new();
        }

        let len = counts[count_start + layer] as usize;
        let node_offset = offsets[ram_idx];
        let l_offset = Self::layer_offset(node_offset, layer, m);

        let arena = self.arena.read();
        if l_offset + len <= arena.len() {
            arena[l_offset..l_offset + len].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Updates connection list for a backlink update safely.
    pub fn update_backlink(
        &self,
        ram_idx: usize,
        layer: usize,
        m: usize,
        updated_connections: &[u32],
    ) -> Result<()> {
        let offsets = self.offsets.read();
        let count_offsets = self.count_offsets.read();
        let mut counts = self.counts.write();
        let mut arena = self.arena.write();

        if ram_idx >= offsets.len() || ram_idx >= count_offsets.len() {
            return Err(ContextraError::Index(format!(
                "Invalid ram_idx {ram_idx} for backlink update"
            )));
        }

        let node_offset = offsets[ram_idx];
        let count_start = count_offsets[ram_idx];
        let count_end = if ram_idx + 1 < count_offsets.len() {
            count_offsets[ram_idx + 1]
        } else {
            counts.len()
        };

        if count_start + layer < count_end {
            let l_offset = Self::layer_offset(node_offset, layer, m);
            let layer_cap = if layer == 0 { m * 2 } else { m };
            let len = updated_connections.len().min(layer_cap);
            if l_offset + len <= arena.len() {
                arena[l_offset..l_offset + len].copy_from_slice(&updated_connections[..len]);
                counts[count_start + layer] = len as u8;
            }
        }

        Ok(())
    }

    /// Non-blocking relinking for lazy neighbor pruning during search traversal.
    pub fn try_relink_pruned_neighbors(
        &self,
        ram_idx: usize,
        layer: usize,
        m: usize,
        kept_neighbors: &[u32],
    ) -> bool {
        if let (Some(offsets), Some(count_offsets), Some(mut counts), Some(mut arena)) = (
            self.offsets.try_read(),
            self.count_offsets.try_read(),
            self.counts.try_write(),
            self.arena.try_write(),
        ) {
            if ram_idx < offsets.len() && ram_idx < count_offsets.len() {
                let node_offset = offsets[ram_idx];
                let count_start = count_offsets[ram_idx];
                let count_end = if ram_idx + 1 < count_offsets.len() {
                    count_offsets[ram_idx + 1]
                } else {
                    counts.len()
                };
                if count_start + layer < count_end {
                    let l_offset = Self::layer_offset(node_offset, layer, m);
                    let layer_cap = if layer == 0 { m * 2 } else { m };
                    let len = kept_neighbors.len().min(layer_cap);
                    if l_offset + len <= arena.len() {
                        arena[l_offset..l_offset + len].copy_from_slice(&kept_neighbors[..len]);
                        counts[count_start + layer] = len as u8;
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Marks a node's slot as free for future slot reuse.
    pub fn free_node(&self, ram_idx: usize) {
        let offsets = self.offsets.read();
        if ram_idx < offsets.len() {
            self.free_list.write().push(ram_idx);
            self.total_allocated.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_arena_allocation_and_retrieval() -> Result<()> {
        let arena = HnswArena::new();
        let m = 8;
        let conns_l0 = vec![1, 2, 3];
        let conns_l1 = vec![4, 5];
        let final_conns = vec![conns_l0.clone(), conns_l1.clone()];

        let idx = arena.allocate_node(1, m, &final_conns)?;
        assert_eq!(idx, 0);

        let got_l0 = arena.get_ram_node_connections(0, 0, m);
        assert_eq!(got_l0, conns_l0);

        let got_l1 = arena.get_ram_node_connections(0, 1, m);
        assert_eq!(got_l1, conns_l1);

        // Verify 64-byte alignment
        let offsets = arena.offsets.read();
        assert_eq!(offsets[0] % ARENA_ALIGNMENT_U32, 0);

        Ok(())
    }

    #[test]
    fn test_hnsw_arena_slot_reuse_capacity_guard() -> Result<()> {
        let arena = HnswArena::new();
        let m = 8;

        // Allocate node 0 with max_layer = 2 (capacity = 16 + 8 + 8 = 32)
        let idx0 = arena.allocate_node(2, m, &[vec![1], vec![2], vec![3]])?;
        assert_eq!(idx0, 0);

        // Free node 0
        arena.free_node(0);

        // Request allocation for node 1 with max_layer = 4 (capacity = 16 + 32 = 48)
        // Since freed slot 0 has capacity 32 < 48, it MUST NOT reuse slot 0!
        let idx1 = arena.allocate_node(4, m, &[vec![1], vec![2], vec![3], vec![4], vec![5]])?;
        assert_eq!(
            idx1, 1,
            "Slot 0 should not be reused due to insufficient capacity"
        );

        Ok(())
    }

    #[test]
    fn test_backlink_table_operations() {
        let mut backlinks = BacklinkTable::new();
        backlinks.insert(5, 0, vec![10, 20, 30]);

        let retrieved = backlinks.get(5, 0);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &vec![10, 20, 30]);

        assert!(backlinks.get(5, 1).is_none());
        assert!(backlinks.get(99, 0).is_none());
    }
}
