// FILE-CONTEXT
// ZWECK: Lokaler partieller Rebuild von Knotenregionen und Bereinigung von Tombstones in der Nachbarschaft.
// INVARIANTEN: Lock-Hierarchie: write_mutex -> arena write lock.
// HOTSPOTS: rebuild_region_sync
// STAND: TS:2026-11-11T02:00:00Z

use std::collections::BTreeSet;
use std::sync::atomic::Ordering;

use crate::error::Result;
use crate::hnsw::arena::HnswArena;
use crate::hnsw::types::HnswIndexCore;

impl HnswIndexCore {
    /// Startet einen partiellen Rebuild für eine bestimmte Menge von Knoten.
    pub fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()> {
        self.rebuild_region_sync(region_node_ids)
    }

    /// Führt die lokale Reorganisation einer Region synchron durch.
    pub fn rebuild_region_sync(&self, region_node_ids: Vec<u64>) -> Result<()> {
        if region_node_ids.is_empty() {
            return Ok(());
        }

        let _write_lock = self.hot.write_mutex.lock();

        let region_set: BTreeSet<u64> = region_node_ids.into_iter().collect();

        let mmap_count = self
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let nodes = self.hot.nodes.read();
        let mut deleted_nodes = self.cold.deleted_nodes.write();

        let mut tombstoned_in_region = Vec::new();
        for &node_id in &region_set {
            if deleted_nodes.contains(node_id) {
                tombstoned_in_region.push(node_id);
            }
        }

        if tombstoned_in_region.is_empty() {
            return Ok(());
        }

        let tombstoned_set: BTreeSet<u32> =
            tombstoned_in_region.iter().map(|&id| id as u32).collect();

        let m = self.cold.config.m;
        let offsets = self.hot.arena.offsets.read();
        let count_offsets = self.hot.arena.count_offsets.read();
        let mut counts = self.hot.arena.counts.write();
        let mut arena = self.hot.arena.arena.write();

        for (i, node) in nodes.iter().enumerate() {
            let global_idx = mmap_count + i;
            if deleted_nodes.contains(global_idx as u64) {
                continue;
            }

            if i < offsets.len() && i < count_offsets.len() {
                let node_offset = offsets[i];
                let count_start = count_offsets[i];
                let count_end = if i + 1 < count_offsets.len() {
                    count_offsets[i + 1]
                } else {
                    counts.len()
                };
                for layer in 0..node.max_layer + 1 {
                    if count_start + layer < count_end {
                        let l_offset = HnswArena::layer_offset(node_offset, layer, m);
                        let old_len = counts[count_start + layer] as usize;
                        if l_offset + old_len <= arena.len() {
                            let layer_slice = &arena[l_offset..l_offset + old_len];
                            let mut kept = Vec::with_capacity(old_len);
                            for &neighbor_u32 in layer_slice {
                                if !tombstoned_set.contains(&neighbor_u32) {
                                    kept.push(neighbor_u32);
                                }
                            }
                            counts[count_start + layer] = kept.len() as u8;
                            arena[l_offset..l_offset + kept.len()].copy_from_slice(&kept);
                        }
                    }
                }
            }
        }

        let mut doc_map = self.hot.doc_to_node.write();
        for &ts_id in &tombstoned_in_region {
            if ts_id >= mmap_count as u64 {
                let ram_idx = (ts_id as usize) - mmap_count;
                if let Some(node) = nodes.get(ram_idx) {
                    doc_map.remove(&node.doc_id.inner());
                }
            }
            deleted_nodes.insert(ts_id);
        }

        self.hot
            .deleted_count
            .store(deleted_nodes.len(), Ordering::SeqCst);

        tracing::info!(
            rebuilt_region_nodes = region_set.len(),
            pruned_tombstones = tombstoned_in_region.len(),
            "HNSW local partial rebuild completed successfully"
        );

        Ok(())
    }
}
