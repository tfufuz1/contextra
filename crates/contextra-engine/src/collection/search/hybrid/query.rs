//! # Modul: query.rs
//!
//! **Zweck:** Strukturierte HybridQuery-Definition und Validierung.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

// FILE-CONTEXT
// ZWECK: Hybrid-Familie Submodule für Query-basierte Suche.

use crate::collection::search::filtered::{MAX_OVERFETCH_SCAN, OVERFETCH_STAGE_MULTIPLIERS};
use crate::collection::Collection;
use crate::fusion::{SearchReport, Signal, SignalFailurePolicy};
use crate::metrics_names::{record_histogram_safe, SEARCH_LATENCY_TEXT, SEARCH_LATENCY_VECTOR};
use contextra_ports::{MetricsSink, StorageEngine, TextIndex, VectorIndex};
use contextra_types::{DocId, Result};

/// Calculates effective candidate count limit based on query overfetch preferences and reranker settings.
pub(super) fn calc_hybrid_candidate_k(query: &contextra_types::HybridQuery, k: usize) -> usize {
    let overfetch =
        k.saturating_mul(Collection::<crate::LsmStorage, crate::HnswIndex>::OVERFETCH_FACTOR);
    if query.has_reranker {
        let pool_max = query.rerank_pool_max.unwrap_or(
            Collection::<crate::LsmStorage, crate::HnswIndex>::DEFAULT_MIN_RERANK_CANDIDATES,
        );
        overfetch.max(pool_max)
    } else {
        overfetch
    }
}

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Performs hybrid search combining BM25 and vector signals configured via `HybridQuery`.
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
        let seq = self.storage.last_seq_no().await?;
        self.hybrid_search_with_query_at(query, seq).await
    }

    /// Performs hybrid search combining BM25 and vector signals configured via `HybridQuery` at a specific snapshot sequence.
    // Begründung: Interne Abwärtskompatibilität für deprecated API bis Abschluss Stufe 2
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
    // Begründung: Interne Abwärtskompatibilität für deprecated API bis Abschluss Stufe 2
    #[allow(deprecated)]
    pub async fn hybrid_search_with_query_and_report_at(
        &self,
        query: &contextra_types::HybridQuery,
        seq: u64,
        _on_signal_failure: SignalFailurePolicy,
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

        let mut report = SearchReport::default();
        let is_text_empty = text.trim().is_empty();
        let candidate_k = calc_hybrid_candidate_k(query, k);
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
        let pre_matched_ids = self.get_matching_doc_ids_for_query_at(query, seq).await?;

        let default_sink = self.metrics_sink();
        let active_sink = metrics.unwrap_or(default_sink.as_ref());

        // 1. Vector Signal Evaluation
        let vector_results = self
            .evaluate_vector_signal_for_query(
                query,
                vector,
                seq,
                candidate_k,
                total_docs,
                is_filtered,
                &pre_matched_ids,
                &filter_pre_rrf,
                &mut report,
                active_sink,
            )
            .await?;

        // 2. Text Signal Evaluation
        let text_results = self
            .evaluate_text_signal_for_query(
                query,
                text,
                is_text_empty,
                seq,
                candidate_k,
                total_docs,
                is_filtered,
                &filter_pre_rrf,
                &mut report,
                active_sink,
            )
            .await?;

        if vector_results.is_empty() && text_results.is_empty() {
            return Ok((Vec::new(), report));
        }

        let fused_results = self
            .fuse_results(query, k, candidate_k, vector_results, text_results)
            .await?;

        let filtered_final = crate::collection::search::hydrate::apply_read_gate(fused_results);
        Ok((filtered_final, report))
    }

    /// Evaluates the vector signal for a hybrid query at a given snapshot sequence.
    #[allow(clippy::too_many_arguments)]
    async fn evaluate_vector_signal_for_query(
        &self,
        query: &contextra_types::HybridQuery,
        vector: &[f32],
        seq: u64,
        candidate_k: usize,
        total_docs: usize,
        is_filtered: bool,
        pre_matched_ids: &Option<std::collections::HashSet<DocId>>,
        filter_pre_rrf: &impl Fn(Vec<crate::SearchResult>) -> Vec<crate::SearchResult>,
        report: &mut SearchReport,
        active_sink: &dyn MetricsSink,
    ) -> Result<Vec<crate::SearchResult>> {
        let is_vector_zero = vector.is_empty() || vector.iter().all(|&v| v == 0.0);
        let vec_start_nanos = self.clock().monotonic_nanos();

        let vector_res: Result<Vec<crate::SearchResult>> = if is_vector_zero {
            Ok(Vec::new())
        } else if is_filtered {
            if let Some(ref ids) = pre_matched_ids {
                let filter_fn = |doc_id: DocId| ids.contains(&doc_id);
                if ids.len() <= candidate_k * 4 {
                    report.overfetch_stages = report.overfetch_stages.max(1);
                    // Begründung: Aufruf der bestehenden search_filtered_at Methode innerhalb der Engine
                    #[allow(deprecated)]
                    self.search_filtered_at(vector, candidate_k, Some(&filter_fn), seq)
                        .await
                } else {
                    let max_cap = total_docs.min(MAX_OVERFETCH_SCAN).max(candidate_k);
                    let mut final_filtered = Vec::new();

                    for (stage_idx, &stage_mult) in OVERFETCH_STAGE_MULTIPLIERS.iter().enumerate() {
                        report.overfetch_stages =
                            report.overfetch_stages.max((stage_idx + 1) as u8);
                        let oversample = (candidate_k * stage_mult).min(max_cap);
                        // Begründung: Aufruf der bestehenden search_filtered_at Methode innerhalb der Engine
                        #[allow(deprecated)]
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
                    report.overfetch_stages = report.overfetch_stages.max((stage_idx + 1) as u8);
                    let oversample = (candidate_k * stage_mult).min(max_cap);
                    #[allow(deprecated)]
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
            // Begründung: Aufruf der bestehenden search_filtered_at Methode innerhalb der Engine
            #[allow(deprecated)]
            self.search_filtered_at(vector, candidate_k, None, seq)
                .await
        };

        let elapsed_vec = (self
            .clock()
            .monotonic_nanos()
            .saturating_sub(vec_start_nanos)) as f64
            / 1_000_000_000.0;
        record_histogram_safe(active_sink, SEARCH_LATENCY_VECTOR, elapsed_vec, &[]);

        match vector_res {
            Ok(res) => Ok(res),
            Err(err) => {
                if query.on_signal_failure == SignalFailurePolicy::Fail {
                    Err(err)
                } else {
                    report.degraded_signals.push(Signal::Vector);
                    tracing::warn!("Vector search signal failed under Degrade policy: {err}");
                    Ok(Vec::new())
                }
            }
        }
    }

    /// Evaluates the text signal for a hybrid query at a given snapshot sequence.
    #[allow(clippy::too_many_arguments)]
    async fn evaluate_text_signal_for_query(
        &self,
        query: &contextra_types::HybridQuery,
        text: &str,
        is_text_empty: bool,
        seq: u64,
        candidate_k: usize,
        total_docs: usize,
        is_filtered: bool,
        filter_pre_rrf: &impl Fn(Vec<crate::SearchResult>) -> Vec<crate::SearchResult>,
        report: &mut SearchReport,
        active_sink: &dyn MetricsSink,
    ) -> Result<Vec<crate::SearchResult>> {
        let text_start_nanos = self.clock().monotonic_nanos();
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

        let elapsed_text = (self
            .clock()
            .monotonic_nanos()
            .saturating_sub(text_start_nanos)) as f64
            / 1_000_000_000.0;
        record_histogram_safe(active_sink, SEARCH_LATENCY_TEXT, elapsed_text, &[]);

        match text_res {
            Ok(res) => Ok(res),
            Err(err) => {
                if query.on_signal_failure == SignalFailurePolicy::Fail {
                    Err(err)
                } else {
                    report.degraded_signals.push(Signal::Text);
                    tracing::warn!("Text search signal failed under Degrade policy: {err}");
                    Ok(Vec::new())
                }
            }
        }
    }

    /// Fuses vector and text signal sets.
    async fn fuse_results(
        &self,
        query: &contextra_types::HybridQuery,
        k: usize,
        candidate_k: usize,
        vector_results: Vec<crate::SearchResult>,
        text_results: Vec<crate::SearchResult>,
    ) -> Result<Vec<crate::SearchResult>> {
        let (vw, tw, _) = crate::fusion::weights_to_signal_factors(Some(&query.fusion_weights));

        let mut signal_sets = Vec::new();
        if !vector_results.is_empty() {
            signal_sets.push(("vector".to_string(), vector_results, vw));
        }
        if !text_results.is_empty() {
            signal_sets.push(("text".to_string(), text_results, tw));
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

        fused_results.truncate(k);
        Ok(fused_results)
    }
}
