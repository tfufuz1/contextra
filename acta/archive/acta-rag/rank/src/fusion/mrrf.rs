//! # Margin-Modulierte Reziproke Rangfusion (mRRF) (`mrrf`)
//!
//! Dieses Modul implementiert die margin-modulierte Variante der reziproken
//! Rangfusion (mRRF). Es moduliert die Signalgewichte anhand der Differenz
//! (Margin) zwischen dem erst- und zweitplatzierten Ergebnis eines kalibrierten Signals.

use super::{apply_resonance_bonus, ResonanceConfig};
use super::rrf::{format_fused_results_from_top_k, g, update_fused_entry_from_doc};
use super::signal::{FusedEntry, MetadataMergePriority, SignalCalibrationContext, SignalKey};
use super::types::SearchResult;
use ahash::AHashMap;

/// Modulates signal weights using mRRF margin formula:
/// $m_s = S_1 - S_2$ (when signal is calibrated AND `drift_status` is `Stable` and $\ge 2$ candidates exist).
/// $w_s' = w_s \times (1 + g(\beta \times m_s))$.
/// Renormalizes modulated weights to maintain sum equal to original weight sum (preserving weight norm in [0.999, 1.001]).
/// Moduliert Signalgewichte anhand der Margin-Differenz (mRRF) und renormalisiert sie.
pub fn modulate_and_renormalize_weights(
    result_sets: &[(String, Vec<SearchResult>, f32)],
    calibration_contexts: Option<&[SignalCalibrationContext]>,
    beta: f32,
) -> Vec<f32> {
    let raw_weights: Vec<f32> = result_sets.iter().map(|(_, _, w)| *w).collect();
    if beta == 0.0 || !beta.is_finite() {
        // NAN-CHECK-OK
        return raw_weights;
    }

    let mut total_orig = 0.0_f32;
    let mut modulated_weights = Vec::with_capacity(result_sets.len());

    for (signal_name, set, weight) in result_sets.iter() {
        let w = *weight;
        if !w.is_finite() || w <= 0.0 {
            modulated_weights.push(w);
            continue;
        }

        total_orig += w;

        // Check calibration context if provided
        let cal_ctx = calibration_contexts.and_then(|ctxs| {
            ctxs.iter()
                .find(|c| c.signal_name.eq_ignore_ascii_case(signal_name))
        });

        let use_calibrated = cal_ctx.is_some_and(|c| c.should_use_calibrated_scores());

        let m_s = if use_calibrated {
            if set.len() >= 2 {
                if let (Some(doc0), Some(doc1)) = (set.first(), set.get(1)) {
                    let s1 = doc0.score;
                    let s2 = doc1.score;
                    if s1.is_finite() && s2.is_finite() {
                        (s1 - s2).max(0.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        let w_prime = w * (1.0 + g(beta * m_s));
        modulated_weights.push(if w_prime.is_finite() && w_prime > 0.0 {
            w_prime
        } else {
            w
        });
    }

    let total_mod: f32 = modulated_weights
        .iter()
        .filter(|w| w.is_finite() && **w > 0.0)
        .sum();

    if total_orig > 0.0 && total_mod > 0.0 {
        let renorm_factor = total_orig / total_mod;
        for w in &mut modulated_weights {
            if w.is_finite() && *w > 0.0 {
                *w *= renorm_factor;
            }
        }
    }

    modulated_weights
}

fn accumulate_mrrf_scores<'a>(
    result_sets: &'a [(String, Vec<SearchResult>, f32)],
    modulated_weights: &[f32],
    rrf_k: f32,
    id_to_idx: &mut AHashMap<&'a str, u32>,
    id_table: &mut Vec<&'a str>,
    scores: &mut Vec<f32>,
    entries: &mut Vec<FusedEntry<'a>>,
) -> usize {
    let mut valid_signal_count = 0usize;

    for (idx, (signal_name, result_set, _orig_weight)) in result_sets.iter().enumerate() {
        let weight = match modulated_weights.get(idx) {
            Some(&w) => w,
            None => continue,
        };
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

        let mut seen_in_signal: AHashMap<&str, usize> = AHashMap::new();
        for (rank_idx, doc) in result_set.iter().enumerate() {
            if !doc.score.is_finite() {
                tracing::error!(
                    signal = %signal_name,
                    doc_id = %doc.id,
                    raw_score = doc.score,
                    "RRF fusion: non-finite raw score from upstream signal detected"
                );
            }
            // Besten Rang innerhalb derselben Liste beibehalten (Deduplizierung)
            seen_in_signal.entry(doc.id.as_str()).or_insert(rank_idx);
        }

        for (doc_id_str, best_rank_idx) in seen_in_signal {
            let doc = match result_set.get(best_rank_idx) {
                Some(d) => d,
                None => continue,
            };
            let rrf_rank = u32::try_from(best_rank_idx.saturating_add(1)).unwrap_or(u32::MAX);
            let denom = rrf_k + rrf_rank as f32;
            let score = weight / denom;
            let score = if score.is_finite() { score } else { 0.0 };

            let entry_idx = match id_to_idx.get(doc_id_str) {
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
            if let Some(s) = scores.get_mut(entry_idx) {
                *s += score;
            }
            if let Some(e) = entries.get_mut(entry_idx) {
                update_fused_entry_from_doc(e, doc, rrf_rank, score, sig_key, signal_kind);
            }
        }
    }

    valid_signal_count
}

/// Weighted Reciprocal Rank Fusion with mRRF margin-based calibration weight modulation.
/// Gewichtete Rangfusion mit margin-basierter mRRF-Gewichtsmodulation.
pub fn weighted_reciprocal_rank_fusion_mrrf(
    mut result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
    calibration_contexts: Option<&[SignalCalibrationContext]>,
    beta: f32,
) -> Vec<SearchResult> {
    if max_results == 0 {
        return Vec::new();
    }

    result_sets.sort_by_key(|(signal_name, _, _)| priority.signal_rank(signal_name));

    let modulated_weights =
        modulate_and_renormalize_weights(&result_sets, calibration_contexts, beta);

    let k = 60;
    let mut id_to_idx: AHashMap<&str, u32> = AHashMap::new();
    let mut id_table: Vec<&str> = Vec::new();
    let mut scores: Vec<f32> = Vec::new();
    let mut entries: Vec<FusedEntry<'_>> = Vec::new();

    let valid_signal_count = accumulate_mrrf_scores(
        &result_sets,
        &modulated_weights,
        k as f32,
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
        k as f32,
    );

    if let Some(cfg) = resonance_config {
        apply_resonance_bonus(results, valid_signal_count, cfg)
    } else {
        results
    }
}
