use ahash::AHashSet;
use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::atomic::Ordering;

use contextra_core::{ContextraError, DistanceMetric, DocId, Result, ScoredDocument};

use super::arena::BacklinkTable;
use super::batch::{PreparedInsert, SearchContext};
use super::types::{Candidate, HnswIndex, HnswIndexCore, VectorData};

impl HnswIndexCore {
    pub(super) fn search_layer(
        &self,
        query: &[f32],
        query_quantized: Option<&[u8]>,
        entry_points: &[usize],
        ef: usize,
        layer: usize,
    ) -> Result<Vec<Candidate>> {
        self.search_layer_with_context(query, query_quantized, entry_points, ef, layer, &[], None)
    }

    pub(super) fn search_layer_with_context(
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

        let mut visited = AHashSet::with_capacity(ef.saturating_mul(4));
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
            self.cold
                .traversal_tracker
                .write()
                .record_traversal(visited_node_ids);
        }

        let mut vec = results.into_vec();
        vec.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        Ok(vec)
    }
}

impl HnswIndex {
    pub(super) async fn search_filtered_internal(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
        snapshot_seq: Option<u64>,
    ) -> Result<Vec<ScoredDocument>> {
        if let Some(ref err) = self.inner.cold.validation_error {
            return Err(ContextraError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }
        if query.len() != self.inner.cold.config.dimension {
            return Err(ContextraError::invalid_input(format!(
                "Expected dimension {}, got {}",
                self.inner.cold.config.dimension,
                query.len()
            )));
        }

        if k == 0 {
            return Ok(Vec::new());
        }
        if k > contextra_core::MAX_SEARCH_K {
            return Err(ContextraError::invalid_input(format!(
                "Requested k ({k}) exceeds maximum allowed search limit ({}). \
                 Use Collection::query() with appropriate k bounds.",
                contextra_core::MAX_SEARCH_K
            )));
        }
        for (i, &val) in query.iter().enumerate() {
            if !val.is_finite() {
                return Err(ContextraError::invalid_input(format!(
                    "Query vector element at index {i} is not finite (value: {val}). \
                     NaN/Inf values corrupt HNSW distance computation and heap ordering. \
                     Validate embedding outputs before search."
                )));
            }
        }

        let query_quantized = if self.inner.cold.config.quantize {
            self.inner
                .cold
                .quantizer
                .read()
                .as_ref()
                .map(|q| q.quantize(query))
                .transpose()?
        } else {
            None
        };

        let mut ep = Vec::new();
        if let Some(global_ep) = self.inner.hot.get_entry_point() {
            ep.push(global_ep);
        }
        if let Some(ram_ep) = self.inner.hot.get_ram_entry_point() {
            if !ep.contains(&ram_ep) {
                ep.push(ram_ep);
            }
        }

        if ep.is_empty() {
            let nodes = self.inner.hot.nodes.read();
            let deleted = self.inner.cold.deleted_nodes.read();
            let mmap_guard = self.inner.cold.mmap_index.read();
            let mmap_node_count = mmap_guard
                .as_ref()
                .map(|m| m.header.node_count() as usize)
                .unwrap_or(0);
            let total = mmap_node_count + nodes.len();
            for i in 0..total {
                if !deleted.contains(i as u64) {
                    ep.push(i);
                    self.inner.hot.set_entry_point(Some(i));
                    break;
                }
            }
        }

        let nodes = self.inner.hot.nodes.read();
        let deleted = self.inner.cold.deleted_nodes.read();

        let mut filter_eps = Vec::new();
        if let Some(f) = filter {
            let mmap_guard = self.inner.cold.mmap_index.read();
            let mmap_node_count = mmap_guard
                .as_ref()
                .map(|m| m.header.node_count() as usize)
                .unwrap_or(0);
            let q_guard = if self.inner.cold.config.quantize {
                Some(self.inner.cold.quantizer.read())
            } else {
                None
            };
            let q_ref = q_guard.as_ref().and_then(|g| g.as_ref());
            let ctx = SearchContext {
                nodes: &nodes,
                mmap: mmap_guard.as_ref(),
                mmap_node_count,
                prior_prepared: &[],
                backlink_map: None,
                quantizer: q_ref.map(Cow::Borrowed),
                arena: &self.inner.hot.arena,
            };

            let factor = if self.inner.cold.config.quantize {
                4
            } else {
                2
            };
            let max_filter_eps = self.inner.cold.config.ef_search.max(k) * factor;

            let total_nodes = mmap_node_count + nodes.len();
            for i in (0..total_nodes).rev() {
                if !ep.contains(&(i as _)) {
                    if snapshot_seq.is_none() && deleted.contains(i as u64) {
                        continue;
                    }
                    if let Ok(doc_id) = self.inner.resolve_doc_id(i, &ctx) {
                        if f(doc_id) {
                            ep.push(i);
                            filter_eps.push(i);
                            if filter_eps.len() >= max_filter_eps {
                                break;
                            }
                        }
                    }
                }
            }
        }

        if ep.is_empty() {
            return Ok(Vec::new());
        }

        let max_layer = self.inner.hot.max_layer.load(Ordering::SeqCst) as usize;

        for layer in (1..=max_layer).rev() {
            let best = self
                .inner
                .search_layer(query, query_quantized.as_deref(), &ep, 1, layer)?;
            if let Some(closest) = best.first() {
                ep = vec![closest.index];
            }
        }

        if let Some(ram_ep) = self.inner.hot.get_ram_entry_point() {
            if !ep.contains(&ram_ep) {
                ep.push(ram_ep);
            }
        }

        for f_ep in filter_eps {
            if !ep.contains(&f_ep) {
                ep.push(f_ep);
            }
        }

        let factor = if self.inner.cold.config.quantize {
            4
        } else {
            2
        };
        let ef = self.inner.cold.config.ef_search.max(k) * factor;
        let candidates = self
            .inner
            .search_layer(query, query_quantized.as_deref(), &ep, ef, 0)?;

        let score = self.inner.connectivity_score();
        if score < self.inner.cold.config.rebuild_threshold {
            let deleted_ratio = (1.0 - score) * 100.0;
            let err = contextra_core::ContextraError::HnswConnectivityDegraded { deleted_ratio };
            tracing::warn!(
                error = %err,
                connectivity_score = score,
                rebuild_threshold = self.inner.cold.config.rebuild_threshold,
                "HNSW index degraded — consider calling rebuild()"
            );
        }

        let mut results = Vec::with_capacity(k);

        let seq_log_guard = if snapshot_seq.is_some() {
            Some(self.inner.cold.seq_log.read())
        } else {
            None
        };

        for c in candidates.iter() {
            let node = nodes.get(c.index).ok_or_else(|| {
                ContextraError::Index(format!("HNSW candidate node missing at index {}", c.index))
            })?;
            if node.committed_tx == 0 {
                continue;
            }
            let doc_id = node.doc_id;

            if let Some(snap_seq) = snapshot_seq {
                if let Some(ref seq_log) = seq_log_guard {
                    if !seq_log.is_visible(doc_id, snap_seq) {
                        continue;
                    }
                }
            } else if deleted.contains(c.index as u64) {
                continue;
            }

            if let Some(f) = filter {
                if !f(doc_id) {
                    continue;
                }
            }

            let final_dist = if self.inner.cold.config.quantize {
                if let VectorData::U8(v) = &node.vector {
                    let guard = self.inner.cold.quantizer.read();
                    let q = guard.as_ref().ok_or_else(|| {
                        contextra_core::ContextraError::Index("Quantizer not trained".into())
                    })?;
                    q.asymmetric_dist(query, v, self.inner.cold.config.distance_metric)?
                } else {
                    c.distance
                }
            } else {
                c.distance
            };

            let score = match self.inner.cold.config.distance_metric {
                DistanceMetric::Cosine => 1.0 - final_dist,
                DistanceMetric::Euclidean => 1.0 / (1.0 + final_dist),
                DistanceMetric::DotProduct => -final_dist,
                other => {
                    return Err(ContextraError::Index(format!(
                        "Unsupported DistanceMetric variant in search_filtered(): {other:?}"
                    )));
                }
            };
            results.push(ScoredDocument::new(doc_id, score));
        }

        if results.len() > k {
            results.select_nth_unstable_by(k - 1, |a, b| {
                b.score
                    .total_cmp(&a.score)
                    .then_with(|| a.doc_id.cmp(&b.doc_id))
            });
            results.truncate(k);
        }
        results.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        Ok(results)
    }
}
