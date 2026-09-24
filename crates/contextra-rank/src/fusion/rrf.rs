use super::normalized::score_normalized_fusion_with_options;
use super::provenance::ProvenanceBuilder;
use super::resonance::{apply_resonance_bonus, ResonanceConfig};
use super::signal::{FusedEntry, MetadataMergePriority, SignalKey, SignalKind};
use super::topk::{BoundedTopK, TopKCandidate};
use super::types::{ProvenanceRecord, SearchResult, SignalContribution};
use ahash::AHashMap;
pub use contextra_types::FusionStrategy;

/// Fuses multiple sets of ranked search results into a single ranked list using Reciprocal Rank Fusion (RRF).
pub fn reciprocal_rank_fusion(
    result_sets: Vec<Vec<SearchResult>>,
    max_results: usize,
) -> Vec<SearchResult> {
    let weighted_sets = result_sets
        .into_iter()
        .map(|set| ("unnamed".to_string(), set, 1.0))
        .collect();
    weighted_reciprocal_rank_fusion(weighted_sets, max_results)
}

pub(super) fn merge_metadata_ref(
    target: &mut Option<serde_json::Value>,
    source: &Option<serde_json::Value>,
) {
    if let Some(s_val) = source {
        match target {
            Some(t_val) => {
                if let (Some(t_obj), Some(s_obj)) = (t_val.as_object_mut(), s_val.as_object()) {
                    for (k, v) in s_obj {
                        if !t_obj.contains_key(k) {
                            t_obj.insert(k.clone(), v.clone());
                        }
                    }
                } else if t_val != s_val {
                    let taken_t_val = std::mem::take(t_val);
                    let arr = if let Some(s_arr) = s_val.as_array() {
                        let mut a = vec![taken_t_val];
                        for item in s_arr {
                            if !a.contains(item) {
                                a.push(item.clone());
                            }
                        }
                        a
                    } else {
                        vec![taken_t_val, s_val.clone()]
                    };
                    *t_val = serde_json::Value::Array(arr);
                }
            }
            None => {
                *target = Some(s_val.clone());
            }
        }
    }
}

/// Weighted Reciprocal Rank Fusion with default signal metadata priority (`VectorFirst`).
pub fn weighted_reciprocal_rank_fusion(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
) -> Vec<SearchResult> {
    weighted_reciprocal_rank_fusion_with_options(
        result_sets,
        max_results,
        MetadataMergePriority::default(),
        true,
        None,
    )
}

/// Weighted Reciprocal Rank Fusion with explicit metadata merge priority.
pub fn weighted_reciprocal_rank_fusion_with_priority(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
) -> Vec<SearchResult> {
    weighted_reciprocal_rank_fusion_with_options(result_sets, max_results, priority, true, None)
}

