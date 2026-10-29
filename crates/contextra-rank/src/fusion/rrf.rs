use super::normalized::score_normalized_fusion_with_options;
use super::provenance::ProvenanceBuilder;
use super::resonance::{apply_resonance_bonus, ResonanceConfig};
use super::signal::{
    FusedEntry, MetadataMergePriority, SignalCalibrationContext, SignalKey, SignalKind,
};
use super::topk::{BoundedTopK, TopKCandidate};
use super::types::{ProvenanceRecord, SearchResult, SignalContribution};
use ahash::AHashMap;
pub use contextra_types::FusionStrategy;

/// Bounded smooth influence function $g(x) = \frac{x}{1 + |x|}$.
///
/// Returns $x / (1 + |x|)$, bounded in $(-1.0, 1.0)$ for all finite $x$.
/// Returns `0.0` if $x$ is non-finite (`NaN` or `Infinity`).
pub fn g(x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    x / (1.0 + x.abs())
}

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

/// Modulates signal weights using mRRF margin formula:
/// $m_s = S_1 - S_2$ (when signal is calibrated AND `drift_status` is `Stable` and $\ge 2$ candidates exist).
/// $w_s' = w_s \times (1 + g(\beta \times m_s))$.
/// Renormalizes modulated weights to maintain sum equal to original weight sum (preserving weight norm in [0.999, 1.001]).
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

/// Weighted Reciprocal Rank Fusion with options.
pub fn weighted_reciprocal_rank_fusion_with_options(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
) -> Vec<SearchResult> {
    weighted_reciprocal_rank_fusion_mrrf(
        result_sets,
        max_results,
        priority,
        include_provenance,
        resonance_config,
        None,
        0.0,
    )
}

fn update_entry_kind_scores<'a>(
    entry: &mut FusedEntry<'a>,
    raw_score: f32,
    signal_kind: Option<SignalKind>,
) {
    match signal_kind {
        Some(SignalKind::Vector) => {
            if entry.vector_distance.is_none() {
                entry.vector_distance = Some(raw_score);
            }
            if entry.index_type.is_none() {
                entry.index_type = Some("hnsw");
            }
        }
        Some(SignalKind::Text) => {
            if entry.bm25_score.is_none() {
                entry.bm25_score = Some(raw_score);
            }
            if entry.index_type.is_none() {
                entry.index_type = Some("bm25");
            }
        }
        Some(SignalKind::Graph) => {
            if entry.graph_score.is_none() {
                entry.graph_score = Some(raw_score);
            }
            if entry.index_type.is_none() {
                entry.index_type = Some("graph");
            }
        }
        Some(SignalKind::EdgeReinforcement) => {
            if entry.graph_score.is_none() {
                entry.graph_score = Some(raw_score);
            }
            if entry.index_type.is_none() {
                entry.index_type = Some("edge-reinforcement");
            }
        }
        None => {}
    }
}

fn merge_doc_provenance<'a>(entry: &mut FusedEntry<'a>, doc_prov: &'a ProvenanceRecord) {
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

pub(super) fn update_fused_entry_from_doc<'a>(
    entry: &mut FusedEntry<'a>,
    doc: &'a SearchResult,
    rank: u32,
    weighted_score: f32,
    sig_key: Option<SignalKey<'a>>,
    signal_kind: Option<SignalKind>,
) {
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

    update_entry_kind_scores(entry, doc.score, signal_kind);

    if let Some(ref doc_prov) = doc.provenance {
        merge_doc_provenance(entry, doc_prov);
    }
}

