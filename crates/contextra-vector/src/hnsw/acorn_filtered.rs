// FILE-CONTEXT
// ZWECK: ACORN Prädikats-gefilterte Graph-Suche (Patel et al. 2024) für HnswIndex.
// INVARIANTEN: Zero Panic; Traversal über Brücken-Knoten gestattet; Zero Cross-Contamination im Ergebnis;
//              Hot-Path Kandidatenspeicher & Visited-Tracking über Vec.

use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::atomic::Ordering;

use contextra_core::DocId;

use crate::acorn::{AcornError, FilteredIndex};
use super::batch::SearchContext;
use super::types::{Candidate, HnswIndex};

#[derive(Clone, Copy, Debug, PartialEq)]
struct CandidateWithDoc {
    doc_id: DocId,
    distance: f32,
}

impl Eq for CandidateWithDoc {}

impl PartialOrd for CandidateWithDoc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CandidateWithDoc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance
            .total_cmp(&other.distance)
            .then_with(|| self.doc_id.cmp(&other.doc_id))
    }
}

impl FilteredIndex for HnswIndex {
    type Predicate = dyn Fn(DocId) -> bool;

    fn search_knn_acorn(
        &self,
        query: &[f32],
        k: usize,
        predicate: &Self::Predicate,
        gamma: u32,
    ) -> Result<Vec<(DocId, f32)>, AcornError> {
        if k == 0 {
            return Err(AcornError::InvalidK { k });
        }

        if gamma == 0 {
            return Err(AcornError::InvalidGamma(gamma));
        }

        let expected_dim = self.inner.cold.config.dimension;
        if query.len() != expected_dim {
            return Err(AcornError::DimensionMismatch {
                expected: expected_dim,
                got: query.len(),
            });
        }

        for (idx, &val) in query.iter().enumerate() {
            if !val.is_finite() {
                return Err(AcornError::Internal(format!(
                    "Query vector element at index {idx} is not finite ({val})"
                )));
            }
        }

        let nodes_guard = self.inner.hot.nodes.read();
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
            nodes: &nodes_guard,
            mmap: mmap_guard.as_ref(),
            mmap_node_count,
            prior_prepared: &[],
            backlink_map: None,
            quantizer: q_ref.map(Cow::Borrowed),
            arena: &self.inner.hot.arena,
        };

        let total_nodes = mmap_node_count.saturating_add(nodes_guard.len());
        if total_nodes == 0 {
            return Ok(Vec::new());
        }

        let mut ep = Vec::new();
        if let Some(global_ep) = self.inner.hot.get_entry_point() {
            ep.push(global_ep);
        }
        if let Some(ram_ep) = self.inner.hot.get_ram_entry_point() {
            if !ep.contains(&ram_ep) {
                ep.push(ram_ep);
            }
        }

        let deleted = self.inner.cold.deleted_nodes.read();

