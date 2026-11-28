// FILE-CONTEXT
// ZWECK: Arena-Allocator (HnswArena) für HNSW-Nachbarlisten mit 64-Byte-Alignment, Slot-Freilisten und CAS-Relinking.
// INVARIANTEN: Zero-Panic Doctrine (ContextraError bei Kapazität/Grenzen); SIMD-Cacheline Alignment (64 Bytes).

//! Arena Allocator for HNSW neighbor lists and O(1) Backlink management.

use contextra_types::{ContextraError, Result};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Slot alignment in bytes (64-byte alignment = 16 u32 elements for SIMD cache-line compatibility).
pub const ARENA_ALIGNMENT_BYTES: usize = 64;
/// Alignment in u32 elements (64 bytes / 4 bytes = 16 u32 elements).
pub const ARENA_ALIGNMENT_U32: usize = ARENA_ALIGNMENT_BYTES / std::mem::size_of::<u32>();

/// Composite key (neighbor_ram_idx, layer) for backlink lookups.
pub type BacklinkKey = (usize, usize);

/// Deterministic backlink connection table using `BTreeMap`.
#[derive(Debug, Default, Clone)]
pub struct BacklinkTable {
    pub map: BTreeMap<BacklinkKey, Vec<u32>>,
}

impl BacklinkTable {
    /// Creates a new `BacklinkTable`.
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
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
    /// Per-slot liveness. Guards `free_node` against double release (F-02).
    /// Lock order: offsets -> capacities -> count_offsets -> counts -> arena -> free_list -> alive.
    pub alive: RwLock<Vec<bool>>,
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
            alive: RwLock::new(Vec::new()),
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
        if final_connections.len() > max_layer + 1 {
            return Err(ContextraError::invalid_input(
                "Node has more layers than max_layer + 1 allows",
            ));
        }
        // Layer lengths are stored as u8 counts; reject configurations where they could truncate.
        if m * 2 > usize::from(u8::MAX) {
            return Err(ContextraError::invalid_input(
                "HNSW m too large: neighbor counts must fit in u8",
            ));
        }
        let layers_needed = final_connections.len();
        let node_capacity = Self::align_capacity(raw_capacity);

        let mut offsets = self.offsets.write();
        let mut capacities = self.capacities.write();
        let mut count_offsets = self.count_offsets.write();
        let mut counts = self.counts.write();
        let mut arena = self.arena.write();
        let mut free_list = self.free_list.write();
        let mut alive = self.alive.write();

        // A freed slot is reusable only if it can hold both the arena block AND the
        // new node's full layer-count region (F-01: otherwise counts spill into the
        // neighbouring slot's region).
        let mut found_free_index = None;
        for (i, &freed_idx) in free_list.iter().enumerate() {
            if freed_idx < capacities.len()
                && capacities[freed_idx] >= node_capacity
                && Self::slot_layer_count(count_offsets.as_slice(), counts.len(), freed_idx)
                    >= layers_needed
            {
                found_free_index = Some(i);
                break;
            }
        }

        let (ram_idx, start_offset) = if let Some(free_pos) = found_free_index {
            let reused_idx = free_list.remove(free_pos);
            let region = Self::slot_layer_count(count_offsets.as_slice(), counts.len(), reused_idx);
            let cs = count_offsets.get(reused_idx).copied().unwrap_or(0);
            // Clear counts of layers this node does not use, so no stale length survives.
            for c in counts
                .iter_mut()
                .skip(cs + layers_needed)
                .take(region.saturating_sub(layers_needed))
            {
                *c = 0;
            }
            (reused_idx, offsets.get(reused_idx).copied().unwrap_or(0))
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
            count_offsets.push(counts.len());
            let new_len = counts.len() + layers_needed;
            counts.resize(new_len, 0);
            alive.push(false);
            (offsets.len() - 1, start)
        };
        if let Some(live) = alive.get_mut(ram_idx) {
            *live = true;
        }

