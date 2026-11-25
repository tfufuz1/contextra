use super::resonance::{apply_resonance_bonus, ResonanceConfig};
use super::rrf::{merge_metadata_ref, weighted_reciprocal_rank_fusion_with_options};
use super::signal::{FusedEntry, MetadataMergePriority, SignalKey, SignalKind};
use super::topk::{BoundedTopK, TopKCandidate};
use super::types::{ProvenanceRecord, SearchResult, SignalContribution};
use ahash::AHashMap;

/// Score-normalized fusion (CombSUM with MinMax normalization).
pub fn score_normalized_fusion_with_options(
    mut result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
) -> Vec<SearchResult> {
    if max_results == 0 {
        return Vec::new();
    }

    let active_sets: Vec<&(String, Vec<SearchResult>, f32)> = result_sets
        .iter()
        .filter(|(_, _, w)| w.is_finite() && *w > 0.0)
        .collect();

    if active_sets.is_empty() {
        return Vec::new();
    }

    let mut degraded = false;
    let total_active_signals = active_sets.len();

    for (sig_name, set, _) in &active_sets {
        if total_active_signals > 1 && set.is_empty() {
            tracing::warn!(
                signal = %sig_name,
                strategy = "ScoreNormalized",
                fallback = "RRF",
                "Signal degradation detected: active signal returned 0 candidates; falling back to RRF"
            );
            degraded = true;
            break;
        }

        if !set.is_empty() {
            let mut min_s = f32::INFINITY;
            let mut max_s = f32::NEG_INFINITY;
            let mut non_finite = false;

            for doc in set.iter() {
                if !doc.score.is_finite() {
                    non_finite = true;
                    break;
                }
                min_s = min_s.min(doc.score);
                max_s = max_s.max(doc.score);
            }

            if non_finite || (set.len() > 1 && max_s <= min_s) {
                tracing::warn!(
                    signal = %sig_name,
                    min_s,
                    max_s,
                    non_finite,
                    strategy = "ScoreNormalized",
                    fallback = "RRF",
                    "Signal degradation detected: degenerate score distribution; falling back to RRF"
                );
                degraded = true;
                break;
            }
        }
    }

    if degraded {
        return weighted_reciprocal_rank_fusion_with_options(
            result_sets,
            max_results,
            priority,
            include_provenance,
            resonance_config,
        );
    }

    result_sets.sort_by_key(|(signal_name, _, _)| priority.signal_rank(signal_name));

    let mut id_to_idx: AHashMap<&str, u32> = AHashMap::new();
    let mut id_table: Vec<&str> = Vec::new();
    let mut scores: Vec<f32> = Vec::new();
    let mut entries: Vec<FusedEntry<'_>> = Vec::new();
    let mut valid_signal_count = 0usize;

    for (signal_name, result_set, weight) in &result_sets {
        let weight = *weight;
        if !weight.is_finite() || weight <= 0.0 || result_set.is_empty() {
            continue;
        }
        valid_signal_count += 1;
        let sig_key = SignalKey::from_name(signal_name);
        let signal_kind = sig_key.and_then(|k| match k {
            SignalKey::Known(kind) => Some(kind),
            _ => None,
        });

        let mut min_s = f32::INFINITY;
        let mut max_s = f32::NEG_INFINITY;
        for doc in result_set {
            min_s = min_s.min(doc.score);
            max_s = max_s.max(doc.score);
        }
        let range = max_s - min_s;

        for (rank_idx, doc) in result_set.iter().enumerate() {
            let rank = (rank_idx + 1) as u32;
            let norm_score = if range > 0.0 && doc.score.is_finite() {
                ((doc.score - min_s) / range).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let weighted_score = weight * norm_score;

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
            scores[idx] += weighted_score;
            let entry = &mut entries[idx];

            if doc.metadata.is_some() {
                entry.deferred_metadata.push(&doc.metadata);
            }

            if let Some(key) = sig_key {
                if !entry.matched_signals.contains(&key) {
                    entry.matched_signals.push(key);
                }
                entry.signal_ranks.insert(key, rank);
                entry.signal_contributions.insert(
                    key,
                    SignalContribution {
                        raw_score: doc.score,
                        rank,
                        rrf_contribution: weighted_score,
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

        let provenance = if include_provenance { Some(prov) } else { None };

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

/// Converts optional `FusionWeights` into (vector, text, graph) weight tuple.
pub fn weights_to_signal_factors(
    weights: Option<&contextra_types::FusionWeights>,
) -> (f32, f32, f32) {
    match weights {
        Some(w) => (w.vector(), w.text(), w.graph()),
        None => (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0),
    }
}