fn synthesize_provenance_fallback(prov: ProvenanceRecord, rrf_k: f32) -> ProvenanceRecord {
    let mut builder = ProvenanceBuilder::new(rrf_k);
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
    if let (Some(score), Some(rank)) = (prov.graph_score, prov.signal_ranks.get("graph").copied()) {
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
}

fn build_provenance_for_fused_entry(
    entry: &mut FusedEntry<'_>,
    include_provenance: bool,
    rrf_k: f32,
) -> Option<ProvenanceRecord> {
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

    if include_provenance
        && (prov.vector_distance.is_some()
            || prov.bm25_score.is_some()
            || prov.graph_score.is_some()
            || prov.rerank_score.is_some()
            || !prov.signal_ranks.is_empty()
            || prov.source_collection.is_some()
            || prov.index_type.is_some())
    {
        if prov.signal_contributions.is_empty() {
            Some(synthesize_provenance_fallback(prov, rrf_k))
        } else {
            Some(prov)
        }
    } else {
        None
    }
}

pub(super) fn format_fused_results_from_top_k(
    scores: &[f32],
    id_table: &[&str],
    entries: &mut [FusedEntry<'_>],
    max_results: usize,
    include_provenance: bool,
    rrf_k: f32,
) -> Vec<SearchResult> {
    let mut top_k = BoundedTopK::new(max_results);
    for idx in 0..u32::try_from(scores.len()).unwrap_or(u32::MAX) {
        let i = idx as usize;
        if let (Some(&score), Some(&id)) = (scores.get(i), id_table.get(i)) {
            top_k.push(TopKCandidate { idx, score, id });
        }
    }

    let ranked_candidates = top_k.into_sorted_vec();
    let mut results: Vec<SearchResult> = Vec::with_capacity(ranked_candidates.len());
    for cand in ranked_candidates {
        let i = cand.idx as usize;
        let id = match id_table.get(i) {
            Some(id_str) => id_str.to_string(),
            None => continue,
        };
        let score = match scores.get(i) {
            Some(&s) => s,
            None => continue,
        };
        let entry = match entries.get_mut(i) {
            Some(e) => e,
            None => continue,
        };

        let mut merged_meta: Option<serde_json::Value> = None;
        for meta in entry.deferred_metadata.drain(..) {
            merge_metadata_ref(&mut merged_meta, meta);
        }

        let matched_signals = entry
            .matched_signals
            .iter()
            .map(|k| k.as_str().to_string())
            .collect();

        let provenance = build_provenance_for_fused_entry(entry, include_provenance, rrf_k);

        results.push(SearchResult {
            id,
            score,
            metadata: merged_meta,
            matched_signals,
            provenance,
        });
    }

    results
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

        for (rank_idx, doc) in result_set.iter().enumerate() {
            if !doc.score.is_finite() {
                tracing::error!(
                    signal = %signal_name,
                    doc_id = %doc.id,
                    raw_score = doc.score,
                    "RRF fusion: non-finite raw score from upstream signal detected"
                );
            }
            let rrf_rank = u32::try_from(rank_idx.saturating_add(1)).unwrap_or(u32::MAX);
            let denom = rrf_k + rrf_rank as f32;
            let score = weight / denom;
            let score = if score.is_finite() { score } else { 0.0 };

            let doc_id_str: &str = doc.id.as_str();
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

fn compute_doc_weighted_score(
    eff_strat: FusionStrategy,
    weight: f32,
    rank: u32,
    k_rrf: f32,
    doc_score: f32,
    norm_range: Option<(f32, f32)>,
) -> f32 {
    match eff_strat {
        FusionStrategy::Rrf => {
            let score = weight / (k_rrf + rank as f32);
            if score.is_finite() {
                score
            } else {
                0.0
            }
        }
        FusionStrategy::ScoreNormalized => {
            let (min_s, range) = norm_range.unwrap_or((0.0, 0.0));
            let norm_score = if range > 0.0 && doc_score.is_finite() {
                ((doc_score - min_s) / range).clamp(0.0, 1.0)
            } else {
                1.0
            };
            weight * norm_score
        }
    }
}

struct HybridAccumulatorContext<'a> {
    result_sets: &'a [(String, Vec<SearchResult>, f32)],
    effective_strategies: &'a AHashMap<String, FusionStrategy>,
    strategies: &'a contextra_types::SignalFusionStrategies,
    k_rrf: f32,
}

fn accumulate_hybrid_strategy_scores<'a>(
    ctx: HybridAccumulatorContext<'a>,
    id_to_idx: &mut AHashMap<&'a str, u32>,
    id_table: &mut Vec<&'a str>,
    scores: &mut Vec<f32>,
    entries: &mut Vec<FusedEntry<'a>>,
) -> usize {
    let mut valid_signal_count = 0usize;

    for (signal_name, result_set, weight) in ctx.result_sets {
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

        let eff_strat = ctx
            .effective_strategies
            .get(signal_name)
            .copied()
            .unwrap_or_else(|| ctx.strategies.strategy_for_signal(signal_name));

        let norm_range = if eff_strat == FusionStrategy::ScoreNormalized {
            let mut min_s = f32::INFINITY;
            let mut max_s = f32::NEG_INFINITY;
            for doc in result_set {
                min_s = min_s.min(doc.score);
                max_s = max_s.max(doc.score);
            }
            Some((min_s, max_s - min_s))
        } else {
            None
        };

        for (rank_idx, doc) in result_set.iter().enumerate() {
            let rank = u32::try_from(rank_idx.saturating_add(1)).unwrap_or(u32::MAX);

            let weighted_score = compute_doc_weighted_score(
                eff_strat, weight, rank, ctx.k_rrf, doc.score, norm_range,
            );

            let doc_id_str: &str = doc.id.as_str();
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
                *s += weighted_score;
            }
            if let Some(e) = entries.get_mut(entry_idx) {
                update_fused_entry_from_doc(e, doc, rank, weighted_score, sig_key, signal_kind);
            }
        }
    }

    valid_signal_count
}

fn evaluate_effective_signal_strategies(
    active_sets: &[&(String, Vec<SearchResult>, f32)],
    strategies: &contextra_types::SignalFusionStrategies,
) -> AHashMap<String, FusionStrategy> {
    let mut effective_strategies = AHashMap::new();
    let total_active_signals = active_sets.len();

    for (sig_name, set, _) in active_sets {
        let req_strat = strategies.strategy_for_signal(sig_name);
        let mut degraded = false;

        if req_strat == FusionStrategy::ScoreNormalized {
            if total_active_signals > 1 && set.is_empty() {
                tracing::warn!(
                    signal = %sig_name,
                    strategy = "ScoreNormalized",
                    fallback = "RRF",
                    "Signal degradation detected: active signal returned 0 candidates; falling back to RRF for this signal"
                );
                degraded = true;
            } else if !set.is_empty() {
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
                        "Signal degradation detected: degenerate score distribution; falling back to RRF for this signal"
                    );
                    degraded = true;
                }
            }
        }

        let eff_strat = if degraded {
            FusionStrategy::Rrf
        } else {
            req_strat
        };
        effective_strategies.insert((*sig_name).clone(), eff_strat);
    }

    effective_strategies
}

