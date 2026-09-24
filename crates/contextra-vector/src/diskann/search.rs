// FILE-CONTEXT
// ZWECK: DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (WP-4.3).
// INVARIANTEN: Lock-Hierarchie: header -> mmap -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Mmap für Vektor- & Graphlesezugriffe, ausgelagert an contextra-sys::mmap_readonly.
// STAND: TS:2026-09-10T19:30:00Z (SESSION: a9d67eae)

use super::config::{DiskAnnFallbackPolicy, VectorData};
use super::format::DiskAnnHeader;
use super::types::{DiskAnnIndex, SearchCandidate};
use crate::distance::compute_distance_trusted;
use contextra_core::{ContextraError, DistanceMetric, DocId, Result, ScoredDocument, VectorIndex};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

impl DiskAnnIndex {
    pub(crate) fn get_dist_to_query(&self, query: &[f32], node_idx: u32) -> Result<f32> {
        let node = self.load_node(node_idx)?;
        match &node.vector {
            VectorData::F32(v) => {
                compute_distance_trusted(query, v, self.inner.config.distance_metric)
            }
            VectorData::U8(v) => {
                let q_guard = self.inner.quantizer.read();
                let q = q_guard
                    .as_ref()
                    .ok_or_else(|| ContextraError::Index("Quantizer missing".into()))?;
                q.asymmetric_dist(query, v, self.inner.config.distance_metric)
            }
        }
    }

    pub(crate) fn search_in_memory(
        &self,
        query: &[f32],
        graph: &[Vec<u32>],
        vectors: &[Vec<f32>],
        entry_point: u32,
        beam_width: usize,
    ) -> Result<Vec<SearchCandidate>> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();

        let ep_dist = compute_distance_trusted(
            query,
            &vectors[entry_point as usize],
            self.inner.config.distance_metric,
        )?;
        if !ep_dist.is_finite() {
            // NAN-CHECK-OK
            tracing::error!(
                entry_point = entry_point,
                "DiskANN search_in_memory: non-finite distance for entry point"
            );
            return Err(ContextraError::Index(
                "Non-finite distance encountered for entry point in search_in_memory".into(),
            ));
        }

        let initial = SearchCandidate {
            index: entry_point,
            distance: ep_dist,
        };
        candidates.push(Reverse(initial.clone()));
        results.push(initial);
        visited.insert(entry_point);

