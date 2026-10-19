// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Post-Fusion Cross-Encoder Reranking Stage (§20.2 / APM-22).

use std::sync::Arc;
use contextra_ports::Reranker;
use contextra_types::{Result};

use super::topk::cmp_scores;
use super::types::{ProvenanceRecord, SearchResult};

/// Post-retrieval reranking stage executing a `Reranker` model on top-k candidates.
pub struct RerankStage {
    reranker: Option<Arc<dyn Reranker>>,
    top_k: usize,
    max_seq: Option<u64>,
}

impl std::fmt::Debug for RerankStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RerankStage")
            .field("has_reranker", &self.reranker.is_some())
            .field("top_k", &self.top_k)
            .field("max_seq", &self.max_seq)
            .finish()
    }
}

impl RerankStage {
    /// Creates a new `RerankStage` with an optional `Reranker` instance and top-k candidate bound.
    pub fn new(reranker: Option<Arc<dyn Reranker>>, top_k: usize) -> Self {
        Self {
            reranker,
            top_k,
            max_seq: None,
        }
    }

    /// Sets the snapshot isolation sequence number (`max_seq`) for consistent multi-signal read views.
    pub fn with_max_seq(mut self, max_seq: u64) -> Self {
        self.max_seq = Some(max_seq);
        self
    }

    /// Returns the snapshot isolation sequence number (`max_seq`), if set.
    pub fn max_seq(&self) -> Option<u64> {
        self.max_seq
    }

    /// Sets or clears the `max_seq` snapshot isolation bound.
    pub fn set_max_seq(&mut self, max_seq: Option<u64>) {
        self.max_seq = max_seq;
    }

    /// Returns the top-k candidate limit for this rerank stage.
    pub fn top_k(&self) -> usize {
        self.top_k
    }

    /// Helper function providing deterministic sorting: score descending, then document ID ascending.
    pub fn sort_results_stably(results: &mut [SearchResult]) {
        results.sort_by(|a, b| cmp_scores(b.score, a.score).then_with(|| a.id.cmp(&b.id)));
    }

    /// Executes the reranking stage on fused search results.
    ///
    /// - Top-k-only: only reranks up to `top_k` candidates; remaining candidates are not passed to the reranker.
    /// - Stable tie-breaking: results with equal scores are ordered deterministically by document ID ascending.
    /// - Safe fallback: if `reranker` is `None` or `top_k == 0`, returns top-k sorted candidates directly.
    pub async fn execute(
        &self,
        query: &str,
        mut results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>> {
        if results.is_empty() || self.top_k == 0 {
            return Ok(vec![]);
        }

        // Pre-sort candidates deterministically (score desc, doc ID asc)
        Self::sort_results_stably(&mut results);

        let reranker = match self.reranker.as_ref() {
            Some(r) => r,
            None => {
                results.truncate(self.top_k);
                return Ok(results);
            }
        };

        // Truncate to top_k to enforce top-k-only execution (never full traversal)
        let mut top_candidates = if results.len() > self.top_k {
            results.drain(..self.top_k).collect::<Vec<_>>()
        } else {
            results
        };

        // Extract text payloads for candidates
        let candidate_texts: Vec<String> = top_candidates
            .iter()
            .map(|r| {
                r.metadata
                    .as_ref()
                    .and_then(|m| m.get("text").or_else(|| m.get("content")))
                    .and_then(|t| t.as_str())
                    .unwrap_or(r.id.as_str())
                    .to_string()
            })
            .collect();

        // Perform async reranking
        let rerank_results = reranker.rerank(query, &candidate_texts).await?;

        // Update scores and provenance for reranked candidates
        for r_res in rerank_results {
            if let Some(candidate) = top_candidates.get_mut(r_res.original_index) {
                candidate.score = r_res.calibrated_score;
                let provenance = candidate
                    .provenance
                    .get_or_insert_with(ProvenanceRecord::default);
                provenance.rerank_score = Some(r_res.score);
            }
        }

        // Re-sort reranked candidates with deterministic tie-breaking
        Self::sort_results_stably(&mut top_candidates);

        Ok(top_candidates)
    }
}
