// FILE-CONTEXT
// ZWECK: Hybrid-Familie (hybrid_search, hybrid_search_reranked, hybrid_search_with_weights, hybrid_search_with_strategy, traverse_links, filter_by_importance) für Collection.

mod query;

use super::checkpoint::with_pinned_checkpoint_at_latest;
use super::{extract_effective_importance, Collection};
use contextra_ports::{GraphIndex, StorageEngine, TextIndex, VectorIndex};
use contextra_types::{DocId, EntityId, Result, TxId};

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Performs hybrid search combining BM25, vector search, and graph traversal results via RRF.
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, text, vector))]
    pub async fn hybrid_search(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        anchor_entities: Option<&[contextra_types::EntityId]>,
    ) -> Result<Vec<crate::SearchResult>> {
        self.hybrid_search_with_weights(text, vector, k, anchor_entities, None)
            .await
    }

    /// Performs hybrid search combining BM25, vector search, and graph traversal, followed by optional Cross-Encoder reranking.
    /// Mindest-Kandidatenpool für Cross-Encoder-Reranking.
    /// Wissenschaftliche Basis: arXiv:2604.01733 (T2-RAGBench).
    /// k_pool=20 → Recall@5=0.458; k_pool=100 → Recall@5=0.888 (Qualitätsknie).
    pub const DEFAULT_MIN_RERANK_CANDIDATES: usize = 100;

    #[cfg(feature = "reranking")]
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, text, vector, reranker, anchor_entities))]
    pub async fn hybrid_search_reranked(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        reranker: Option<&contextra_infer_onnx::CrossEncoderReranker>,
        anchor_entities: Option<&[contextra_types::EntityId]>,
    ) -> Result<Vec<crate::SearchResult>> {
        let mut builder = self.query().text(text).vector(vector).k(k);
        if let Some(r) = reranker {
            builder = builder.reranker(r);
        }
        if let Some(anchors) = anchor_entities {
            builder = builder.anchors(anchors.iter().copied());
        }
        builder.execute().await
    }

    /// Performs hybrid search with custom fusion weights for vector, text, and graph signals,
    /// and optional community filtering/boosting.
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, text, vector))]
    pub async fn hybrid_search_with_weights(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        anchor_entities: Option<&[contextra_types::EntityId]>,
        weights: Option<&contextra_types::FusionWeights>,
    ) -> Result<Vec<crate::SearchResult>> {
        self.hybrid_search_with_strategy(text, vector, k, anchor_entities, weights, None, None)
            .await
    }

    /// Performs hybrid search with custom signal fusion weights and graph traversal strategy.
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, text, vector, strategy))]
    #[allow(clippy::too_many_arguments)]
    pub async fn hybrid_search_with_strategy(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        anchor_entities: Option<&[contextra_types::EntityId]>,
        weights: Option<&contextra_types::FusionWeights>,
        strategy: Option<&contextra_types::GraphTraversalStrategy>,
        same_community_as: Option<EntityId>,
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

        // Pin-first, read-after: the seq is read under the protection of the pin,
        // eliminating the TOCTOU window between snapshot_seq() and pin activation.
        with_pinned_checkpoint_at_latest(self.storage.as_ref(), |seq| async move {
            let is_vector_zero = vector.iter().all(|&v| v == 0.0);
            let is_text_empty = text.trim().is_empty();

            let default_strategy = contextra_types::GraphTraversalStrategy::default();
            let graph_strat = strategy.unwrap_or(&default_strategy);

            // 1. Vector Signal (Candidate overfetching for RRF fusion)
            let vector_results = if is_vector_zero {
                Vec::new()
            } else {
                self.search_filtered_at(
                    vector,
                    k.saturating_mul(Self::OVERFETCH_FACTOR),
                    None,
                    seq,
                )
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

            // 3. Graph Signal
            // AI-TAG[RESOLVED][MINOR] graph_search snapshot isolation via multi_traverse_at for Hops; PARTIAL with warning for PPR/PathRag. (ID: AGT-DB-6d724b1a)
            let implicit_anchors: Vec<contextra_types::EntityId>;
            let anchors_ref: Option<&[contextra_types::EntityId]> = if let Some(anchors) = anchor_entities
            {
                if anchors.is_empty() {
                    None
                } else {
                    Some(anchors)
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
                let tuples = match graph_strat {
                    contextra_types::GraphTraversalStrategy::Hops { max_hops } => {
                        let mut raw_tuples = self
                            .graph_index
                            .multi_traverse_at(anchors, *max_hops, seq)
                            .await?;
                        raw_tuples.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                        raw_tuples.truncate(k);
                        raw_tuples
                    }
                    contextra_types::GraphTraversalStrategy::PersonalizedPageRank(cfg) => {
                        let mut raw_tuples = self
                            .graph_index
                            .personalized_page_rank_at(anchors, cfg, seq)
                            .await?;
                        raw_tuples.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                        raw_tuples.truncate(k);
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
                self.hydrate_from_tuples_at(doc_tuples, seq).await?
            } else {
                Vec::new()
            };

            if !text_results.is_empty() && graph_results.is_empty() && anchor_entities.is_none() {
                tracing::warn!(
                    text_count = text_results.len(),
                    "Implicit graph anchors derived from text results produced empty graph signal. Verify mapping invariant: graph nodes must share string keys with text documents (EntityId::from_key)."
                );
            }

            if vector_results.is_empty() && text_results.is_empty() && graph_results.is_empty() {
                return Ok(Vec::new());
            }

            let (vw, tw, gw) = crate::fusion::weights_to_signal_factors(weights);

            let target_community_id: Option<u64> = if let Some(same_comm_entity) = same_community_as {
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

            let mut fused = crate::fusion::fuse_search_results_with_strategy(
                signal_sets,
                k.saturating_mul(Self::OVERFETCH_FACTOR),
                crate::fusion::MetadataMergePriority::default(),
                true,
                None,
                contextra_types::FusionStrategy::Rrf,
            );
            fused.truncate(k);

            let boosted = self
                .apply_community_boost_post_rrf(
                    fused,
                    target_community_id,
                    Self::DEFAULT_COMMUNITY_BOOST,
                )
                .await?;
            Ok(boosted)
        })
        .await
    }

    /// Standard overfetch factor applied to candidate limits before RRF fusion to balance OOM protection and recall.
    pub const OVERFETCH_FACTOR: usize = 3;

    /// Standard community boost factor applied to RRF scores for matching community members.
    pub const DEFAULT_COMMUNITY_BOOST: f32 = 1.2;

    /// Multiplies the post-RRF scores of results belonging to `target_community_id` by `boost_factor`,
    /// then re-sorts descending by score with deterministic secondary sorting by document ID.
    pub(super) async fn apply_community_boost_post_rrf(
        &self,
        mut results: Vec<crate::SearchResult>,
        target_community_id: Option<u64>,
        boost_factor: f32,
    ) -> Result<Vec<crate::SearchResult>> {
        let Some(target_comm) = target_community_id else {
            return Ok(results);
        };

        if results.is_empty() {
            return Ok(results);
        }

        let parsed_eids: Vec<Option<contextra_types::EntityId>> = results
            .iter()
            .map(|res| contextra_types::EntityId::from_key(&res.id).ok())
            .collect();

        let candidate_eids: Vec<contextra_types::EntityId> =
            parsed_eids.iter().filter_map(|&eid| eid).collect();

        if candidate_eids.is_empty() {
            return Ok(results);
        }

        let community_map = self.get_communities_batch(&candidate_eids).await?;

        for (res, &opt_eid) in results.iter_mut().zip(parsed_eids.iter()) {
            if let Some(eid) = opt_eid {
                if let Some(&comm) = community_map.get(&eid) {
                    if comm == target_comm {
                        res.score *= boost_factor;
                    }
                }
            }
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });

        Ok(results)
    }

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
