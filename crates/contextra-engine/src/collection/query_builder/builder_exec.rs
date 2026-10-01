use super::builder::HybridQueryBuilder;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{DocId, Result};

impl<'a, S: StorageEngine, V: VectorIndex> HybridQueryBuilder<'a, S, V> {
    /// Executes search query, delegating core signal retrieval to `Collection::hybrid_search_with_query()`.
    pub async fn execute(self) -> Result<Vec<crate::SearchResult>> {
        let k = self.k.unwrap_or(10);
        if k == 0 {
            return Ok(Vec::new());
        }

        let _has_reranker = self.reranker.is_some();

        let fusion_weights = self.weights.unwrap_or_default();
        let fusion_strategy = self
            .fusion_strategy
            .or_else(|| {
                self.strategy
                    .as_ref()
                    .map(|s| s.to_signal_fusion_strategies())
            })
            .unwrap_or_default();

        // Calculate candidate pool limits according to requirements:
        // Base candidate pool size is user wish multiplier * k (default multiplier = 10).
        // Enforce pool bounds [50, 200].
        let raw_mult = self
            .rerank_pool_multiplier
            .unwrap_or(super::builder::DEFAULT_RERANK_POOL_MULTIPLIER);
        let base_pool = (raw_mult * k).clamp(50, 200);

        #[cfg(feature = "adaptive-candidate-pool-sizing")]
        let effective_pool_max = if let Some(ref pid) = self.pid_controller {
            let pid_pool = pid.lock().current_pool_size().unwrap_or(base_pool);
            base_pool.min(pid_pool).clamp(50, 200)
        } else {
            self.rerank_pool_max
                .map_or(base_pool, |max_val| base_pool.min(max_val).clamp(50, 200))
        };

        #[cfg(not(feature = "adaptive-candidate-pool-sizing"))]
        let effective_pool_max = self
            .rerank_pool_max
            .map_or(base_pool, |max_val| base_pool.min(max_val).clamp(50, 200));

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
            rerank_pool_max: Some(effective_pool_max),
            has_reranker: _has_reranker,
            on_signal_failure: self.on_signal_failure,
            k,
        };

        let seq_no = if let Some(s) = self.seq {
            s
        } else {
            self.collection.snapshot_seq().await?
        };

        if self.hard_scope.is_some() {
            return Err(contextra_types::ContextraError::capability_unsupported(
                "acorn_hard_boundary",
                "ACORN hard-boundary scoping requires execute_with_scope() on a vector index implementing FilteredIndex",
            ));
        }

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
            let ctx = self
                .current_tx
                .unwrap_or(contextra_types::TxId::new(u64::MAX));
            results = crate::temporal_filter::apply_temporal_validity_filter(
                results,
                ctx,
                self.query_timestamp,
            );
        }

        if let Some(reranker) = self.reranker {
            // RESOLVED: AGT-DB-8ddf8937 (TS: 2026-09-10T00:00:00Z SESSION: 8ddf8937)
            // Explicitly bind query text for Cross-Encoder reranking
            let text_str = self.text.as_deref().unwrap_or("");
            if !results.is_empty() && !text_str.is_empty() {
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

                let deadline_ms = reranker.rerank_deadline_ms().unwrap_or(500);
                let rerank_deadline = std::time::Duration::from_millis(deadline_ms);

                let chunk_size = if candidate_texts.len() <= 10 { 1 } else { 10 };
                let mut accumulated_ranked: Vec<contextra_types::RerankResult> = Vec::new();

                for (chunk_idx, chunk) in candidate_texts.chunks(chunk_size).enumerate() {
                    let elapsed = start_time.elapsed();
                    if elapsed >= rerank_deadline {
                        tracing::warn!(
                            deadline_ms = rerank_deadline.as_millis(),
                            "Reranker deadline reached before evaluating all candidates — executing Anytime-Rerank fallback"
                        );
                        break;
                    }
                    let remaining = rerank_deadline - elapsed;
                    let start_candidate_idx = chunk_idx * chunk_size;

                    let rerank_res =
                        tokio::time::timeout(remaining, reranker.rerank(text_str, chunk)).await;

                    match rerank_res {
                        Ok(Ok(ranked_chunk)) => {
                            for r in ranked_chunk {
                                accumulated_ranked.push(contextra_types::RerankResult {
                                    index: r.index,
                                    original_index: start_candidate_idx + r.original_index,
                                    score: r.score,
                                    calibrated_score: r.calibrated_score,
                                });
                            }
                        }
                        Ok(Err(e)) => {
                            tracing::warn!("Reranking chunk failed: {e}");
                            break;
                        }
                        Err(_) => {
                            tracing::warn!(
                                deadline_ms = rerank_deadline.as_millis(),
                                "Reranker deadline exceeded during chunk evaluation — executing Anytime-Rerank fallback"
                            );
                            break;
                        }
                    }
                }

                let _elapsed = start_time.elapsed();
                #[cfg(feature = "adaptive-candidate-pool-sizing")]
                if let Some(ref pid) = self.pid_controller {
                    pid.lock().update(_elapsed, _elapsed.as_millis() as f32);
                }

                if !accumulated_ranked.is_empty() {
                    // Sort evaluated candidates by score descending
                    accumulated_ranked.sort_by(|a, b| {
                        b.score
                            .partial_cmp(&a.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    // Implizites Calibration-Feedback (k=5 als Relevanz-Cutoff)
                    reranker.record_implicit_feedback(&accumulated_ranked, 5.min(k));

                    let mut evaluated_indices = std::collections::HashSet::new();
                    let mut final_results = Vec::with_capacity(k);

                    for r in accumulated_ranked {
                        evaluated_indices.insert(r.original_index);
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
                            final_results.push(result);
                        }
                    }

                    // Append unevaluated candidates in their original fusion order
                    for (idx, res) in results.into_iter().enumerate() {
                        if !evaluated_indices.contains(&idx) {
                            final_results.push(res);
                        }
                    }

                    final_results.truncate(k);
                    tracing::debug!(
                        "Anytime reranking applied: {} candidates",
                        final_results.len()
                    );
                    return Ok(final_results);
                } else {
                    // Complete timeout before any candidate completed: fallback to original fusion results
                    results.truncate(k);
                    return Ok(results);
                }
            }
        }

        results.truncate(k);
        Ok(results)
    }
}

