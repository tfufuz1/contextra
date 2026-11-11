// FILE-CONTEXT
// ZWECK: Gefilterte Vektorsuche und Snapshot-Sichtbarkeit im HNSW-Index.
// INVARIANTEN: Zero-Panic; Filter vor Distanz; Soft-Delete Respektierung.
// HOTSPOTS: search_filtered_internal
// STAND: 2026-11-12

//! Gefilterte Vektorsuche und Traversierung im HNSW-Index.

use std::borrow::Cow;
use std::sync::atomic::Ordering;

use crate::error::{Result, VectorError};
use crate::types::{DocId, ScoredDocument};

use super::super::batch::SearchContext;
use super::super::types::{HnswIndex, VectorData};
use super::rerank::{compute_distance_score, sort_and_truncate_results};
use super::validate::validate_search_params;

impl HnswIndex {
    /// Führt die interne gefilterte k-NN Suche aus.
    pub(crate) async fn search_filtered_internal(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
        snapshot_seq: Option<u64>,
    ) -> Result<Vec<ScoredDocument>> {
        validate_search_params(
            self.inner.cold.validation_error.as_ref(),
            self.inner.cold.config.dimension,
            query,
            k,
        )?;

        if k == 0 {
            return Ok(Vec::new());
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

        let mut filter_eps = Vec::new();
        if let Some(f) = filter {
            let nodes = self.inner.hot.nodes.read();
            let deleted = self.inner.cold.deleted_nodes.read();
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
            let err = crate::error::VectorError::ConnectivityDegraded { deleted_ratio };
            tracing::warn!(
                error = %err,
                connectivity_score = score,
                rebuild_threshold = self.inner.cold.config.rebuild_threshold,
                "HNSW index degraded — consider calling rebuild()"
            );
        }

        let mut results = Vec::with_capacity(k);

        let nodes = self.inner.hot.nodes.read();
        let deleted = self.inner.cold.deleted_nodes.read();

        let seq_log_guard = if snapshot_seq.is_some() {
            Some(self.inner.cold.seq_log.read())
        } else {
            None
        };

        for c in candidates.iter() {
            let node = nodes.get(c.index).ok_or_else(|| {
                VectorError::Index(format!("HNSW candidate node missing at index {}", c.index))
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
                        crate::error::VectorError::Index("Quantizer not trained".into())
                    })?;
                    q.asymmetric_dist(query, v, self.inner.cold.config.distance_metric)?
                } else {
                    c.distance
                }
            } else {
                c.distance
            };

            let score = compute_distance_score(self.inner.cold.config.distance_metric, final_dist)?;
            results.push(ScoredDocument::from_score(doc_id, score));
        }

        sort_and_truncate_results(&mut results, k);
        Ok(results)
    }
}