/// Fuses search result sets using per-signal fusion strategies (`SignalFusionStrategies`).
pub fn fuse_search_results_with_signal_strategies(
    mut result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
    strategies: contextra_types::SignalFusionStrategies,
) -> Vec<SearchResult> {
    if max_results == 0 {
        return Vec::new();
    }

    if strategies.vector == strategies.text && strategies.text == strategies.graph {
        return match strategies.vector {
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
        };
    }

    let active_sets: Vec<&(String, Vec<SearchResult>, f32)> = result_sets
        .iter()
        .filter(|(_, _, w)| w.is_finite() && *w > 0.0)
        .collect();

    if active_sets.is_empty() {
        return Vec::new();
    }

    let effective_strategies = evaluate_effective_signal_strategies(&active_sets, &strategies);

    result_sets.sort_by_key(|(signal_name, _, _)| priority.signal_rank(signal_name));

    let k_rrf = 60.0_f32;
    let mut id_to_idx: AHashMap<&str, u32> = AHashMap::new();
    let mut id_table: Vec<&str> = Vec::new();
    let mut scores: Vec<f32> = Vec::new();
    let mut entries: Vec<FusedEntry<'_>> = Vec::new();

    let valid_signal_count = accumulate_hybrid_strategy_scores(
        HybridAccumulatorContext {
            result_sets: &result_sets,
            effective_strategies: &effective_strategies,
            strategies: &strategies,
            k_rrf,
        },
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
        k_rrf,
    );

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
    fuse_search_results_with_signal_strategies(
        result_sets,
        max_results,
        priority,
        include_provenance,
        resonance_config,
        strategy.into(),
    )
}