/// Weighted Reciprocal Rank Fusion with options.
pub fn weighted_reciprocal_rank_fusion_with_options(
    mut result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
) -> Vec<SearchResult> {
    if max_results == 0 {
        return Vec::new();
    }

    result_sets.sort_by_key(|(signal_name, _, _)| priority.signal_rank(signal_name));

    let k = 60;
    let mut id_to_idx: AHashMap<&str, u32> = AHashMap::new();
    let mut id_table: Vec<&str> = Vec::new();
    let mut scores: Vec<f32> = Vec::new();
    let mut entries: Vec<FusedEntry<'_>> = Vec::new();
    let mut valid_signal_count = 0usize;

    for (signal_name, result_set, weight) in &result_sets {
        let weight = *weight;
        if !weight.is_finite() || weight <= 0.0 {
            tracing::warn!(
                signal = %signal_name,
                weight,
                "RRF fusion: non-finite or non-positive weight skipped"
            );
            continue;
        }
        valid_signal_count += 1;
        let sig_key = SignalKey::from_name(signal_name);
        let signal_kind = sig_key.and_then(|k| match k {
            SignalKey::Known(kind) => Some(kind),
            _ => None,
        });

        let rrf_k = k as f32;
        debug_assert!(
            rrf_k >= 0.0,
            "rrf_k must be non-negative; division by zero risk"
        );

        for (rank_idx, doc) in result_set.iter().enumerate() {
            if !doc.score.is_finite() {
                tracing::error!(
                    signal = %signal_name,
                    doc_id = %doc.id,
                    raw_score = doc.score,
                    "RRF fusion: non-finite raw score from upstream signal detected"
                );
            }
            let rrf_rank = (rank_idx + 1) as u32;
            let denom = rrf_k + rrf_rank as f32;
            debug_assert!(denom > 0.0, "RRF denominator must be positive");
            let score = weight / denom;
            let score = if score.is_finite() { score } else { 0.0 };

            let doc_id_str: &str = doc.id.as_str();
            let idx = match id_to_idx.get(doc_id_str) {
                Some(&i) => i as usize,
                None => {
                    let new_idx = id_table.len();
                    id_to_idx.insert(doc_id_str, new_idx as u32);
                    id_table.push(doc_id_str);
                    scores.push(0.0_f32);
                    entries.push(FusedEntry::default());
                    new_idx
                }
            };
            scores[idx] += score;
            let entry = &mut entries[idx];

            if doc.metadata.is_some() {
                entry.deferred_metadata.push(&doc.metadata);
            }

            if let Some(key) = sig_key {
                if !entry.matched_signals.contains(&key) {
                    entry.matched_signals.push(key);
                }
                entry.signal_ranks.insert(key, rrf_rank);
                entry.signal_contributions.insert(
                    key,
                    SignalContribution {
                        raw_score: doc.score,
                        rank: rrf_rank,
                        rrf_contribution: score,
                    },
                );
            }

            match signal_kind {
                Some(SignalKind::Vector) => {
                    if entry.vector_distance.is_none() {
                        entry.vector_distance = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("hnsw");
                    }
                }
                Some(SignalKind::Text) => {
                    if entry.bm25_score.is_none() {
                        entry.bm25_score = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("bm25");
                    }
                }
                Some(SignalKind::Graph) => {
                    if entry.graph_score.is_none() {
                        entry.graph_score = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("graph");
                    }
                }
                Some(SignalKind::EdgeReinforcement) => {
                    if entry.graph_score.is_none() {
                        entry.graph_score = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("edge-reinforcement");
                    }
                }
                None => {}
            }

            if let Some(ref doc_prov) = doc.provenance {
                if entry.vector_distance.is_none() {
                    entry.vector_distance = doc_prov.vector_distance;
                }
                if entry.bm25_score.is_none() {
                    entry.bm25_score = doc_prov.bm25_score;
                }
                if entry.graph_score.is_none() {
                    entry.graph_score = doc_prov.graph_score;
                }
                if entry.rerank_score.is_none() {
                    entry.rerank_score = doc_prov.rerank_score;
                }
                if entry.source_collection.is_none() {
                    entry.source_collection = doc_prov.source_collection.as_deref();
                }
                if entry.index_type.is_none() {
                    entry.index_type = doc_prov.index_type.as_deref();
                }
                for (sig, r) in &doc_prov.signal_ranks {
                    if let Some(k) = SignalKey::from_name(sig.as_str()) {
                        entry.signal_ranks.entry(k).or_insert(*r);
                    } else {
                        entry
                            .extra_signal_ranks
                            .get_or_insert_with(AHashMap::new)
                            .entry(sig.clone())
                            .or_insert(*r);
                    }
                }
                for (sig, contrib) in &doc_prov.signal_contributions {
                    if let Some(k) = SignalKey::from_name(sig.as_str()) {
                        entry
                            .signal_contributions
                            .entry(k)
                            .or_insert_with(|| contrib.clone());
                    } else {
                        entry
                            .extra_signal_contributions
                            .get_or_insert_with(AHashMap::new)
                            .entry(sig.clone())
                            .or_insert_with(|| contrib.clone());
                    }
                }
            }
        }
    }

    let mut top_k = BoundedTopK::new(max_results);
    for idx in 0..scores.len() as u32 {
        let i = idx as usize;
        top_k.push(TopKCandidate {
            idx,
            score: scores[i],
            id: id_table[i],
        });
    }

    let ranked_candidates = top_k.into_sorted_vec();
    let mut results: Vec<SearchResult> = Vec::with_capacity(ranked_candidates.len());
    for cand in ranked_candidates {
        let i = cand.idx as usize;
        let id = id_table[i].to_string();
        let score = scores[i];
        let entry = &mut entries[i];

        let mut merged_meta: Option<serde_json::Value> = None;
        for meta in entry.deferred_metadata.drain(..) {
            merge_metadata_ref(&mut merged_meta, meta);
        }

        let matched_signals = entry
            .matched_signals
            .iter()
            .map(|k| k.as_str().to_string())
            .collect();

        let mut signal_ranks: AHashMap<String, u32> = entry
            .signal_ranks
            .drain()
            .map(|(k, v)| (k.as_str().to_string(), v))
            .collect();
        if let Some(extras) = entry.extra_signal_ranks.take() {
            for (k, v) in extras {
                signal_ranks.entry(k).or_insert(v);
            }
        }

        let mut signal_contributions: AHashMap<String, SignalContribution> = entry
            .signal_contributions
            .drain()
            .map(|(k, v)| (k.as_str().to_string(), v))
            .collect();
        if let Some(extras) = entry.extra_signal_contributions.take() {
            for (k, v) in extras {
                signal_contributions.entry(k).or_insert(v);
            }
        }

        let prov = ProvenanceRecord {
            vector_distance: entry.vector_distance,
            bm25_score: entry.bm25_score,
            graph_score: entry.graph_score,
            rerank_score: entry.rerank_score,
            signal_ranks,
            source_collection: entry.source_collection.map(|s| s.to_string()),
            index_type: entry.index_type.map(|s| s.to_string()),
            signal_contributions,
            coherence_bonus: 0.0,
        };

        let provenance = if include_provenance
            && (prov.vector_distance.is_some()
                || prov.bm25_score.is_some()
                || prov.graph_score.is_some()
                || prov.rerank_score.is_some()
                || !prov.signal_ranks.is_empty()
                || prov.source_collection.is_some()
                || prov.index_type.is_some())
        {
            let final_prov = if prov.signal_contributions.is_empty() {
                let mut builder = ProvenanceBuilder::new(k as f32);
                if let (Some(dist), Some(rank)) = (
                    prov.vector_distance,
                    prov.signal_ranks.get("vector").copied(),
                ) {
                    builder = builder.vector(dist, rank, None);
                }
                if let (Some(score), Some(rank)) = (
                    prov.bm25_score,
                    prov.signal_ranks
                        .get("text")
                        .copied()
                        .or_else(|| prov.signal_ranks.get("bm25").copied()),
                ) {
                    builder = builder.bm25(score, rank, None);
                }
                if let (Some(score), Some(rank)) =
                    (prov.graph_score, prov.signal_ranks.get("graph").copied())
                {
                    builder = builder.graph(score, rank, None);
                }
                if let Some(score) = prov.rerank_score {
                    builder = builder.rerank_score(score);
                }
                if let Some(col) = prov.source_collection {
                    builder = builder.source_collection(col);
                }
                if let Some(idx) = prov.index_type {
                    builder = builder.index_type(idx);
                }
                builder.build()
            } else {
                prov
            };

            Some(final_prov)
        } else {
            None
        };

        results.push(SearchResult {
            id,
            score,
            metadata: merged_meta,
            matched_signals,
            provenance,
        });
    }

    if let Some(cfg) = resonance_config {
        apply_resonance_bonus(results, valid_signal_count, cfg)
    } else {
        results
    }
}

/// Fuses search result sets using the specified `FusionStrategy`.
pub fn fuse_search_results_with_strategy(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
    strategy: FusionStrategy,
) -> Vec<SearchResult> {
    match strategy {
        FusionStrategy::Rrf => weighted_reciprocal_rank_fusion_with_options(
            result_sets,
            max_results,
            priority,
            include_provenance,
            resonance_config,
        ),
        FusionStrategy::ScoreNormalized => score_normalized_fusion_with_options(
            result_sets,
            max_results,
            priority,
            include_provenance,
            resonance_config,
        ),
    }
}
