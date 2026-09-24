use super::builder::HybridQueryBuilder;
use contextra_types::{DocId, Result};
use contextra_ports::{StorageEngine, VectorIndex};

impl<'a, S: StorageEngine, V: VectorIndex> HybridQueryBuilder<'a, S, V> {
    /// Executes search query, delegating core signal retrieval to `Collection::hybrid_search_with_query()`.
    pub async fn execute(self) -> Result<Vec<crate::SearchResult>> {
        let k = self.k.unwrap_or(10);
        if k == 0 {
            return Ok(Vec::new());
        }

        #[cfg(feature = "reranking")]
        let _has_reranker = self.reranker.is_some();
        #[cfg(not(feature = "reranking"))]
        let _has_reranker = false;

        let fusion_weights = self.weights.unwrap_or_default();
        let fusion_strategy = self
            .fusion_strategy
            .or_else(|| self.strategy.as_ref().map(|s| s.to_fusion_strategy()))
            .unwrap_or_default();

        let hybrid_query = contextra_types::HybridQuery {
            text_query: self.text.clone(),
            vector_query: self.vector.clone(),
            graph_start_node: self
                .anchor_entities
                .as_ref()
                .and_then(|a| a.first())
                .map(|e| e.to_string()),
            graph_strategy: self
                .strategy
                .as_ref()
                .map(|s| s.to_graph_strategy())
                .unwrap_or_default(),
            fusion_weights,
            fusion_strategy,
            filter: self.filter.clone(),
            memory_type_filter: self.memory_type_filter.clone(),
            same_community_as: self.same_community_as,
            include_superseded: self.include_superseded,
            include_provenance: self.include_provenance,
            rerank_pool_multiplier: self.rerank_pool_multiplier,
            rerank_pool_max: self.rerank_pool_max,
            has_reranker: _has_reranker,
            k,
        };

        let seq_no = if let Some(s) = self.seq {
            s
        } else {
            self.collection.snapshot_seq().await?
        };

        #[allow(deprecated)]
        let mut results = self
            .collection
            .hybrid_search_with_query_at(&hybrid_query, seq_no)
            .await?;

        if let Some(ref filter_expr) = self.filter {
            results.retain(|res| {
                let meta_ref = res.metadata.as_ref().unwrap_or(&serde_json::Value::Null);
                filter_expr.evaluate(meta_ref)
            });
        }

        if let Some(ref memory_types) = self.memory_type_filter {
            results.retain(|res| {
                let mt = crate::filter::extract_memory_type(&res.metadata);
                memory_types.contains(&mt)
            });
        }

        if let Some(ref filter_fn) = self.filter_fn {
            let mut filtered = Vec::with_capacity(results.len());
            for res in results {
                if let Ok(doc_id) = DocId::from_key(&res.id) {
                    if filter_fn(doc_id) {
                        filtered.push(res);
                    }
                }
            }
            results = filtered;
        }

        // Post-RRF Bi-temporal Validity Filtering (ADR-033 / ADR-038)
        // Executed NACH RRF-Fusion and VOR CrossEncoder Reranking to ensure high efficiency
        // (expensive CrossEncoder reranker is only invoked on temporally valid candidates).
        if let Some(as_of) = self.as_of_timestamp {
            results = crate::temporal_filter::apply_temporal_validity_filter_at(results, as_of);
        } else if self.query_timestamp.is_some() || self.current_tx.is_some() {
            let ctx = self.current_tx.unwrap_or(contextra_types::TxId::new(u64::MAX));
            results = crate::temporal_filter::apply_temporal_validity_filter(
                results,
                ctx,
                self.query_timestamp,
            );
        }

        #[cfg(feature = "reranking")]
        if let Some(reranker) = self.reranker {
            // RESOLVED: AGT-DB-8ddf8937 (TS: 2026-09-10T00:00:00Z SESSION: 8ddf8937)
            // Explicitly bind query text for Cross-Encoder reranking
            let text_str = self.text.as_deref().unwrap_or("");
            if !results.is_empty() && !text_str.is_empty() {
                let _current_pool = results.len();
                let start_time = std::time::Instant::now();

                let candidate_texts: Vec<String> = results
                    .iter()
                    .map(|r| {
                        r.metadata
                            .as_ref()
                            .and_then(|m| m.get("text").or_else(|| m.get("content")))
                            .and_then(|v| v.as_str())
                            .unwrap_or(&r.id)
                            .to_string()
                    })
                    .collect();

                let rerank_deadline = std::time::Duration::from_millis(
                    reranker.config().rerank_deadline_ms.unwrap_or(500),
                );

                let reranked = tokio::time::timeout(
                    rerank_deadline,
                    reranker.rerank(text_str, &candidate_texts),
                )
                .await
                .map_err(|_| {
                    tracing::warn!(
                        deadline_ms = rerank_deadline.as_millis(),
                        "Reranker deadline exceeded — falling back to RRF order"
                    );
                })
                .and_then(|r| {
                    r.map_err(|e| {
                        tracing::warn!("Reranking failed: {e}");
                    })
                });

                if let Ok(ranked) = reranked {
                    let _elapsed = start_time.elapsed();
                    #[cfg(feature = "adaptive-candidate-pool-sizing")]
                    if let Some(ref pid) = self.pid_controller {
                        pid.lock()
                            .update(_current_pool, _elapsed.as_millis() as f32);
                    }

                    // Implizites Calibration-Feedback (k=5 als Relevanz-Cutoff)
                    reranker.record_implicit_feedback(&ranked, 5.min(k));

                    let mut reranked_results = Vec::with_capacity(k);
                    for r in ranked.into_iter().take(k) {
                        if let Some(mut result) = results.get(r.original_index).cloned() {
                            if let Some(meta) = result.metadata.as_mut() {
                                if let Some(obj) = meta.as_object_mut() {
                                    obj.insert("ce_score".to_string(), serde_json::json!(r.score));
                                }
                            } else {
                                result.metadata = Some(serde_json::json!({ "ce_score": r.score }));
                            }
                            result.score = r.score;
                            if let Some(p) = result.provenance.as_mut() {
                                p.rerank_score = Some(r.score);
                            }
                            reranked_results.push(result);
                        }
                    }
                    tracing::debug!("Reranking applied: {} candidates", reranked_results.len());
                    return Ok(reranked_results);
                }
            }
        }

        results.truncate(k);
        Ok(results)
    }
}
