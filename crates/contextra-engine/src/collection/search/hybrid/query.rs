// FILE-CONTEXT
// ZWECK: Hybrid-Familie Submodule für Query-basierte Suche.

use crate::collection::search::checkpoint::{
    with_pinned_checkpoint, with_pinned_checkpoint_at_latest,
};
use crate::collection::search::filtered::{MAX_OVERFETCH_SCAN, OVERFETCH_STAGE_MULTIPLIERS};
use crate::collection::Collection;
use crate::fusion::{SearchReport, Signal, SignalFailurePolicy};
use contextra_ports::{GraphIndex, MetricsSink, StorageEngine, TextIndex, VectorIndex};
use contextra_types::{DocId, Result};

/// Fixed pipeline stages for hybrid search processing (§W1-15).
pub const HYBRID_PIPELINE_STAGES: &[&str] =
    &["signal_retrieval", "fusion", "rerank", "egress_filter"];

/// Typed pipeline stage identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HybridPipelineStage {
    SignalRetrieval,
    Fusion,
    Rerank,
    EgressFilter,
}

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Performs hybrid search combining BM25, vector, and graph signals configured via `HybridQuery`.
    /// Scheduled for removal in v0.2.0. Use `Collection::query().query_config()` instead.
    #[deprecated(
        since = "0.1.0",
        note = "Use Collection::query() instead. Scheduled for removal in v0.2.0."
    )]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, query))]
    pub async fn hybrid_search_with_query(
        &self,
        query: &contextra_types::HybridQuery,
    ) -> Result<Vec<crate::SearchResult>> {
        with_pinned_checkpoint_at_latest(self.storage.as_ref(), |seq| async move {
            let snapshot = contextra_ports::ReadSnapshot::from_seq(seq);
            self.hybrid_search_with_query_at(query, &snapshot).await
        })
        .await
    }

    /// Performs hybrid search combining BM25, vector, and graph signals configured via `HybridQuery` at a specific snapshot sequence.
    #[allow(deprecated)]
    pub async fn hybrid_search_with_query_at(
        &self,
        query: &contextra_types::HybridQuery,
        snapshot: &contextra_ports::ReadSnapshot,
    ) -> Result<Vec<crate::SearchResult>> {
        let (results, _) = self
            .hybrid_search_with_query_and_report_at(
                query,
                snapshot,
                SignalFailurePolicy::Fail,
                None,
            )
            .await?;
        Ok(results)
    }

    /// Performs hybrid search returning both search results and an execution `SearchReport`.
    #[allow(deprecated)]
    pub async fn hybrid_search_with_query_and_report_at(
        &self,
        query: &contextra_types::HybridQuery,
        snapshot: &contextra_ports::ReadSnapshot,
        on_signal_failure: SignalFailurePolicy,
        metrics: Option<&dyn MetricsSink>,
    ) -> Result<(Vec<crate::SearchResult>, SearchReport)> {
        let seq = snapshot.max_seq();
        if query.k == 0 {
            return Ok((Vec::new(), SearchReport::default()));
        }
        let k = query.k.min(contextra_types::MAX_SEARCH_K);

        let text = query.text_query.as_deref().unwrap_or("");
        let vector = query.vector_query.as_deref().unwrap_or(&[]);

        // S-1 FIX: Fail-fast is_finite() check at the API boundary (Fail-Fast before HNSW).
        if !vector.is_empty() {
            for (i, &val) in vector.iter().enumerate() {
                if !val.is_finite() {
                    return Err(contextra_types::ContextraError::invalid_input(format!(
                        "hybrid_search: query vector element at index {i} is not finite (value: {val}). \
                         Check embedding model output for NaN/Inf before querying."
                    )));
                }
            }
        }

        with_pinned_checkpoint(self.storage.as_ref(), seq, || async move {
            let mut report = SearchReport::default();
            let candidate_k = Self::compute_candidate_k(query, k);
            let total_docs = self.len().await;

            let vector_results = self
                .execute_vector_signal(
                    query,
                    vector,
                    candidate_k,
                    total_docs,
                    seq,
                    on_signal_failure,
                    metrics,
                    &mut report,
                )
                .await?;

            let text_results = self
                .execute_text_signal(
                    query,
                    text,
                    candidate_k,
                    total_docs,
                    seq,
                    on_signal_failure,
                    metrics,
                    &mut report,
                )
                .await?;

            let graph_results = self
                .execute_graph_signal(
                    query,
                    &text_results,
                    candidate_k,
                    seq,
                    on_signal_failure,
                    metrics,
                    &mut report,
                )
                .await?;

            if vector_results.is_empty() && text_results.is_empty() && graph_results.is_empty() {
                return Ok((Vec::new(), report));
            }

            let fused_results = self
                .fuse_signals_and_post_process(
                    query,
                    k,
                    candidate_k,
                    vector_results,
                    text_results,
                    graph_results,
                )
                .await?;

            Ok((fused_results, report))
        })
        .await
    }

    /// Computes the candidate pool size (`candidate_k`) for hybrid search overfetching and reranking.
    ///
    /// Pre-condition: `k` is positive and bounded by `MAX_SEARCH_K`.
    /// Post-condition: Returns candidate size clamped between `k` and `MAX_SEARCH_K`.
    fn compute_candidate_k(query: &contextra_types::HybridQuery, k: usize) -> usize {
        let (mult, max_pool) = if query.has_reranker {
            (
                query
                    .rerank_pool_multiplier
                    .unwrap_or(crate::collection::query_builder::DEFAULT_RERANK_POOL_MULTIPLIER),
                query
                    .rerank_pool_max
                    .unwrap_or(crate::collection::query_builder::DEFAULT_RERANK_POOL_MAX),
            )
        } else {
            (1, k)
        };

        let mut candidate_k = k;
        if !query.include_superseded {
            candidate_k = candidate_k.max(k.saturating_mul(3));
        }
        let rerank_k = k.saturating_mul(mult).min(max_pool);
        candidate_k
            .max(rerank_k)
            .saturating_mul(Self::OVERFETCH_FACTOR)
            .min(contextra_types::MAX_SEARCH_K)
            .max(k)
    }

    /// Filters search results against `query.filter` metadata expression and `query.memory_type_filter`.
    ///
    /// Pre-condition: `list` contains search result candidates prior to RRF fusion.
    /// Post-condition: Returns only search results that satisfy all configured pre-RRF filter predicates.
    fn filter_pre_rrf(
        query: &contextra_types::HybridQuery,
        list: Vec<crate::SearchResult>,
    ) -> Vec<crate::SearchResult> {
        let mut filtered = Vec::with_capacity(list.len());
        for res in list {
            if let Some(ref filter_expr) = query.filter {
                let meta_ref = res.metadata.as_ref().unwrap_or(&serde_json::Value::Null);
                if !filter_expr.evaluate(meta_ref) {
                    continue;
                }
            }

            if let Some(ref type_filter) = query.memory_type_filter {
                let memory_type = crate::filter::extract_memory_type(&res.metadata);
                if !type_filter.contains(&memory_type) {
                    continue;
                }
            }

            filtered.push(res);
        }
        filtered
    }

    /// Executes vector similarity search channel with adaptive oversampling if filtered.
    ///
    /// Pre-condition: `vector` is validated (all elements finite), snapshot `seq` is pinned.
    /// Post-condition: Returns vector channel results or handles degraded signal according to `on_signal_failure`.
    #[allow(clippy::too_many_arguments)]
    async fn execute_vector_signal(
        &self,
        query: &contextra_types::HybridQuery,
        vector: &[f32],
        candidate_k: usize,
        total_docs: usize,
        seq: u64,
        on_signal_failure: SignalFailurePolicy,
        metrics: Option<&dyn MetricsSink>,
        report: &mut SearchReport,
    ) -> Result<Vec<crate::SearchResult>> {
        let is_vector_zero = vector.is_empty() || vector.iter().all(|&v| v == 0.0);
        let is_filtered = query.filter.is_some() || query.memory_type_filter.is_some();

        let vec_start = std::time::Instant::now();
        let vector_res: Result<Vec<crate::SearchResult>> = if is_vector_zero {
            Ok(Vec::new())
        } else if is_filtered {
            let matched_ids_opt = self.get_matching_doc_ids_for_query_at(query, seq).await?;
            if let Some(ref matched_ids) = matched_ids_opt {
                if matched_ids.is_empty() {
                    Ok(Vec::new())
                } else {
                    let matched_ids_cloned = matched_ids.clone();
                    let filter_fn = move |id: DocId| matched_ids_cloned.contains(&id);
                    let max_cap = total_docs.min(MAX_OVERFETCH_SCAN).max(candidate_k);
                    let mut final_filtered = Vec::new();

                    for (stage_idx, &stage_mult) in OVERFETCH_STAGE_MULTIPLIERS.iter().enumerate() {
                        report.overfetch_stages =
                            report.overfetch_stages.max((stage_idx + 1) as u8);
                        let oversample = (candidate_k * stage_mult).min(max_cap);
                        let raw_vec_results = self
                            .search_filtered_at(vector, oversample, Some(&filter_fn), seq)
                            .await?;
                        let raw_len = raw_vec_results.len();
                        final_filtered = Self::filter_pre_rrf(query, raw_vec_results);

                        if final_filtered.len() >= candidate_k
                            || oversample >= max_cap
                            || raw_len < oversample
                        {
                            tracing::debug!(
                                search_stage = stage_idx + 1,
                                signal = "vector",
                                matched_count = final_filtered.len(),
                                "Adaptive oversampling vector signal completed"
                            );
                            break;
                        }
                    }
                    Ok(final_filtered)
                }
            } else {
                let max_cap = total_docs.min(MAX_OVERFETCH_SCAN).max(candidate_k);
                let mut final_filtered = Vec::new();

                for (stage_idx, &stage_mult) in OVERFETCH_STAGE_MULTIPLIERS.iter().enumerate() {
                    report.overfetch_stages = report.overfetch_stages.max((stage_idx + 1) as u8);
                    let oversample = (candidate_k * stage_mult).min(max_cap);
                    let raw_vec_results = self
                        .search_filtered_at(vector, oversample, None, seq)
                        .await?;
                    let raw_len = raw_vec_results.len();
                    final_filtered = Self::filter_pre_rrf(query, raw_vec_results);

                    if final_filtered.len() >= candidate_k
                        || oversample >= max_cap
                        || raw_len < oversample
                    {
                        tracing::debug!(
                            search_stage = stage_idx + 1,
                            signal = "vector",
                            matched_count = final_filtered.len(),
                            "Adaptive oversampling vector signal completed"
                        );
                        break;
                    }
                }
                Ok(final_filtered)
            }
        } else {
            report.overfetch_stages = report.overfetch_stages.max(1);
            self.search_filtered_at(vector, candidate_k, None, seq)
                .await
        };

        let elapsed_vec = vec_start.elapsed().as_secs_f64();
        if let Some(sink) = metrics {
            sink.record_histogram(
                crate::collection::query_builder::METRIC_SEARCH_LATENCY_VECTOR,
                elapsed_vec,
                &[],
            );
        }

        match vector_res {
            Ok(res) => Ok(res),
            Err(err) => {
                if on_signal_failure == SignalFailurePolicy::Fail {
                    Err(err)
                } else {
                    report.degraded_signals.push(Signal::Vector);
                    tracing::warn!("Vector search signal failed under Degrade policy: {err}");
                    Ok(Vec::new())
                }
            }
        }
    }

    /// Executes text BM25 search channel with selectivity estimation and hydration.
    ///
    /// Pre-condition: Snapshot `seq` is pinned, `text` is dereferenced.
    /// Post-condition: Returns hydrated and filtered search results or handles degraded signal per `on_signal_failure`.
    #[allow(clippy::too_many_arguments)]
    async fn execute_text_signal(
        &self,
        query: &contextra_types::HybridQuery,
        text: &str,
        candidate_k: usize,
        total_docs: usize,
        seq: u64,
        on_signal_failure: SignalFailurePolicy,
        metrics: Option<&dyn MetricsSink>,
        report: &mut SearchReport,
    ) -> Result<Vec<crate::SearchResult>> {
        let is_text_empty = text.trim().is_empty();
        let is_filtered = query.filter.is_some() || query.memory_type_filter.is_some();

        let text_start = std::time::Instant::now();
        let text_res: Result<Vec<crate::SearchResult>> = if is_text_empty {
            Ok(Vec::new())
        } else if is_filtered {
            let selectivity = if let Some(ref filter_expr) = query.filter {
                self.estimate_filter_selectivity(filter_expr, seq, total_docs)
                    .await?
            } else {
                0.1
            };
            let max_cap = total_docs.min(MAX_OVERFETCH_SCAN).max(candidate_k);
            let calculated_initial =
                ((candidate_k as f64) / selectivity.max(0.0001)).ceil() as usize;

            let mut final_filtered = Vec::new();

            for (stage_idx, &stage_mult) in OVERFETCH_STAGE_MULTIPLIERS.iter().enumerate() {
                report.overfetch_stages = report.overfetch_stages.max((stage_idx + 1) as u8);
                let oversample = (calculated_initial * stage_mult / 3).clamp(candidate_k, max_cap);
                let bm25_results = self.text_index.search_at(text, oversample, seq).await?;
                let bm25_len = bm25_results.len();
                let hydrated = self
                    .hydrate_from_tuples_at(
                        bm25_results
                            .into_iter()
                            .map(|sd| (sd.doc_id, sd.score))
                            .collect(),
                        seq,
                    )
                    .await?;
                final_filtered = Self::filter_pre_rrf(query, hydrated);

                if final_filtered.len() >= candidate_k
                    || oversample >= max_cap
                    || bm25_len < oversample
                {
                    tracing::debug!(
                        search_stage = stage_idx + 1,
                        signal = "text",
                        matched_count = final_filtered.len(),
                        "Adaptive oversampling text signal completed"
                    );
                    break;
                }
            }
            Ok(final_filtered)
        } else {
            report.overfetch_stages = report.overfetch_stages.max(1);
            let bm25_results = self.text_index.search_at(text, candidate_k, seq).await?;
            self.hydrate_from_tuples_at(
                bm25_results
                    .into_iter()
                    .map(|sd| (sd.doc_id, sd.score))
                    .collect(),
                seq,
            )
            .await
        };

        let elapsed_text = text_start.elapsed().as_secs_f64();
        if let Some(sink) = metrics {
            sink.record_histogram(
                crate::collection::query_builder::METRIC_SEARCH_LATENCY_TEXT,
                elapsed_text,
                &[],
            );
        }

        match text_res {
            Ok(res) => Ok(res),
            Err(err) => {
                if on_signal_failure == SignalFailurePolicy::Fail {
                    Err(err)
                } else {
                    report.degraded_signals.push(Signal::Text);
                    tracing::warn!("Text search signal failed under Degrade policy: {err}");
                    Ok(Vec::new())
                }
            }
        }
    }

    /// Executes graph traversal search channel using explicit or text-derived implicit anchors.
    ///
    /// Pre-condition: `text_results` from text signal phase are available for implicit anchor extraction.
    /// Post-condition: Returns hydrated graph search results and appends warning messages if anchors produce empty results.
    async fn execute_graph_signal(
        &self,
        query: &contextra_types::HybridQuery,
        text_results: &[crate::SearchResult],
        candidate_k: usize,
        seq: u64,
        on_signal_failure: SignalFailurePolicy,
        metrics: Option<&dyn MetricsSink>,
        report: &mut SearchReport,
    ) -> Result<Vec<crate::SearchResult>> {
        let implicit_anchors: Vec<contextra_types::EntityId>;
        let anchors_ref: Option<&[contextra_types::EntityId]> =
            if let Some(ref start_node) = query.graph_start_node {
                let parsed_eid = if let Ok(u) = start_node.parse::<u64>() {
                    Some(contextra_types::EntityId::new(u))
                } else if let Some(inner_str) = start_node
                    .strip_prefix("EntityId(")
                    .and_then(|s| s.strip_suffix(')'))
                {
                    inner_str
                        .parse::<u64>()
                        .ok()
                        .map(contextra_types::EntityId::new)
                } else {
                    contextra_types::EntityId::from_key(start_node).ok()
                };
                if let Some(eid) = parsed_eid {
                    implicit_anchors = vec![eid];
                    Some(&implicit_anchors)
                } else {
                    None
                }
            } else if !text_results.is_empty() {
                implicit_anchors = text_results
                    .iter()
                    .filter_map(|r| contextra_types::EntityId::from_key(r.id.as_str()).ok())
                    .collect();
                Some(&implicit_anchors)
            } else {
                None
            };

        let graph_start = std::time::Instant::now();
        let graph_res: Result<Vec<crate::SearchResult>> = if let Some(anchors) = anchors_ref {
            let tuples_res = match query.graph_strategy {
                contextra_types::GraphTraversalStrategy::Hops { max_hops } => {
                    self.graph_index
                        .multi_traverse_at(anchors, max_hops, seq)
                        .await
                }
                contextra_types::GraphTraversalStrategy::PersonalizedPageRank(ref cfg) => {
                    self.graph_index
                        .personalized_page_rank_at(anchors, cfg, seq)
                        .await
                }
                contextra_types::GraphTraversalStrategy::PathRag { max_hops, .. } => {
                    self.graph_index.path_rag_at(anchors, max_hops, seq).await
                }
            };

            match tuples_res {
                Ok(mut raw_tuples) => {
                    raw_tuples.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                    raw_tuples.truncate(candidate_k);
                    let doc_tuples = raw_tuples
                        .into_iter()
                        .map(|(eid, score)| (contextra_types::DocId::new(eid.inner()), score))
                        .collect();
                    let hydrated = self.hydrate_from_tuples_at(doc_tuples, seq).await?;
                    Ok(Self::filter_pre_rrf(query, hydrated))
                }
                Err(e) => Err(e),
            }
        } else {
            Ok(Vec::new())
        };

        let elapsed_graph = graph_start.elapsed().as_secs_f64();
        if let Some(sink) = metrics {
            sink.record_histogram(
                crate::collection::query_builder::METRIC_SEARCH_LATENCY_GRAPH,
                elapsed_graph,
                &[],
            );
        }

        let graph_results = match graph_res {
            Ok(res) => res,
            Err(err) => {
                if on_signal_failure == SignalFailurePolicy::Fail {
                    return Err(err);
                }
                report.degraded_signals.push(Signal::Graph);
                tracing::warn!("Graph search signal failed under Degrade policy: {err}");
                Vec::new()
            }
        };

        if !text_results.is_empty() && graph_results.is_empty() && query.graph_start_node.is_none()
        {
            let msg =
                "Implicit graph anchors derived from text results produced empty graph signal";
            tracing::warn!(text_count = text_results.len(), "{msg}");
            report.warnings.push(msg.to_string());
        } else if query.graph_start_node.is_some() && graph_results.is_empty() {
            let msg = "Explicit graph anchor produced empty graph signal";
            tracing::warn!("{msg}");
            report.warnings.push(msg.to_string());
        }

        Ok(graph_results)
    }

    /// Fuses signal result sets using configured fusion strategy and applies post-fusion filtering & boosting.
    ///
    /// Pre-condition: At least one signal channel returned non-empty results.
    /// Post-condition: Returns fused, superseded-filtered, truncated-to-k, and community-boosted `SearchResult` vector.
    async fn fuse_signals_and_post_process(
        &self,
        query: &contextra_types::HybridQuery,
        k: usize,
        candidate_k: usize,
        vector_results: Vec<crate::SearchResult>,
        text_results: Vec<crate::SearchResult>,
        graph_results: Vec<crate::SearchResult>,
    ) -> Result<Vec<crate::SearchResult>> {
        let (vw, tw, gw) = crate::fusion::weights_to_signal_factors(Some(&query.fusion_weights));

        let target_community_id: Option<u64> =
            if let Some(same_comm_entity) = query.same_community_as {
                self.get_community(same_comm_entity).await?
            } else {
                None
            };

        let mut signal_sets = Vec::new();
        if !vector_results.is_empty() {
            signal_sets.push(("vector".to_string(), vector_results, vw));
        }
        if !text_results.is_empty() {
            signal_sets.push(("text".to_string(), text_results, tw));
        }
        if !graph_results.is_empty() {
            signal_sets.push(("graph".to_string(), graph_results, gw));
        }

        let max_fusion_results = candidate_k
            .saturating_mul(Self::OVERFETCH_FACTOR)
            .min(contextra_types::MAX_SEARCH_K);

        let mut fused_results = crate::fusion::fuse_search_results_with_signal_strategies(
            signal_sets,
            max_fusion_results,
            crate::fusion::MetadataMergePriority::default(),
            query.include_provenance,
            None,
            query.fusion_strategy,
        );

        let supersedes_pool_size = k.saturating_mul(3).max(k);
        fused_results.truncate(supersedes_pool_size);

        if !query.include_superseded {
            let mut superseded_targets = std::collections::HashSet::new();
            for res in &fused_results {
                if let Ok(doc_id) = DocId::from_key(&res.id) {
                    let links = self.get_links(doc_id).await?;
                    for link in links {
                        if link.relation == contextra_types::domain::LinkRelation::Supersedes {
                            superseded_targets.insert(link.target);
                        }
                    }
                }
            }
            if !superseded_targets.is_empty() {
                fused_results.retain(|res| {
                    if let Ok(doc_id) = DocId::from_key(&res.id) {
                        !superseded_targets.contains(&doc_id)
                    } else {
                        true
                    }
                });
            }
        }

        fused_results.truncate(k);

        let mut boosted = self
            .apply_community_boost_post_fusion(
                fused_results,
                target_community_id,
                Self::DEFAULT_COMMUNITY_BOOST,
            )
            .await?;

        // Deterministic secondary tie-breaking on document ID (§W1-15)
        boosted.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });

        Ok(boosted)
    }

    /// Additive extension for hybrid search supporting optional Cross-Encoder reranking and egress filtering.
    pub async fn hybrid_search_with_query_and_report_at_reranked(
        &self,
        query: &contextra_types::HybridQuery,
        snapshot: &contextra_ports::ReadSnapshot,
        on_signal_failure: SignalFailurePolicy,
        metrics: Option<&dyn MetricsSink>,
        reranker: Option<&dyn contextra_ports::Reranker>,
    ) -> Result<(Vec<crate::SearchResult>, SearchReport)> {
        let (mut results, report) = self
            .hybrid_search_with_query_and_report_at(query, snapshot, on_signal_failure, metrics)
            .await?;

        if let Some(r) = reranker {
            let text_str = query.text_query.as_deref().unwrap_or("");
            if !results.is_empty() && !text_str.is_empty() {
                let candidate_texts: Vec<String> = results
                    .iter()
                    .map(|res| {
                        res.metadata
                            .as_ref()
                            .and_then(|m| m.get("text").or_else(|| m.get("content")))
                            .and_then(|v| v.as_str())
                            .unwrap_or(&res.id)
                            .to_string()
                    })
                    .collect();

                if let Ok(ranked) = r.rerank(text_str, &candidate_texts).await {
                    let mut reranked = Vec::with_capacity(query.k);
                    for item in ranked.into_iter().take(query.k) {
                        if let Some(mut orig) = results.get(item.original_index).cloned() {
                            orig.score = item.score;
                            reranked.push(orig);
                        }
                    }
                    reranked.sort_by(|a, b| {
                        b.score
                            .partial_cmp(&a.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .then_with(|| a.id.cmp(&b.id))
                    });
                    results = reranked;
                }
            }
        }

        Ok((results, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::TxId;

    #[test]
    fn test_filter_by_importance_filtering() {
        let doc_high = crate::SearchResult {
            id: "doc-high".to_string(),
            score: 0.9,
            metadata: Some(serde_json::json!({
                "importance": {
                    "base_score": 0.8,
                    "decay": "None",
                    "created_at_tx": 1
                }
            })),
            matched_signals: Vec::new(),
            provenance: None,
        };

        let doc_low = crate::SearchResult {
            id: "doc-low".to_string(),
            score: 0.85,
            metadata: Some(serde_json::json!({
                "importance": {
                    "base_score": 0.2,
                    "decay": "None",
                    "created_at_tx": 1
                }
            })),
            matched_signals: Vec::new(),
            provenance: None,
        };

        let results = vec![doc_high.clone(), doc_low.clone()];
        let now_tx = TxId::try_new(10).unwrap();

        // 1. Default / No threshold filter (threshold 0.0): returns both
        let no_filter = Collection::<contextra_store::LsmStorage, contextra_vector::HnswIndex>::filter_by_importance(results.clone(), 0.0, now_tx);
        assert_eq!(no_filter.len(), 2);
        assert_eq!(no_filter[0].id, "doc-high");
        assert_eq!(no_filter[1].id, "doc-low");

        // 2. High threshold (0.5): filters out doc-low, preserves doc-high without reordering
        let filtered = Collection::<contextra_store::LsmStorage, contextra_vector::HnswIndex>::filter_by_importance(results, 0.5, now_tx);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "doc-high");
    }

    #[test]
    fn test_traverse_links_structure() {
        // Verify DocId representations for Zettelkasten link traversal
        let start = DocId::new(100);
        let target1 = DocId::new(101);
        let target2 = DocId::new(102);

        let hops = vec![(target1, 1), (target2, 2)];
        assert_eq!(hops.len(), 2);
        assert_eq!(hops[0].0, target1);
        assert_eq!(hops[0].1, 1);
        assert_eq!(hops[1].0, target2);
        assert_eq!(hops[1].1, 2);
        assert_ne!(start, target1);
    }
}
