//! # Modul: mod.rs
//!
//! **Zweck:** Hybride Multi-Signal-Suche mit paralleler Signalerfassung.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

// FILE-CONTEXT
// ZWECK: Hybrid-Familie (hybrid_search, hybrid_search_reranked, hybrid_search_with_weights, traverse_links, filter_by_importance) für Collection.

mod query;

use super::{Collection, extract_effective_importance};
use contextra_ports::{Reranker, StorageEngine, TextIndex, VectorIndex};
use contextra_types::{DocId, Result, TxId};

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Performs hybrid search combining BM25 and vector search results via RRF.
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, text, vector))]
    pub async fn hybrid_search(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
    ) -> Result<Vec<crate::SearchResult>> {
        self.hybrid_search_with_weights(text, vector, k, None).await
    }

    /// Performs hybrid search combining BM25 and vector search, followed by optional Cross-Encoder reranking.
    /// Mindest-Kandidatenpool für Cross-Encoder-Reranking.
    /// Wissenschaftliche Basis: arXiv:2604.01733 (T2-RAGBench).
    /// k_pool=20 → Recall@5=0.458; k_pool=100 → Recall@5=0.888 (Qualitätsknie).
    pub const DEFAULT_MIN_RERANK_CANDIDATES: usize = 100;

    // ANCHOR[contextra-engine:COL-HYB-043]
    /// Dokumentation für `hybrid_search_reranked` im Rahmen der Collection- und Suchabstraktion.
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, text, vector, reranker))]
    pub async fn hybrid_search_reranked(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        reranker: Option<&dyn Reranker>,
    ) -> Result<Vec<crate::SearchResult>> {
        let mut builder = self.query().text(text).vector(vector).k(k);
        if let Some(r) = reranker {
            builder = builder.reranker(r);
        }
        builder.execute().await
    }

    /// Performs hybrid search with custom fusion weights for vector and text signals.
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, text, vector))]
    pub async fn hybrid_search_with_weights(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        weights: Option<&contextra_types::FusionWeights>,
    ) -> Result<Vec<crate::SearchResult>> {
        self.hybrid_search_with_strategy(text, vector, k, weights, None)
            .await
    }

    /// Performs hybrid search with custom signal fusion weights and signal fusion strategy.
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, text, vector))]
    pub async fn hybrid_search_with_strategy(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        weights: Option<&contextra_types::FusionWeights>,
        fusion_strategy: Option<contextra_types::SignalFusionStrategies>,
    ) -> Result<Vec<crate::SearchResult>> {
        if k == 0 {
            return Ok(Vec::new());
        }
        if !vector.is_empty() && vector.len() != self.dimension {
            return Err(contextra_types::ContextraError::invalid_input(format!(
                "Dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.len()
            )));
        }
        let k = k.min(contextra_types::MAX_SEARCH_K);

        let seq = self.storage.last_seq_no().await?;

        let is_vector_zero = vector.iter().all(|&v| v == 0.0);
        let is_text_empty = text.trim().is_empty();

        // 1. Vector Signal (Candidate overfetching for RRF fusion)
        let vector_results = if is_vector_zero {
            Vec::new()
        } else {
            self.search_filtered_at(vector, k.saturating_mul(Self::OVERFETCH_FACTOR), None, seq)
                .await?
        };

        // 2. Text Signal (Candidate overfetching for RRF fusion)
        let text_results = if is_text_empty {
            Vec::new()
        } else {
            let bm25_results = self
                .text_index
                .search_at(text, k.saturating_mul(Self::OVERFETCH_FACTOR), seq)
                .await?;
            self.hydrate_from_tuples_at(
                bm25_results
                    .into_iter()
                    .map(|sd| (sd.doc_id, sd.score))
                    .collect(),
                seq,
            )
            .await?
        };

        if vector_results.is_empty() && text_results.is_empty() {
            return Ok(Vec::new());
        }

        let (vw, tw, _) = crate::fusion::weights_to_signal_factors(weights);

        let mut signal_sets = Vec::new();
        if !vector_results.is_empty() {
            signal_sets.push(("vector".to_string(), vector_results, vw));
        }
        if !text_results.is_empty() {
            signal_sets.push(("text".to_string(), text_results, tw));
        }

        let selected_fusion_strategy = fusion_strategy.unwrap_or_default();

        let mut fused = crate::fusion::fuse_search_results_with_signal_strategies(
            signal_sets,
            k.saturating_mul(Self::OVERFETCH_FACTOR),
            crate::fusion::MetadataMergePriority::default(),
            true,
            None,
            selected_fusion_strategy,
        );
        fused.truncate(k);

        Ok(super::hydrate::apply_read_gate(fused))
    }

    /// Standard overfetch factor applied to candidate limits before fusion to balance OOM protection and recall.
    pub const OVERFETCH_FACTOR: usize = 3;

    /// Traverses Zettelkasten memory links starting from a given document up to a maximum hop depth (ADR-038).
    ///
    /// Performs iterative BFS with cycle detection using `VecDeque`, returning `(DocId, depth)` tuples.
    /// Result count is capped at `MAX_SEARCH_K`.
    pub async fn traverse_links(
        &self,
        start: DocId,
        max_depth: usize,
    ) -> Result<Vec<(DocId, usize)>> {
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut results = Vec::new();

        visited.insert(start);
        queue.push_back((start, 0));

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            let links = self.get_links(current_id).await?;
            for link in links {
                if visited.insert(link.target) {
                    let next_depth = depth + 1;
                    results.push((link.target, next_depth));
                    if results.len() >= contextra_types::MAX_SEARCH_K {
                        return Ok(results);
                    }
                    if next_depth < max_depth {
                        queue.push_back((link.target, next_depth));
                    }
                }
            }
        }

        Ok(results)
    }

    /// Filters a candidate list of search results by effective importance score threshold.
    ///
    /// Candidate results with `effective_score(now_tx) < min_threshold` are removed from the result list.
    /// Does NOT reorder remaining items, keeping RRF & Reranking order intact (ADR-024).
    pub fn filter_by_importance(
        results: Vec<crate::SearchResult>,
        min_threshold: f32,
        now_tx: TxId,
    ) -> Vec<crate::SearchResult> {
        results
            .into_iter()
            .filter(|r| {
                let eff = extract_effective_importance(&r.metadata, now_tx);
                eff >= min_threshold
            })
            .collect()
    }
}