        // Invariant: the slot's count region holds at least `layers_needed` entries
        // (ensured above for both new and reused slots), so every write stays inside it.
        let count_start = count_offsets.get(ram_idx).copied().unwrap_or(0);
        for (layer, layer_conns) in final_connections.iter().enumerate() {
            let l_offset = Self::layer_offset(start_offset, layer, m);
            let layer_cap = if layer == 0 { m * 2 } else { m };
            let len = layer_conns.len().min(layer_cap);
            if l_offset + len > arena.len() {
                arena.resize(l_offset + len, 0);
            }
            arena[l_offset..l_offset + len].copy_from_slice(&layer_conns[..len]);
            let slot = counts
                .get_mut(count_start + layer)
                .ok_or_else(|| ContextraError::Index("Layer count region overflow".to_string()))?;
            *slot = len as u8;
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

    /// Zeroizes connection memory for a node slot across all layers and clears counts,
    /// without releasing to `free_list` (`INV-VEC-ZEROIZE-1`, `INV-HNSW-SLOT-1`).
    pub fn zeroize_node_connections(&self, ram_idx: usize, max_layer: usize, m: usize) {
        let offsets = self.offsets.read();
        let count_offsets = self.count_offsets.read();
        let mut counts = self.counts.write();
        let mut arena = self.arena.write();

        if ram_idx < offsets.len() && ram_idx < count_offsets.len() {
            let node_offset = offsets[ram_idx];
            let count_start = count_offsets[ram_idx];
            let count_end = if ram_idx + 1 < count_offsets.len() {
                count_offsets[ram_idx + 1]
            } else {
                counts.len()
            };

            for layer in 0..=max_layer {
                if count_start + layer < count_end {
                    let l_offset = Self::layer_offset(node_offset, layer, m);
                    let layer_cap = if layer == 0 { m * 2 } else { m };
                    if l_offset + layer_cap <= arena.len() {
                        use zeroize::Zeroize;
                        arena[l_offset..l_offset + layer_cap].zeroize();
                    }
                    counts[count_start + layer] = 0;
                }
            }
        }
    }

    /// Zeroizes connection memory for a node slot across all layers, clears counts,
    /// and then marks the node's slot as free in the free_list.
    pub fn zeroize_and_free_node(&self, ram_idx: usize, max_layer: usize, m: usize) {
        self.zeroize_node_connections(ram_idx, max_layer, m);
        self.free_node(ram_idx);
    }

    /// Marks a node's slot as free for future slot reuse.
    ///
    /// Idempotent: only a live slot is released. A second call for the same slot is a
    /// no-op, so the slot cannot be handed out twice and `total_allocated` stays exact (F-02).
    pub fn free_node(&self, ram_idx: usize) {
        let offsets = self.offsets.read();
        if ram_idx >= offsets.len() {
            return;
        }
        let mut free_list = self.free_list.write();
        let mut alive = self.alive.write();
        match alive.get_mut(ram_idx) {
            Some(live) if *live => {
                *live = false;
                free_list.push(ram_idx);
                self.total_allocated.fetch_sub(1, Ordering::SeqCst);
            }
            _ => {}
        }
    }

    /// Number of layer-count entries owned by `ram_idx`: the region from its own
    /// `count_offsets` entry up to the next slot's entry, or to the end for the last slot.
    fn slot_layer_count(count_offsets: &[usize], counts_len: usize, ram_idx: usize) -> usize {
        let Some(&start) = count_offsets.get(ram_idx) else {
            return 0;
        };
        let end = count_offsets
            .get(ram_idx + 1)
            .copied()
            .unwrap_or(counts_len);
        end.saturating_sub(start)
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

    /// F-01: reusing a slot that held fewer layers must not write the new node's
    /// layer counts into the region of the neighbouring slot.
    #[test]
    fn test_slot_reuse_does_not_clobber_neighbor_counts() -> Result<()> {
        let arena = HnswArena::new();
        let m = 8;
        let a = arena.allocate_node(1, m, &[vec![1], vec![2]])?;
        let b = arena.allocate_node(0, m, &[vec![10, 11, 12]])?;
        arena.free_node(a);
        let _c = arena.allocate_node(2, m, &[vec![3], vec![4], vec![5]])?;
        assert_eq!(arena.get_ram_node_connections(b, 0, m), vec![10, 11, 12]);
        Ok(())
    }

    /// F-02: freeing the same slot twice must not hand it out twice.
    #[test]
    fn test_double_free_does_not_alias_slots() -> Result<()> {
        let arena = HnswArena::new();
        let a = arena.allocate_node(0, 8, &[vec![1]])?;
        arena.free_node(a);
        arena.free_node(a);
        let x = arena.allocate_node(0, 8, &[vec![7]])?;
        let y = arena.allocate_node(0, 8, &[vec![9]])?;
        assert_ne!(x, y);
        Ok(())
    }

    /// F-02: the second free is a no-op and must not decrement the live count.
    #[test]
    fn test_double_free_keeps_total_allocated_consistent() -> Result<()> {
        let arena = HnswArena::new();
        let a = arena.allocate_node(0, 8, &[vec![1]])?;
        assert_eq!(arena.total_allocated.load(Ordering::SeqCst), 1);
        arena.free_node(a);
        arena.free_node(a);
        assert_eq!(arena.total_allocated.load(Ordering::SeqCst), 0);
        Ok(())
    }
}
