// FILE-CONTEXT
// ZWECK: Schichtbasierte Graph-Traversierung (Layer Search) mit Backlink- und Kontextanbindung.
// INVARIANTEN: Zero-Panic; deterministische Auswertung; Kantenauflösung über SearchContext.
// HOTSPOTS: search_layer_with_context
// STAND: 2026-11-12

//! Schichtbasierte Graph-Traversierung im HNSW-Index.

use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::atomic::Ordering;

use crate::error::Result;
use crate::fixed_hasher::new_fixed_hashset;

use super::super::arena::BacklinkTable;
use super::super::batch::{PreparedInsert, SearchContext};
use super::super::types::{Candidate, HnswIndexCore};

impl HnswIndexCore {
    /// Sucht auf einer bestimmten Ebene nach den `ef` nächsten Nachbarn.
    pub(crate) fn search_layer(
        &self,
        query: &[f32],
        query_quantized: Option<&[u8]>,
        entry_points: &[usize],
        ef: usize,
        layer: usize,
    ) -> Result<Vec<Candidate>> {
        self.search_layer_with_context(query, query_quantized, entry_points, ef, layer, &[], None)
    }

    // ANCHOR[contextra-vector:HNSW-SEARCH-LAYER-CTX]
    /// Führt die Schichtsuche mit Berücksichtigung vorheriger vorbereiteter Einfügungen aus.
    pub(crate) fn search_layer_with_context(
        &self,
        query: &[f32],
        query_quantized: Option<&[u8]>,
        entry_points: &[usize],
        ef: usize,
        layer: usize,
        prior_prepared: &[PreparedInsert],
        backlink_map: Option<&BacklinkTable>,
    ) -> Result<Vec<Candidate>> {
        let nodes_guard = self.hot.nodes.read();
        let mmap_guard = self.cold.mmap_index.read();
        let mmap_node_count = mmap_guard
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let q_guard = if self.cold.config.quantize {
            Some(self.cold.quantizer.read())
        } else {
            None
        };
        let q_ref = q_guard.as_ref().and_then(|g| g.as_ref());

        let ctx = SearchContext {
            nodes: &nodes_guard,
            mmap: mmap_guard.as_ref(),
            mmap_node_count,
            prior_prepared,
            backlink_map,
            quantizer: q_ref.map(Cow::Borrowed),
            arena: &self.hot.arena,
        };

        let mut visited = new_fixed_hashset(ef.saturating_mul(4));
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();

        #[cfg(feature = "partial-index-rebuild")]
        let mut visited_node_ids = Vec::new();

        for &ep in entry_points {
            if visited.insert(ep) {
                #[cfg(feature = "partial-index-rebuild")]
                visited_node_ids.push(ep as u64);

                let dist = self.resolve_dist(ep, query, query_quantized, &ctx)?;
                let cand = Candidate {
                    index: ep,
                    distance: dist,
                };
                candidates.push(Reverse(cand));
                results.push(cand);
            }
        }

        let deleted_snapshot = self.cold.deleted_nodes.read();

        while let Some(Reverse(current)) = candidates.pop() {
            if let Some(worst_result) = results.peek() {
                if current.distance > worst_result.distance && results.len() >= ef {
                    break;
                }
            }

            let connections = self.resolve_connections(current.index, layer, &ctx)?;
            let mut has_dead_neighbors = false;

            for &neighbor_u32 in connections.iter() {
                let neighbor = neighbor_u32 as usize;
                if deleted_snapshot.contains(neighbor as u64) {
                    self.cold.visited_dead_nodes.fetch_add(1, Ordering::Relaxed);
                    has_dead_neighbors = true;
                }
                if visited.insert(neighbor) {
                    #[cfg(feature = "partial-index-rebuild")]
                    visited_node_ids.push(neighbor as u64);

                    let dist = self.resolve_dist(neighbor, query, query_quantized, &ctx)?;
                    let is_better = match results.peek() {
                        Some(worst) => dist < worst.distance,
                        None => true,
                    };

                    if is_better || results.len() < ef {
                        let cand = Candidate {
                            index: neighbor,
                            distance: dist,
                        };
                        candidates.push(Reverse(cand));
                        results.push(cand);
                        if results.len() > ef {
                            results.pop();
                        }
                    }
                }
            }

            if has_dead_neighbors && current.index >= mmap_node_count {
                let ram_idx = current.index - mmap_node_count;
                let seq_log = self.cold.seq_log.read();
                let min_retention_seq = seq_log.min_retention_seq();
                let m = self.cold.config.m;
                let layer_slice = self.hot.arena.get_ram_node_connections(ram_idx, layer, m);
                let mut kept = Vec::with_capacity(layer_slice.len());
                for &neighbor_u32 in &layer_slice {
                    if !deleted_snapshot.contains(neighbor_u32 as u64) {
                        kept.push(neighbor_u32);
                    } else if let Some(min_ret_seq) = min_retention_seq {
                        let neighbor_idx = neighbor_u32 as usize;
                        if neighbor_idx >= mmap_node_count {
                            let neighbor_ram_idx = neighbor_idx - mmap_node_count;
                            if let Some(neighbor_node) = nodes_guard.get(neighbor_ram_idx) {
                                if let Some(del_seq) = seq_log.deletion_seq(neighbor_node.doc_id) {
                                    if del_seq >= min_ret_seq {
                                        kept.push(neighbor_u32);
                                    }
                                }
                            }
                        }
                    }
                }
                self.hot
                    .arena
                    .try_relink_pruned_neighbors(ram_idx, layer, m, &kept);
            }
        }
        #[cfg(feature = "partial-index-rebuild")]
        if layer == 0 && !visited_node_ids.is_empty() {
            if let Some(ref tracker) = self.cold.traversal_tracker {
                tracker.write().record_traversal(visited_node_ids);
            }
        }

        let mut vec = results.into_vec();
        vec.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        Ok(vec)
    }
}
