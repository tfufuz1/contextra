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

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Performs hybrid search combining BM25, vector, and graph signals configured via `HybridQuery`.
    ///
    /// Applies `memory_type_filter` and metadata `FilterExpr` as Pre-RRF filters to preserve
    /// Reciprocal Rank Fusion properties (ADR-024).
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, query))]
    pub async fn hybrid_search_with_query(
        &self,
        query: &contextra_types::HybridQuery,
    ) -> Result<Vec<crate::SearchResult>> {
        // Pin-first, read-after: the seq is read under the protection of the pin,
        // eliminating the TOCTOU window between snapshot_seq() and pin activation.
        with_pinned_checkpoint_at_latest(self.storage.as_ref(), |seq| async move {
            self.hybrid_search_with_query_at(query, seq).await
        })
        .await
    }

    /// Performs hybrid search combining BM25, vector, and graph signals configured via `HybridQuery` at a specific snapshot sequence.
    #[allow(deprecated)]
    pub async fn hybrid_search_with_query_at(
        &self,
        query: &contextra_types::HybridQuery,
        seq: u64,
    ) -> Result<Vec<crate::SearchResult>> {
        let (results, _) = self
            .hybrid_search_with_query_and_report_at(query, seq, SignalFailurePolicy::Fail, None)
            .await?;
        Ok(results)
    }

    /// Performs hybrid search returning both search results and an execution `SearchReport`.
    #[allow(deprecated)]
    pub async fn hybrid_search_with_query_and_report_at(
        &self,
        query: &contextra_types::HybridQuery,
        seq: u64,
        on_signal_failure: SignalFailurePolicy,
        metrics: Option<&dyn MetricsSink>,
    ) -> Result<(Vec<crate::SearchResult>, SearchReport)> {
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
            let is_vector_zero = vector.is_empty() || vector.iter().all(|&v| v == 0.0);
            let is_text_empty = text.trim().is_empty();

            // Candidate pool calculation considering pre-reranking multiplier/max bounds and supersedes displacement requirements
            let (mult, max_pool) = if query.has_reranker {
                (
                    query.rerank_pool_multiplier.unwrap_or(
                        crate::collection::query_builder::DEFAULT_RERANK_POOL_MULTIPLIER,
                    ),
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
            candidate_k = candidate_k
                .max(rerank_k)
                .saturating_mul(Self::OVERFETCH_FACTOR)
                .min(contextra_types::MAX_SEARCH_K)
                .max(k);

            let total_docs = self.len().await;

            let filter_pre_rrf = |list: Vec<crate::SearchResult>| {
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
            };

            let is_filtered = query.filter.is_some() || query.memory_type_filter.is_some();

            // 1. Vector Signal
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

                        for (stage_idx, &stage_mult) in
                            OVERFETCH_STAGE_MULTIPLIERS.iter().enumerate()
                        {
                            report.overfetch_stages =
                                report.overfetch_stages.max((stage_idx + 1) as u8);
                            let oversample = (candidate_k * stage_mult).min(max_cap);
                            let raw_vec_results = self
                                .search_filtered_at(vector, oversample, Some(&filter_fn), seq)
                                .await?;
                            let raw_len = raw_vec_results.len();
                            final_filtered = filter_pre_rrf(raw_vec_results);

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
                        report.overfetch_stages =
                            report.overfetch_stages.max((stage_idx + 1) as u8);
                        let oversample = (candidate_k * stage_mult).min(max_cap);
                        let raw_vec_results = self
                            .search_filtered_at(vector, oversample, None, seq)
                            .await?;
                        let raw_len = raw_vec_results.len();
                        final_filtered = filter_pre_rrf(raw_vec_results);

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

            let vector_results = match vector_res {
                Ok(res) => res,
                Err(err) => {
                    if on_signal_failure == SignalFailurePolicy::Fail {
                        return Err(err);
                    }
                    report.degraded_signals.push(Signal::Vector);
                    tracing::warn!("Vector search signal failed under Degrade policy: {err}");
                    Vec::new()
                }
            };

            // 2. Text Signal
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
                    let oversample =
                        (calculated_initial * stage_mult / 3).clamp(candidate_k, max_cap);
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
                    final_filtered = filter_pre_rrf(hydrated);

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

            let text_results = match text_res {
                Ok(res) => res,
                Err(err) => {
                    if on_signal_failure == SignalFailurePolicy::Fail {
                        return Err(err);
                    }
                    report.degraded_signals.push(Signal::Text);
                    tracing::warn!("Text search signal failed under Degrade policy: {err}");
                    Vec::new()
                }
            };

            // 3. Graph Signal
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
                        Ok(filter_pre_rrf(hydrated))
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

            if !text_results.is_empty()
                && graph_results.is_empty()
                && query.graph_start_node.is_none()
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

            if vector_results.is_empty() && text_results.is_empty() && graph_results.is_empty() {
                return Ok((Vec::new(), report));
            }

            let (vw, tw, gw) =
                crate::fusion::weights_to_signal_factors(Some(&query.fusion_weights));

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

            let fused_results = self
                .apply_community_boost_post_fusion(
                    fused_results,
                    target_community_id,
                    Self::DEFAULT_COMMUNITY_BOOST,
                )
                .await?;

            Ok((fused_results, report))
        })
        .await
    }
}