        while let Some(Reverse(current)) = candidates.pop() {
            if let Some(worst) = results.peek() {
                if current.distance > worst.distance && results.len() >= beam_width {
                    break;
                }
            }

            for &neighbor in &graph[current.index as usize] {
                if !visited.insert(neighbor) {
                    continue;
                }
                let dist = compute_distance_trusted(
                    query,
                    &vectors[neighbor as usize],
                    self.inner.config.distance_metric,
                )?;
                if !dist.is_finite() {
                    // NAN-CHECK-OK
                    tracing::error!(
                        neighbor = neighbor,
                        "DiskANN search_in_memory: non-finite distance encountered for neighbor, skipping"
                    );
                    continue;
                }
                let cand = SearchCandidate {
                    index: neighbor,
                    distance: dist,
                };

                if results.len() < beam_width
                    || results.peek().map(|w| dist < w.distance).unwrap_or(true)
                {
                    candidates.push(Reverse(cand.clone()));
                    results.push(cand);
                    if results.len() > beam_width {
                        results.pop();
                    }
                }
            }
        }
        Ok(results.into_vec())
    }

    pub(crate) fn get_dist_mixed(
        &self,
        query: &[f32],
        idx: u32,
        existing_count: usize,
        new_vecs: &[(DocId, Vec<f32>)],
    ) -> Result<f32> {
        if (idx as usize) < existing_count {
            self.get_dist_to_query(query, idx)
        } else {
            let new_idx = idx as usize - existing_count;
            compute_distance_trusted(
                query,
                &new_vecs[new_idx].1,
                self.inner.config.distance_metric,
            )
        }
    }

    pub(crate) fn get_vec_mixed(
        &self,
        idx: u32,
        existing_count: usize,
        new_vecs: &[(DocId, Vec<f32>)],
    ) -> Result<Vec<f32>> {
        if (idx as usize) < existing_count {
            let node = self.load_node(idx)?;
            match node.vector {
                VectorData::F32(v) => Ok(v),
                VectorData::U8(v) => {
                    let q_guard = self.inner.quantizer.read();
                    let q = q_guard
                        .as_ref()
                        .ok_or_else(|| ContextraError::Index("Quantizer missing".into()))?;
                    q.dequantize(&v)
                }
            }
        } else {
            Ok(new_vecs[idx as usize - existing_count].1.clone())
        }
    }

    pub(crate) fn search_streaming(
        &self,
        query: &[f32],
        graph: &[Vec<u32>],
        existing_count: usize,
        new_vecs: &[(DocId, Vec<f32>)],
        entry_point: u32,
        beam_width: usize,
    ) -> Result<Vec<SearchCandidate>> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();

        let ep_dist = self.get_dist_mixed(query, entry_point, existing_count, new_vecs)?;
        if !ep_dist.is_finite() {
            // NAN-CHECK-OK
            tracing::error!(
                entry_point = entry_point,
                "DiskANN search_streaming: non-finite distance for entry point"
            );
            return Err(ContextraError::Index(
                "Non-finite distance encountered for entry point in search_streaming".into(),
            ));
        }

        let initial = SearchCandidate {
            index: entry_point,
            distance: ep_dist,
        };
        candidates.push(Reverse(initial.clone()));
        results.push(initial);
        visited.insert(entry_point);

        while let Some(Reverse(current)) = candidates.pop() {
            if let Some(worst) = results.peek() {
                if current.distance > worst.distance && results.len() >= beam_width {
                    break;
                }
            }

            for &neighbor in &graph[current.index as usize] {
                if !visited.insert(neighbor) {
                    continue;
                }
                let dist = self.get_dist_mixed(query, neighbor, existing_count, new_vecs)?;
                if !dist.is_finite() {
                    // NAN-CHECK-OK
                    tracing::error!(
                        neighbor = neighbor,
                        "DiskANN search_streaming: non-finite distance encountered for neighbor, skipping"
                    );
                    continue;
                }
                let cand = SearchCandidate {
                    index: neighbor,
                    distance: dist,
                };

                if results.len() < beam_width
                    || results.peek().map(|w| dist < w.distance).unwrap_or(true)
                {
                    candidates.push(Reverse(cand.clone()));
                    results.push(cand);
                    if results.len() > beam_width {
                        results.pop();
                    }
                }
            }
        }
        Ok(results.into_vec())
    }

    pub async fn search_internal(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>> {
        if k > contextra_core::MAX_SEARCH_K {
            return Err(ContextraError::invalid_input(format!(
                "Requested k ({}) exceeds maximum allowed search limit ({})",
                k,
                contextra_core::MAX_SEARCH_K
            )));
        }

        if !query.iter().all(|v| v.is_finite()) {
            return Err(ContextraError::invalid_input(
                "Query vector contains non-finite (NaN or Infinity) components".to_string(),
            ));
        }

        let fallback_opt = self.inner.hnsw_fallback.read().clone();
        if let Some(hnsw) = fallback_opt {
            return hnsw.search(query, k).await;
        }

        let header_opt = *self.inner.header.read();
        let header = match header_opt {
            Some(h) => h,
            None => {
                if self.inner.config.fallback_policy == DiskAnnFallbackPolicy::UseHnswOnFailure {
                    tracing::error!(
                        index_path = %self.inner.config.index_path.display(),
                        "DiskANN index not loaded or header missing — using HNSW fallback"
                    );
                    let hnsw = self.init_hnsw_fallback()?;
                    return hnsw.search(query, k).await;
                } else {
                    return Err(ContextraError::Index("Index not loaded".into()));
                }
            }
        };

        if header.node_count == 0 {
            return Ok(Vec::new());
        }

        self.check_quantizer_drift(query);

        let search_res = self.search_blocking(query, k, header);

        match search_res {
            Ok(res) => Ok(res),
            Err(err) => {
                if self.inner.config.fallback_policy == DiskAnnFallbackPolicy::UseHnswOnFailure {
                    tracing::error!(
                        index_path = %self.inner.config.index_path.display(),
                        error = %err,
                        "DiskANN search failed — falling back to HNSW"
                    );
                    let hnsw = self.init_hnsw_fallback()?;
                    hnsw.search(query, k).await
                } else {
                    Err(err)
                }
            }
        }
    }

    #[allow(clippy::unnecessary_cast)]
    pub(crate) fn search_blocking(
        &self,
        query: &[f32],
        k: usize,
        header: DiskAnnHeader,
    ) -> Result<Vec<ScoredDocument>> {
        let beam_width = self.inner.config.beam_width;
        let metric = self.inner.config.distance_metric;

        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();

        let ep = header.entry_point;
        let ep_dist = self.get_dist_to_query(query, ep)?;
        if !ep_dist.is_finite() {
            // NAN-CHECK-OK
            tracing::error!(
                entry_point = ep,
                "DiskANN search: non-finite distance for entry point"
            );
            return Err(ContextraError::Index(
                "Non-finite distance encountered for entry point in search".into(),
            ));
        }

        let initial_cand = SearchCandidate {
            index: ep,
            distance: ep_dist,
        };
        candidates.push(Reverse(initial_cand.clone()));
        results.push(initial_cand);
        visited.insert(ep);

        while let Some(Reverse(current)) = candidates.pop() {
            if let Some(worst) = results.peek() {
                if current.distance > worst.distance && results.len() >= beam_width {
                    break;
                }
            }

            let node = self.load_node(current.index)?;
            for &neighbor in &node.neighbors {
                if !visited.insert(neighbor) {
                    continue;
                }

                let dist = self.get_dist_to_query(query, neighbor)?;
                if !dist.is_finite() {
                    // NAN-CHECK-OK
                    tracing::error!(
                        neighbor = neighbor,
                        "DiskANN search: non-finite distance encountered for neighbor, skipping"
                    );
                    continue;
                }
                let cand = SearchCandidate {
                    index: neighbor,
                    distance: dist,
                };

                let is_better = if results.len() < beam_width {
                    true
                } else if let Some(worst) = results.peek() {
                    dist < worst.distance
                } else {
                    false
                };

                if is_better {
                    candidates.push(Reverse(cand.clone()));
                    results.push(cand);
                    if results.len() > beam_width {
                        results.pop();
                    }
                }
            }
        }

        let mut final_results = Vec::with_capacity(k);
        let mut sorted_results: Vec<SearchCandidate> = results.into_vec();
        if sorted_results.len() > beam_width {
            sorted_results
                .select_nth_unstable_by(beam_width - 1, |a, b| a.distance.total_cmp(&b.distance));
            sorted_results.truncate(beam_width);
        }
        sorted_results.sort_by(|a, b| a.distance.total_cmp(&b.distance));

        let tombstones = self.inner.tombstones.read();
        for c in sorted_results.into_iter() {
            if final_results.len() >= k {
                break;
            }
            let node = self.load_node(c.index)?;
            if tombstones.contains(node.doc_id.inner() as u64) {
                continue;
            }
            let score = match metric {
                DistanceMetric::Cosine => 1.0 - c.distance,
                DistanceMetric::Euclidean => 1.0 / (1.0 + c.distance),
                DistanceMetric::DotProduct => -c.distance,
                _ => 1.0 / (1.0 + c.distance),
            };
            final_results.push(ScoredDocument::new(node.doc_id, score));
        }

        Ok(final_results)
    }
}