impl<'a, S: StorageEngine, V: VectorIndex> HybridQueryBuilder<'a, S, V>
where
    V: contextra_vector::acorn::FilteredIndex<Predicate = dyn Fn(DocId) -> bool>,
{
    /// Executes search query with ACORN hard-boundary scoping (`.scope(ScopeConstraint)`).
    ///
    /// Evaluates vector similarity strictly within `allowed_doc_ids` during ACORN graph traversal
    /// with `gamma` edge budget augmentation (Spec §5.3, §8.5, §10.4).
    pub async fn execute_with_scope(self) -> Result<Vec<crate::SearchResult>> {
        let k = self.k.unwrap_or(10);
        if k == 0 {
            return Ok(Vec::new());
        }

        let scope = self.hard_scope.as_ref().ok_or_else(|| {
            contextra_types::ContextraError::invalid_input(
                "execute_with_scope called without a configured ScopeConstraint via .scope()",
            )
        })?;

        let embedding = self.vector.as_ref().ok_or_else(|| {
            contextra_types::ContextraError::invalid_input(
                "Dense vector query embedding is required for ACORN hard-boundary scoped search",
            )
        })?;

        let allowed_set = scope.allowed_doc_ids.clone();
        let predicate: Box<dyn Fn(DocId) -> bool + Send + Sync> =
            Box::new(move |doc_id| allowed_set.contains(&doc_id));

        let acorn_tuples = self
            .collection
            .index
            .search_knn_acorn(embedding, k, predicate.as_ref(), scope.gamma)
            .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;

        let seq_no = if let Some(s) = self.seq {
            s
        } else {
            self.collection.snapshot_seq().await?
        };

        let mut results = Vec::with_capacity(acorn_tuples.len());
        for (doc_id, score) in acorn_tuples {
            let doc_key = self
                .collection
                .namespaced_key(&doc_id.inner().to_le_bytes(), 1);
            // SSI: nicht tx-gebunden — Hydrierung von ACORN-Suchergebnissen am Snapshot
            if let Some(bytes) = self
                .collection
                .storage()
                .get_at_seq(&doc_key, seq_no)
                .await?
            {
                let (id, metadata) = if let Ok(meta) =
                    serde_json::from_slice::<crate::collection::StoredDocumentMeta>(&bytes)
                {
                    (meta.id, meta.metadata)
                } else if let Ok(full) =
                    serde_json::from_slice::<crate::collection::StoredDocument>(&bytes)
                {
                    (full.id, full.metadata)
                } else {
                    continue;
                };
                let prov = crate::ProvenanceRecord {
                    source_collection: Some(self.collection.name().to_string()),
                    ..Default::default()
                };
                results.push(crate::SearchResult {
                    id,
                    score,
                    metadata,
                    matched_signals: vec!["acorn_vector".to_string()],
                    provenance: Some(prov),
                });
            }
        }

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

        if let Some(as_of) = self.as_of_timestamp {
            results = crate::temporal_filter::apply_temporal_validity_filter_at(results, as_of);
        } else if self.query_timestamp.is_some() || self.current_tx.is_some() {
            let ctx = self
                .current_tx
                .unwrap_or(contextra_types::TxId::new(u64::MAX));
            results = crate::temporal_filter::apply_temporal_validity_filter(
                results,
                ctx,
                self.query_timestamp,
            );
        }

        if let Some(reranker) = self.reranker {
            let text_str = self.text.as_deref().unwrap_or("");
            if !results.is_empty() && !text_str.is_empty() {
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

                let deadline_ms = reranker.rerank_deadline_ms().unwrap_or(500);
                let rerank_deadline = std::time::Duration::from_millis(deadline_ms);

                if let Ok(Ok(ranked)) = tokio::time::timeout(
                    rerank_deadline,
                    reranker.rerank(text_str, &candidate_texts),
                )
                .await
                {
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
                    return Ok(reranked_results);
                }
            }
        }

        results.truncate(k);
        Ok(results)
    }
}
