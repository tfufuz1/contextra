//! # Score-Normalisierte Fusion (`normalized`)
//!
//! **Ring:** Ring 0 (Ranking & Kalibrierung)
//!
//! Implementiert die lineare Score-Kombination nach Min-Max-Normalisierung.

use super::resonance::{apply_resonance_bonus, ResonanceConfig};
use super::rrf::{
    format_fused_results_from_top_k, update_fused_entry_from_doc,
    weighted_reciprocal_rank_fusion_with_options,
};
use super::signal::{FusedEntry, MetadataMergePriority, SignalKey};
use super::types::SearchResult;
use ahash::AHashMap;

fn accumulate_normalized_scores<'a>(
    result_sets: &'a [(String, Vec<SearchResult>, f32)],
    id_to_idx: &mut AHashMap<&'a str, u32>,
    id_table: &mut Vec<&'a str>,
    scores: &mut Vec<f32>,
    entries: &mut Vec<FusedEntry<'a>>,
) -> usize {
    let mut valid_signal_count = 0usize;
    for (signal_name, result_set, weight) in result_sets {
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
            let rank = u32::try_from(rank_idx.saturating_add(1)).unwrap_or(u32::MAX);
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
                    if let Ok(new_idx_u32) = u32::try_from(new_idx) {
                        id_to_idx.insert(doc_id_str, new_idx_u32);
                    }
                    id_table.push(doc_id_str);
                    scores.push(0.0_f32);
                    entries.push(FusedEntry::default());
                    new_idx
                }
            };
            if let Some(s) = scores.get_mut(idx) {
                *s += weighted_score;
            }
            if let Some(e) = entries.get_mut(idx) {
                update_fused_entry_from_doc(e, doc, rank, weighted_score, sig_key, signal_kind);
            }
        }
    }
    valid_signal_count
}

fn check_signal_set_degradation(active_sets: &[&(String, Vec<SearchResult>, f32)]) -> bool {
    let total_active_signals = active_sets.len();

    for (sig_name, set, _) in active_sets {
        if total_active_signals > 1 && set.is_empty() {
            tracing::warn!(
                signal = %sig_name,
                strategy = "ScoreNormalized",
                fallback = "RRF",
                "Signal degradation detected: active signal returned 0 candidates; falling back to RRF"
            );
            return true;
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
                return true;
            }
        }
    }

    false
}

/// Score-normalized fusion (CombSUM with MinMax normalization).
// ANCHOR[contextra-rank:NORM-001]
/// Führt eine score-normalisierte lineare Fusion mehrerer Signal-Ergebnismengen durch.
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

    if check_signal_set_degradation(&active_sets) {
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
    let valid_signal_count = accumulate_normalized_scores(
        &result_sets,
        &mut id_to_idx,
        &mut id_table,
        &mut scores,
        &mut entries,
    );

    let results = format_fused_results_from_top_k(
        &scores,
        &id_table,
        &mut entries,
        max_results,
        include_provenance,
        60.0,
    );

    if let Some(cfg) = resonance_config {
        apply_resonance_bonus(results, valid_signal_count, cfg)
    } else {
        results
    }
}

/// Converts optional `FusionWeights` into (vector, text, graph) weight tuple.
// ANCHOR[contextra-rank:NORM-002]
/// Konvertiert relative Signalgewichte in Normalisierungsfaktoren.
pub fn weights_to_signal_factors(
    weights: Option<&contextra_types::FusionWeights>,
) -> (f32, f32, f32) {
    match weights {
        Some(w) => (w.vector(), w.text(), w.graph()),
        None => (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0),
    }
}
