// FILE-CONTEXT
// ZWECK: Multi-Step Iterative Retrieval Engine für komplexe Agenten-Abfragen (o-series Pattern).
// INVARIANTEN: RRF-Fusion über alle Runden; Abbruch bei Erreichen des Qualitätsschwellenwerts.
// NICHT-OFFENSICHTLICH: Sub-Queries nutzen BM25-only (leerer Vektor), da sie textuelle Umformulierungen darstellen.

use crate::pid_latency_controller::{LatencyBudgetGuard, PidLatencyController};
use crate::Collection;
use memfuse_core::{Result, StorageEngine};
pub use memfuse_rank::multistep::{MultiStepConfig, MultiStepResult, QueryRewriter};
use memfuse_rank::SearchResult;
use std::sync::Arc;

fn convert_engine_results(
    results: Vec<memfuse_engine::SearchResult>,
) -> Vec<SearchResult> {
    results
        .into_iter()
        .map(|r| SearchResult {
            id: r.id,
            score: r.score,
            metadata: r.metadata,
            matched_signals: r.matched_signals,
            provenance: r.provenance.map(|p| memfuse_rank::ProvenanceRecord {
                vector_distance: p.vector_distance,
                bm25_score: p.bm25_score,
                graph_score: p.graph_score,
                rerank_score: p.rerank_score,
                signal_ranks: p.signal_ranks,
                source_collection: p.source_collection,
                index_type: p.index_type,
                signal_contributions: p
                    .signal_contributions
                    .into_iter()
                    .map(|(k, c)| {
                        (
                            k,
                            memfuse_rank::SignalContribution {
                                raw_score: c.raw_score,
                                rank: c.rank,
                                rrf_contribution: c.rrf_contribution,
                            },
                        )
                    })
                    .collect(),
                coherence_bonus: p.coherence_bonus,
            }),
        })
        .collect()
}

/// Multi-Step Retrieval Engine.
///
/// Implementiert iteratives Query-Rewriting für komplexe Agenten-Abfragen.
/// Erfordert ein `QueryRewriter`-Trait für LLM-basiertes Rewriting.
pub struct MultiStepEngine<S: StorageEngine> {
    collection: Arc<Collection<S>>,
    config: MultiStepConfig,
    pid_controller: parking_lot::Mutex<PidLatencyController>,
}

impl<S: StorageEngine> MultiStepEngine<S> {
    pub fn new(collection: Arc<Collection<S>>, config: MultiStepConfig) -> Self {
        let pid = PidLatencyController::new(config.latency_budget_ms);
        Self {
            collection,
            config,
            pid_controller: parking_lot::Mutex::new(pid),
        }
    }

    /// Führt iterative Hybrid-Suche durch.
    pub async fn search(
        &self,
        original_query: &str,
        vector: &[f32],
        k: usize,
        rewriter: Option<&dyn QueryRewriter>,
    ) -> Result<MultiStepResult> {
        use crate::fusion::reciprocal_rank_fusion;

        let k = k.min(memfuse_core::MAX_SEARCH_K);
        let current_k = k;
        let mut all_result_sets: Vec<Vec<SearchResult>> = Vec::new();
        let mut sub_queries: Vec<String> = Vec::new();
        let mut rounds_executed = 0;
        let budget_guard = LatencyBudgetGuard::new(self.config.latency_budget_ms);

        // Runde 1: Standard-Suche
        let round1 = self
            .collection
            .query()
            .text(original_query)
            .vector(vector)
            .k(current_k * 2)
            .execute()
            .await?;
        all_result_sets.push(convert_engine_results(round1));
        rounds_executed += 1;

        let round1_ref = all_result_sets.first().map(|v| v.as_slice()).unwrap_or(&[]);
        if self.quality_sufficient(round1_ref) || rewriter.is_none() {
            let fused = reciprocal_rank_fusion(all_result_sets, k);
            return Ok(MultiStepResult {
                results: fused,
                rounds_executed,
                sub_queries,
            });
        }

        let rewriter = match rewriter {
            Some(r) => r,
            None => {
                let fused = reciprocal_rank_fusion(all_result_sets, k);
                return Ok(MultiStepResult {
                    results: fused,
                    rounds_executed,
                    sub_queries,
                });
            }
        };

        for _round in 2..=self.config.max_rounds {
            if budget_guard.is_exceeded() {
                tracing::info!(
                    elapsed_ms = budget_guard.elapsed_ms(),
                    budget_ms = budget_guard.budget_ms(),
                    "MultiStep search budget exceeded; terminating expansion gracefully with partial results"
                );
                break;
            }

            let scaling_factor = self
                .pid_controller
                .lock()
                .compute_adjustment(budget_guard.elapsed_ms());
            let scaled_k = ((k as f64) * scaling_factor).round() as usize;
            let scaled_k = scaled_k.max(1);

            let current_results = all_result_sets.last().map(|v| v.as_slice()).unwrap_or(&[]);
            let sub_qs = match rewriter.rewrite(original_query, current_results).await {
                Ok(qs) => qs,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "QueryRewriter.rewrite() failed in round; stopping expansion gracefully"
                    );
                    break;
                }
            };

