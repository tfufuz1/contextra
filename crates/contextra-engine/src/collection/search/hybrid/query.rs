// FILE-CONTEXT
// ZWECK: Hybrid-Familie Submodule für Query-basierte Suche.

use crate::collection::search::checkpoint::{with_pinned_checkpoint, with_pinned_checkpoint_at_latest};
use crate::collection::Collection;
use contextra_types::{DocId, Result};
use contextra_ports::{GraphIndex, StorageEngine, TextIndex, VectorIndex};

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
        if query.k == 0 {
            return Ok(Vec::new());
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
            let is_vector_zero = vector.is_empty() || vector.iter().all(|&v| v == 0.0);
            let is_text_empty = text.trim().is_empty();

            // Candidate pool calculation considering pre-reranking multiplier/max bounds and supersedes displacement requirements
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
            let vector_results = if is_vector_zero {
                Vec::new()
            } else if is_filtered {
                let matched_ids_opt = self.get_matching_doc_ids_for_query_at(query, seq).await?;
                if let Some(ref matched_ids) = matched_ids_opt {
                    if matched_ids.is_empty() {
                        Vec::new()
                    } else {
                        let matched_ids_cloned = matched_ids.clone();
                        let filter_fn = move |id: DocId| matched_ids_cloned.contains(&id);
                        let max_cap = total_docs.min(contextra_types::MAX_SEARCH_K).max(candidate_k);
                        let mut oversample = candidate_k;
                        let mut iterations = 0;
                        loop {
                            iterations += 1;
                            let raw_vec_results = self
                                .search_filtered_at(vector, oversample, Some(&filter_fn), seq)
                                .await?;
                            let raw_len = raw_vec_results.len();
                            let filtered = filter_pre_rrf(raw_vec_results);

                            if filtered.len() >= candidate_k
                                || oversample >= max_cap
                                || raw_len < oversample
                            {
                                tracing::debug!(
                                    search_iterations_needed = iterations,
                                    signal = "vector",
                                    matched_count = filtered.len(),
                                    "Adaptive oversampling vector signal completed"
                                );
                                break filtered;
                            }
                            oversample = (oversample * 2).min(max_cap);
                        }
                    }
                } else {
                    let max_cap = total_docs.min(contextra_types::MAX_SEARCH_K).max(candidate_k);
                    let mut oversample = candidate_k;
                    let mut iterations = 0;
                    loop {
                        iterations += 1;
                        let raw_vec_results = self
                            .search_filtered_at(vector, oversample, None, seq)
                            .await?;
                        let raw_len = raw_vec_results.len();
                        let filtered = filter_pre_rrf(raw_vec_results);

                        if filtered.len() >= candidate_k
                            || oversample >= max_cap
                            || raw_len < oversample
                        {
                            tracing::debug!(
                                search_iterations_needed = iterations,
                                signal = "vector",
                                matched_count = filtered.len(),
                                "Adaptive oversampling vector signal completed"
                            );
                            break filtered;
                        }
                        oversample = (oversample * 2).min(max_cap);
                    }
                }
            } else {
                self.search_filtered_at(vector, candidate_k, None, seq)
                    .await?
            };

            // 2. Text Signal
            let text_results = if is_text_empty {
                Vec::new()
            } else if is_filtered {
                let selectivity = if let Some(ref filter_expr) = query.filter {
                    self.estimate_filter_selectivity(filter_expr, seq, total_docs)
                        .await?
                } else {
                    0.1
                };
                let max_cap = total_docs.min(contextra_types::MAX_SEARCH_K).max(candidate_k);
                let calculated_initial =
                    ((candidate_k as f64) / selectivity.max(0.0001)).ceil() as usize;
                let min_oversample = candidate_k.min(max_cap);
                let mut oversample = calculated_initial.clamp(min_oversample, max_cap);

                let mut iterations = 0;
                loop {
                    iterations += 1;
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
                    let filtered = filter_pre_rrf(hydrated);

                    if filtered.len() >= candidate_k || oversample >= max_cap || bm25_len < oversample {
                        tracing::debug!(
                            search_iterations_needed = iterations,
                            signal = "text",
                            matched_count = filtered.len(),
                            "Adaptive oversampling text signal completed"
                        );
                        break filtered;
                    }
                    oversample = (oversample * 2).min(max_cap);
                }
            } else {
                let bm25_results = self.text_index.search_at(text, candidate_k, seq).await?;
                self.hydrate_from_tuples_at(
                    bm25_results
                        .into_iter()
                        .map(|sd| (sd.doc_id, sd.score))
                        .collect(),
                    seq,
                )
                .await?
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
                    // Graph-Knoten MÜSSEN mit demselben String-Schlüssel wie das korrespondierende Textdokument erstellt werden (via `EntityId::from_key`), sonst wird das Graph-Signal für Multi-Step-Query-Expansion und Zettelkasten-Displacement unbemerkt leer.
                    implicit_anchors = text_results
                        .iter()
                        .filter_map(|r| contextra_types::EntityId::from_key(r.id.as_str()).ok())
                        .collect();
                    Some(&implicit_anchors)
                } else {
                    None
                };

            let graph_results = if let Some(anchors) = anchors_ref {
                let tuples = match query.graph_strategy {
                    contextra_types::GraphTraversalStrategy::Hops { max_hops } => {
                        let mut raw_tuples = self
                            .graph_index
                            .multi_traverse_at(anchors, max_hops, seq)
                            .await?;
                        raw_tuples.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                        raw_tuples.truncate(candidate_k);
                        raw_tuples
                    }
                    contextra_types::GraphTraversalStrategy::PersonalizedPageRank(ref cfg) => {
                        let mut raw_tuples = self
                            .graph_index
                            .personalized_page_rank_at(anchors, cfg, seq)
                            .await?;
                        raw_tuples.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                        raw_tuples.truncate(candidate_k);
                        raw_tuples
                    }
                    contextra_types::GraphTraversalStrategy::PathRag { .. } => {
                        return Err(contextra_types::ContextraError::snapshot_unsupported_for_signal(
                            "PathRag strategy does not support snapshot-isolated retrieval",
                        ));
                    }
                };
                let doc_tuples = tuples
                    .into_iter()
                    .map(|(eid, score)| (contextra_types::DocId::new(eid.inner()), score))
                    .collect();
                let hydrated = self.hydrate_from_tuples_at(doc_tuples, seq).await?;
                filter_pre_rrf(hydrated)
            } else {
                Vec::new()
            };

            if !text_results.is_empty() && graph_results.is_empty() && query.graph_start_node.is_none()
            {
                tracing::warn!(
                    text_count = text_results.len(),
                    "Implicit graph anchors derived from text results produced empty graph signal. Verify mapping invariant: graph nodes must share string keys with text documents (EntityId::from_key)."
                );
            }

            if vector_results.is_empty() && text_results.is_empty() && graph_results.is_empty() {
                return Ok(Vec::new());
            }

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

            let mut fused_results = crate::fusion::fuse_search_results_with_strategy(
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
                .apply_community_boost_post_rrf(
                    fused_results,
                    target_community_id,
                    Self::DEFAULT_COMMUNITY_BOOST,
                )
                .await?;

            #[cfg(feature = "edge-reinforcement-learning")]
            if fused_results.len() >= 2 {
                let graph_index = self.graph_index.clone();
                let result_eids: Vec<contextra_types::EntityId> = fused_results
                    .iter()
                    .filter_map(|r| contextra_types::EntityId::from_key(&r.id).ok())
                    .collect();
                tokio::spawn(async move {
                    let _config = contextra_graph::edge_reinforcement::EdgeReinforcementConfig::default();
                    for i in 0..result_eids.len() {
                        for j in (i + 1)..result_eids.len() {
                            let e1 = result_eids[i];
                            let _e2 = result_eids[j];
                            let _ = graph_index.neighbors(e1).await;
                        }
                    }
                });
            }

            Ok(fused_results)
        })
        .await
    }
}