        if ep.is_empty() {
            for i in 0..total_nodes {
                if !deleted.contains(i as u64) {
                    ep.push(i);
                    self.inner.hot.set_entry_point(Some(i));
                    break;
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
                .search_layer(query, None, &ep, 1, layer)
                .map_err(|e| AcornError::Internal(e.to_string()))?;
            if let Some(closest) = best.first() {
                ep = vec![closest.index];
            }
        }

        if let Some(ram_ep) = self.inner.hot.get_ram_entry_point() {
            if !ep.contains(&ram_ep) {
                ep.push(ram_ep);
            }
        }

        // Augmented search beam size ef_acorn
        let base_ef = self.inner.cold.config.ef_search.max(k);
        let ef_acorn = base_ef.saturating_mul(gamma as usize).min(total_nodes.max(1));

        // Hot path: Visited tracking uses Vec<bool> (zero hash table allocator latency)
        let mut visited = vec![false; total_nodes];

        // Min-heap for search candidates expansion
        let mut candidates = BinaryHeap::<Reverse<Candidate>>::new();
        // Max-heap for tracking top ef_acorn search horizon distances
        let mut beam = BinaryHeap::<Candidate>::new();
        // Max-heap for keeping valid predicate-matching documents (bounded to keep best matches)
        let max_valid_capacity = ef_acorn.max(k.saturating_mul(10));
        let mut valid_matches = BinaryHeap::<CandidateWithDoc>::new();

        for &e in &ep {
            if e < total_nodes {
                let is_visited = visited.get(e).copied().unwrap_or(true);
                if !is_visited {
                    if let Some(visited_entry) = visited.get_mut(e) {
                        *visited_entry = true;
                    }

                    let dist = self
                        .inner
                        .resolve_dist(e, query, None, &ctx)
                        .map_err(|err| AcornError::Internal(err.to_string()))?;

                    let cand = Candidate {
                        index: e,
                        distance: dist,
                    };
                    candidates.push(Reverse(cand));
                    beam.push(cand);

                    if !deleted.contains(e as u64) {
                        let doc_id = self
                            .inner
                            .resolve_doc_id(e, &ctx)
                            .map_err(|err| AcornError::Internal(err.to_string()))?;

                        // Zero-Cross-Contamination: test predicate before considering match
                        if predicate(doc_id) {
                            valid_matches.push(CandidateWithDoc { doc_id, distance: dist });
                            if valid_matches.len() > max_valid_capacity {
                                valid_matches.pop();
                            }
                        }
                    }
                }
            }
        }

        while let Some(Reverse(current)) = candidates.pop() {
            if beam.len() >= ef_acorn {
                if let Some(worst) = beam.peek() {
                    if current.distance > worst.distance {
                        break;
                    }
                }
            }

            let connections = self
                .inner
                .resolve_connections(current.index, 0, &ctx)
                .map_err(|err| AcornError::Internal(err.to_string()))?;

            for &neighbor_u32 in connections.iter() {
                let neighbor = neighbor_u32 as usize;
                if neighbor >= total_nodes {
                    continue;
                }

                let is_visited = visited.get(neighbor).copied().unwrap_or(true);
                if !is_visited {
                    if let Some(visited_entry) = visited.get_mut(neighbor) {
                        *visited_entry = true;
                    }

                    let dist = self
                        .inner
                        .resolve_dist(neighbor, query, None, &ctx)
                        .map_err(|err| AcornError::Internal(err.to_string()))?;

                    let cand = Candidate {
                        index: neighbor,
                        distance: dist,
                    };

                    let is_better = match beam.peek() {
                        Some(worst) => dist < worst.distance,
                        None => true,
                    };

                    if is_better || beam.len() < ef_acorn {
                        candidates.push(Reverse(cand));
                        beam.push(cand);
                        if beam.len() > ef_acorn {
                            beam.pop();
                        }
                    }

                    if !deleted.contains(neighbor as u64) {
                        let doc_id = self
                            .inner
                            .resolve_doc_id(neighbor, &ctx)
                            .map_err(|err| AcornError::Internal(err.to_string()))?;

                        // Zero-Cross-Contamination: strictly filter by predicate
                        if predicate(doc_id) {
                            valid_matches.push(CandidateWithDoc { doc_id, distance: dist });
                            if valid_matches.len() > max_valid_capacity {
                                valid_matches.pop();
                            }
                        }
                    }
                }
            }
        }

        let mut sorted_valid: Vec<CandidateWithDoc> = valid_matches.into_vec();
        sorted_valid.sort_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        // Deduplicate doc_ids if any duplicate node entries exist
        let mut results = Vec::with_capacity(k);
        for item in sorted_valid {
            if !results.iter().any(|(id, _): &(DocId, f32)| *id == item.doc_id) {
                results.push((item.doc_id, item.distance));
                if results.len() >= k {
                    break;
                }
            }
        }

        Ok(results)
    }
}