            if sub_qs.is_empty() {
                break;
            }

            let mut executed_sub_query = false;
            for sub_q in &sub_qs {
                match self
                    .collection
                    .query()
                    .text(sub_q)
                    .k(scaled_k)
                    .execute()
                    .await
                {
                    Ok(sub_results) => {
                        all_result_sets.push(convert_engine_results(sub_results));
                        sub_queries.push(sub_q.clone());
                        executed_sub_query = true;
                    }
                    Err(e) => {
                        tracing::warn!(
                            sub_query = %sub_q,
                            error = %e,
                            "Sub-query search failed in multi-step execution; skipping sub-query"
                        );
                    }
                }
            }

            if executed_sub_query {
                rounds_executed += 1;
            }

            let latest_results = all_result_sets.last().map(|v| v.as_slice()).unwrap_or(&[]);
            if self.quality_sufficient(latest_results) {
                break;
            }
        }

        let fused = reciprocal_rank_fusion(all_result_sets, k);
        Ok(MultiStepResult {
            results: fused,
            rounds_executed,
            sub_queries,
        })
    }

    fn quality_sufficient(&self, results: &[SearchResult]) -> bool {
        let high_quality = results
            .iter()
            .filter(|r| r.score >= self.config.quality_threshold)
            .count();
        high_quality >= self.config.min_quality_hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memfuse_core::BoxFuture;
    use memfuse_graph::CsrGraph;
    use memfuse_index::{HnswConfig, HnswIndex};
    use memfuse_store::{LsmConfig, LsmStorage};
    use std::sync::atomic::AtomicU64;
    use tempfile::tempdir;

    struct DummyRewriter {
        responses: std::sync::Mutex<Vec<Vec<String>>>,
    }

    impl QueryRewriter for DummyRewriter {
        fn rewrite<'a>(
            &'a self,
            _original_query: &'a str,
            _current_results: &'a [SearchResult],
        ) -> BoxFuture<'a, Result<Vec<String>>> {
            Box::pin(async move {
                let mut guard = self.responses.lock().unwrap(); // unwrap
                if !guard.is_empty() {
                    Ok(guard.remove(0))
                } else {
                    Ok(vec![])
                }
            })
        }
    }

    async fn create_test_collection() -> Arc<Collection<LsmStorage>> {
        let dir = tempdir().expect("tempdir"); // expect
        let lsm_config = LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(LsmStorage::new(lsm_config).await.expect("lsm storage")); // expect
        let hnsw_config = HnswConfig {
            dimension: 4,
            ..Default::default()
        };
        let index = Arc::new(HnswIndex::try_new(hnsw_config).expect("hnsw index")); // expect
        let graph = Arc::new(CsrGraph::new());
        let next_tx = Arc::new(AtomicU64::new(1));

        let col = Collection::new(
            "default".to_string(),
            storage,
            index,
            graph,
            next_tx,
            4,
            memfuse_text::Language::English,
        );

        Arc::new(col)
    }

    #[tokio::test]
    async fn test_multistep_single_round_sufficient() {
        let col = create_test_collection().await;
        col.insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(serde_json::json!({"text": "rust programming"})),
        )
        .await
        .expect("insert"); // expect
        col.insert(
            "doc2",
            &[0.9, 0.1, 0.0, 0.0],
            Some(serde_json::json!({"text": "rust language"})),
        )
        .await
        .expect("insert"); // expect

        let config = MultiStepConfig {
            max_rounds: 3,
            quality_threshold: 0.001,
            min_quality_hits: 1,
            latency_budget_ms: 1000.0,
        };
        let engine = MultiStepEngine::new(col, config);

        let rewriter = DummyRewriter {
            responses: std::sync::Mutex::new(vec![vec!["sub query 1".to_string()]]),
        };

        let result = engine
            .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
            .await
            .expect("search"); // expect

        assert_eq!(result.rounds_executed, 1);
        assert!(result.sub_queries.is_empty());
        assert!(!result.results.is_empty());
    }

    #[tokio::test]
    async fn test_multistep_query_rewriting_triggers() {
        let col = create_test_collection().await;
        col.insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(serde_json::json!({"text": "rust programming"})),
        )
        .await
        .expect("insert"); // expect

        let config = MultiStepConfig {
            max_rounds: 3,
            quality_threshold: 0.99, // high threshold, round 1 won't meet min_quality_hits=2
            min_quality_hits: 2,
            latency_budget_ms: 1000.0,
        };
        let engine = MultiStepEngine::new(col, config);

        let rewriter = DummyRewriter {
            responses: std::sync::Mutex::new(vec![
                vec!["rust programming".to_string()], // round 2
                vec![],                               // round 3 (stop)
            ]),
        };

        let result = engine
            .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
            .await
            .expect("search"); // expect

        assert_eq!(result.rounds_executed, 2);
        assert_eq!(result.sub_queries, vec!["rust programming"]);
        assert!(!result.results.is_empty());
    }
}
