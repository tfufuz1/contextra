// FILE-CONTEXT
// ZWECK: Additiver, rein lesender Prototyp für prädikatsbewusste Graph-Traversierung (ACORN-γ-Prinzip).
// INVARIANTEN: Rein additiv & lesend, verändert den bestehenden Graph/Index NICHT.
//              Bei gamma = 0 verhält sich die Suche identisch zu search_filtered_internal.
//              Deterministische Sortierung (Score absteigend, DocId aufsteigend), Schranke MAX_SEARCH_K.

use super::types::{DiskAnnIndex, SearchCandidate};
use contextra_core::{ContextraError, DistanceMetric, DocId, Result, ScoredDocument};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

impl DiskAnnIndex {
    /// Executes a predicate-augmented vector search using ACORN 2-hop neighbor expansion during graph traversal.
    ///
    /// # Parameters
    /// - `query`: Query vector slice.
    /// - `k`: Number of top documents requested.
    /// - `filter`: Predicate closure for candidate document filtering.
    /// - `gamma`: Expansion degree: up to `gamma` predicate-satisfying 2-hop neighbors are evaluated per step.
    ///
    /// # Safety & Read-Only Guarantee
    /// This method is purely additive and read-only. It does NOT modify or rewrote any existing graph or index data.
    /// When `gamma == 0`, traversal expansion is strictly 1-hop and behaves as documented by standard post-filtered search.
    #[allow(clippy::unnecessary_cast)]
    pub async fn search_predicate_augmented(
        &self,
        query: &[f32],
        k: usize,
        filter: &(dyn Fn(DocId) -> bool + Send + Sync),
        gamma: usize,
    ) -> Result<Vec<ScoredDocument>> {
        if k == 0 || k > contextra_core::MAX_SEARCH_K {
            return Err(ContextraError::invalid_input(format!(
                "Requested k ({k}) is invalid or exceeds maximum allowed search limit ({})",
                contextra_core::MAX_SEARCH_K
            )));
        }

        if !query.iter().all(|v| v.is_finite()) {
            return Err(ContextraError::invalid_input(
                "Query vector contains non-finite (NaN or Infinity) components".to_string(),
            ));
        }

        let header = match *self.inner.header.read() {
            Some(h) => h,
            None => return Err(ContextraError::Index("Index not loaded".into())),
        };

        if header.node_count == 0 {
            return Ok(Vec::new());
        }

        let beam_width = self.inner.config.beam_width.max(k * 4);
        let metric = self.inner.config.distance_metric;
        let tombstones = self.inner.tombstones.read();

        let ep = header.entry_point;
        let mut visited = HashSet::new();

        let ep_node = self.load_node(ep)?;
        let ep_dist = self.get_dist_to_query(query, ep)?;

        if !ep_dist.is_finite() {
            return Err(ContextraError::Index(
                "Non-finite distance encountered for entry point in search_predicate_augmented".into(),
            ));
        }

        let ep_passes = !tombstones.contains(ep_node.doc_id.inner() as u64) && filter(ep_node.doc_id);

        let mut queue = BinaryHeap::new(); // Min-heap by distance ( Reverse(SearchCandidate) )
        let mut matching_results = Vec::new();

        visited.insert(ep);

        let ep_cand = SearchCandidate {
            index: ep,
            distance: ep_dist,
        };
        queue.push(Reverse(ep_cand));

        if ep_passes {
            matching_results.push((ep_node.doc_id, ep_dist));
        }

        while let Some(Reverse(current)) = queue.pop() {
            // If we already have enough matching results and queue length exceeds beam width, stop early.
            if matching_results.len() >= k * 4 && queue.len() > beam_width {
                break;
            }

            let curr_node = self.load_node(current.index)?;

            // 1-Hop neighbors
            for &nbr in &curr_node.neighbors {
                if !visited.insert(nbr) {
                    continue;
                }

                let nbr_node = self.load_node(nbr)?;
                let nbr_dist = self.get_dist_to_query(query, nbr)?;
                if !nbr_dist.is_finite() {
                    continue;
                }

                let nbr_passes = !tombstones.contains(nbr_node.doc_id.inner() as u64) && filter(nbr_node.doc_id);

                if nbr_passes {
                    matching_results.push((nbr_node.doc_id, nbr_dist));
                }

                queue.push(Reverse(SearchCandidate {
                    index: nbr,
                    distance: nbr_dist,
                }));

                // 2-Hop ACORN-gamma expansion: if gamma > 0, inspect 2-hop neighbors of nbr
                if gamma > 0 {
                    let mut added_2hop = 0;
                    for &hop2_idx in &nbr_node.neighbors {
                        if added_2hop >= gamma {
                            break;
                        }
                        if !visited.contains(&hop2_idx) {
                            let hop2_node = self.load_node(hop2_idx)?;
                            let hop2_passes = !tombstones.contains(hop2_node.doc_id.inner() as u64)
                                && filter(hop2_node.doc_id);

                            if hop2_passes {
                                visited.insert(hop2_idx);
                                let hop2_dist = self.get_dist_to_query(query, hop2_idx)?;
                                if hop2_dist.is_finite() {
                                    matching_results.push((hop2_node.doc_id, hop2_dist));
                                    queue.push(Reverse(SearchCandidate {
                                        index: hop2_idx,
                                        distance: hop2_dist,
                                    }));
                                    added_2hop += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Deduplicate matching results by DocId, retaining best distance
        let mut best_matching: std::collections::HashMap<DocId, f32> = std::collections::HashMap::new();
        for (doc_id, dist) in matching_results {
            best_matching
                .entry(doc_id)
                .and_modify(|d| {
                    if dist < *d {
                        *d = dist;
                    }
                })
                .or_insert(dist);
        }

        let mut scored_docs: Vec<ScoredDocument> = best_matching
            .into_iter()
            .map(|(doc_id, dist)| {
                let score = match metric {
                    DistanceMetric::Cosine => 1.0 - dist,
                    DistanceMetric::Euclidean => 1.0 / (1.0 + dist),
                    DistanceMetric::DotProduct => -dist,
                    _ => 1.0 / (1.0 + dist),
                };
                ScoredDocument::new(doc_id, score)
            })
            .collect();

        // Deterministic sorting: score descending, doc_id ascending
        scored_docs.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        scored_docs.truncate(k);
        Ok(scored_docs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diskann::config::DiskAnnConfig;
    use contextra_core::{DistanceMetric, DocId, VectorIndex};

    #[tokio::test]
    async fn test_gamma_zero_regression_reduction() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(ContextraError::Io)?;
        let config = DiskAnnConfig {
            index_path: temp_dir.path().join("gamma_zero.idx"),
            dimension: 4,
            max_degree: 8,
            beam_width: 16,
            distance_metric: DistanceMetric::Euclidean,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(config)?;
        let n = 100;
        let mut vecs = Vec::new();
        let mut ids = Vec::new();
        for i in 0..n {
            vecs.push(vec![i as f32, 0.0, 0.0, 0.0]);
            ids.push(DocId::from((i + 1) as u64));
        }
        index.build(&vecs, &ids).await?;

        let query = vec![10.0, 0.0, 0.0, 0.0];
        let filter = |id: DocId| id.inner() % 2 == 0;

        let filtered_res = index.search_filtered(&query, 5, Some(&filter)).await?;
        let augmented_res = index.search_predicate_augmented(&query, 5, &filter, 0).await?;

        assert!(!filtered_res.is_empty());
        assert_eq!(filtered_res.len(), augmented_res.len());
        for (f, a) in filtered_res.iter().zip(augmented_res.iter()) {
            assert_eq!(f.doc_id, a.doc_id);
            assert!((f.score - a.score).abs() < 1e-5);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_low_selectivity_augmented_improves_recall() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(ContextraError::Io)?;
        let config = DiskAnnConfig {
            index_path: temp_dir.path().join("low_selectivity.idx"),
            dimension: 4,
            max_degree: 8,
            beam_width: 16,
            distance_metric: DistanceMetric::Euclidean,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(config)?;
        let n = 500;
        let mut vecs = Vec::new();
        let mut ids = Vec::new();
        for i in 0..n {
            vecs.push(vec![i as f32, 0.0, 0.0, 0.0]);
            ids.push(DocId::from((i + 1) as u64));
        }
        index.build(&vecs, &ids).await?;

        let query = vec![250.0, 0.0, 0.0, 0.0];
        // Low selectivity: < 2% matching documents (only DocIds divisible by 60)
        let filter = |id: DocId| id.inner() % 60 == 0;

        let filtered_res = index.search_filtered(&query, 5, Some(&filter)).await?;
        let augmented_res = index.search_predicate_augmented(&query, 5, &filter, 2).await?;

        assert!(
            augmented_res.len() >= filtered_res.len(),
            "Predicate-augmented search with gamma > 0 must return at least as many or more valid candidates"
        );

        Ok(())
    }
}
