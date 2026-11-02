//! # Reziproke Rangfusion (RRF) & mRRF (`rrf`)
//!
//! **Ring:** Ring 0 (Ranking & Kalibrierung)
//!
//! Dieses Modul implementiert die reziproke Rangfusion (RRF = Reciprocal Rank Fusion)
//! gemäß SPEC §8.13 sowie abwärtskompatible Hüllen.

use super::normalized::score_normalized_fusion_with_options;
use super::provenance::ProvenanceBuilder;
use super::resonance::{apply_resonance_bonus, ResonanceConfig};
use super::signal::{FusedEntry, MetadataMergePriority, SignalKey, SignalKind};
use super::topk::{BoundedTopK, TopKCandidate};
use super::types::{Hit, ProvenanceRecord, SearchResult, SignalContribution};
use ahash::AHashMap;
pub use contextra_types::FusionStrategy;

/// Standard-Glättungsparameter $k = 60$ für Reciprocal Rank Fusion (SPEC §8.13).
pub const DEFAULT_RRF_K: u32 = 60;

/// Kanonische Reciprocal-Rank-Fusion gemäß SPEC §8.13 / AGENTS.md.
///
/// Führt mehrere sortierte Trefferlisten (`lists`) mit dem Glättungsfaktor `k`
/// (Standard 60, falls `k == 0`) zusammen und gibt die `top` besten Treffer zurück.
///
/// # Invarianten
/// 1. **Permutationsinvarianz:** Die Reihenfolge der Eingabelisten in `lists` beeinflusst weder Scores noch Rangfolge.
/// 2. **Deterministischer Tie-Break:** Sortierung primär nach Score absteigend, bei Gleichheit deterministisch
///    nach aufsteigendem `doc_tag` (falls vorhanden) bzw. aufsteigender String-`id`.
/// 3. **NaN/Inf-Sicherheit:** Nicht-endliche Scores werden sicher ignoriert bzw. führen zu keinen Panics/Inkonsistenzen.
pub fn rrf_fuse(lists: &[Vec<Hit>], k: u32, top: usize) -> Vec<Hit> {
    if top == 0 || lists.is_empty() {
        return Vec::new();
    }

    let k_val = if k == 0 { DEFAULT_RRF_K } else { k };
    let k_f32 = k_val as f32;

    let mut id_to_idx: AHashMap<String, usize> = AHashMap::new();
    let mut entries: Vec<HitEntry> = Vec::new();

    for list in lists {
        for (rank_idx, hit) in list.iter().enumerate() {
            let rank = match u32::try_from(rank_idx.saturating_add(1)) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let denom = k_f32 + rank as f32;
            let contribution = 1.0_f32 / denom;
            let contribution = if contribution.is_finite() && contribution > 0.0 {
                contribution
            } else {
                0.0
            };

            match id_to_idx.get(&hit.id) {
                Some(&idx) => {
                    if let Some(entry) = entries.get_mut(idx) {
                        entry.score += contribution;
                        if entry.doc_tag.is_none() && hit.doc_tag.is_some() {
                            entry.doc_tag = hit.doc_tag;
                        }
                        merge_metadata_ref(&mut entry.metadata, &hit.metadata);
                    }
                }
                None => {
                    let new_idx = entries.len();
                    id_to_idx.insert(hit.id.clone(), new_idx);
                    entries.push(HitEntry {
                        id: hit.id.clone(),
                        score: contribution,
                        doc_tag: hit.doc_tag,
                        metadata: hit.metadata.clone(),
                    });
                }
            }
        }
    }

    if entries.is_empty() {
        return Vec::new();
    }

    entries.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| match (a.doc_tag, b.doc_tag) {
                (Some(tag_a), Some(tag_b)) => tag_a.cmp(&tag_b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            })
            .then_with(|| a.id.cmp(&b.id))
    });

    entries.truncate(top);

    entries
        .into_iter()
        .map(|e| Hit {
            id: e.id,
            score: e.score,
            doc_tag: e.doc_tag,
            metadata: e.metadata,
        })
        .collect()
}

struct HitEntry {
    id: String,
    score: f32,
    doc_tag: Option<[u8; 16]>,
    metadata: Option<serde_json::Value>,
}

/// Bounded smooth influence function $g(x) = \frac{x}{1 + |x|}$.
///
/// Returns $x / (1 + |x|)$, bounded in $(-1.0, 1.0)$ for all finite $x$.
/// Returns `0.0` if $x$ is non-finite (`NaN` or `Infinity`).
// ANCHOR[contextra-rank:RRF-001]
/// Glatte, beschränkte Einflussfunktion $g(x) = \frac{x}{1 + |x|}$.
pub fn g(x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    x / (1.0 + x.abs())
}

/// Fuses multiple sets of ranked search results into a single ranked list using Reciprocal Rank Fusion (RRF).
// ANCHOR[contextra-rank:RRF-002]
/// Führt mehrere ungewichtete Signal-Ergebnislisten mittels Standard-RRF ($k=60$) zusammen.
#[deprecated(note = "Nutze rrf_fuse als kanonische Schnittstelle (SPEC §8.13).")]
pub fn reciprocal_rank_fusion(
    result_sets: Vec<Vec<SearchResult>>,
    max_results: usize,
) -> Vec<SearchResult> {
    let weighted_sets = result_sets
        .into_iter()
        .map(|set| ("unnamed".to_string(), set, 1.0))
        .collect();
    // Begründung: Interner Aufruf zur Delegierung an abwärtskompatible Hülle.
    #[allow(deprecated)]
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
// ANCHOR[contextra-rank:RRF-003]
/// Gewichtete reziproke Rangfusion (Weighted RRF) mit Standard-Metadatenpriorität.
#[deprecated(note = "Nutze rrf_fuse als kanonische Schnittstelle (SPEC §8.13).")]
pub fn weighted_reciprocal_rank_fusion(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
) -> Vec<SearchResult> {
    // Begründung: Interner Aufruf zur Delegierung an abwärtskompatible Hülle.
    #[allow(deprecated)]
    weighted_reciprocal_rank_fusion_with_options(
        result_sets,
        max_results,
        MetadataMergePriority::default(),
        true,
        None,
    )
}

/// Weighted Reciprocal Rank Fusion with explicit metadata merge priority.
// ANCHOR[contextra-rank:RRF-004]
/// Gewichtete reziproke Rangfusion mit expliziter Metadaten-Zusammenführungspriorität.
#[deprecated(note = "Nutze rrf_fuse als kanonische Schnittstelle (SPEC §8.13).")]
pub fn weighted_reciprocal_rank_fusion_with_priority(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
) -> Vec<SearchResult> {
    // Begründung: Interner Aufruf zur Delegierung an abwärtskompatible Hülle.
    #[allow(deprecated)]
    weighted_reciprocal_rank_fusion_with_options(result_sets, max_results, priority, true, None)
}

/// Modulates signal weights using mRRF margin formula.
// ANCHOR[contextra-rank:RRF-005]
/// Moduliert Signalgewichte anhand der Margin-Differenz (mRRF) und renormalisiert sie.
pub fn modulate_and_renormalize_weights(
    result_sets: &[(String, Vec<SearchResult>, f32)],
    _calibration_contexts: Option<&()>,
    beta: f32,
) -> Vec<f32> {
    let raw_weights: Vec<f32> = result_sets.iter().map(|(_, _, w)| *w).collect();
    if beta == 0.0 || !beta.is_finite() {
        return raw_weights;
    }

    let mut total_orig = 0.0_f32;
    let mut modulated_weights = Vec::with_capacity(result_sets.len());

    for (_signal_name, _set, weight) in result_sets.iter() {
        let w = *weight;
        if !w.is_finite() || w <= 0.0 {
            modulated_weights.push(w);
            continue;
        }

        total_orig += w;

        let m_s = 0.0_f32;
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
// ANCHOR[contextra-rank:RRF-006]
/// Vollständig konfigurierbare gewichtete Rangfusion mit Resonanz- und Provenance-Optionen.
#[deprecated(note = "Nutze rrf_fuse als kanonische Schnittstelle (SPEC §8.13).")]
pub fn weighted_reciprocal_rank_fusion_with_options(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
) -> Vec<SearchResult> {
    // Begründung: Interner Aufruf zur Delegierung an abwärtskompatible Hülle.
    #[allow(deprecated)]
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

pub(super) fn update_fused_entry_from_doc<'a>(
    entry: &mut FusedEntry<'a>,
    doc: &'a SearchResult,
    rank: u32,
    weighted_score: f32,
    sig_key: Option<SignalKey<'a>>,
    signal_kind: Option<SignalKind>,
) {
    entry.deferred_metadata.push(&doc.metadata);
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
}

fn build_provenance_for_fused_entry(
    entry: &mut FusedEntry<'_>,
    include_provenance: bool,
    rrf_k: f32,
) -> Option<ProvenanceRecord> {
    if !include_provenance {
        return None;
    }
    let mut builder = ProvenanceBuilder::new(rrf_k);
    if let Some(score) = entry.vector_distance {
        let rank = entry
            .signal_ranks
            .get(&SignalKey::Known(SignalKind::Vector))
            .copied()
            .unwrap_or(0);
        let contrib = entry
            .signal_contributions
            .get(&SignalKey::Known(SignalKind::Vector))
            .map(|c| c.rrf_contribution)
            .unwrap_or(0.0);
        builder = builder.vector(score, rank, contrib);
    }
    if let Some(score) = entry.bm25_score {
        let rank = entry
            .signal_ranks
            .get(&SignalKey::Known(SignalKind::Text))
            .copied()
            .unwrap_or(0);
        let contrib = entry
            .signal_contributions
            .get(&SignalKey::Known(SignalKind::Text))
            .map(|c| c.rrf_contribution)
            .unwrap_or(0.0);
        builder = builder.bm25(score, rank, contrib);
    }
    if let Some(score) = entry.graph_score {
        let rank = entry
            .signal_ranks
            .get(&SignalKey::Known(SignalKind::Graph))
            .copied()
            .unwrap_or(0);
        let contrib = entry
            .signal_contributions
            .get(&SignalKey::Known(SignalKind::Graph))
            .map(|c| c.rrf_contribution)
            .unwrap_or(0.0);
        builder = builder.graph(score, rank, contrib);
    }
    if let Some(score) = entry.rerank_score {
        builder = builder.rerank_score(score);
    }
    if let Some(src) = entry.source_collection {
        builder = builder.source_collection(src);
    }
    if let Some(idx) = entry.index_type {
        builder = builder.index_type(idx);
    }

    let mut prov = builder.build();
    for (k, v) in entry.signal_ranks.drain() {
        prov.signal_ranks.insert(k.as_str().to_string(), v);
    }
    for (k, v) in entry.signal_contributions.drain() {
        prov.signal_contributions.insert(k.as_str().to_string(), v);
    }
    Some(prov)
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
    weights: &[f32],
    rrf_k: f32,
    id_to_idx: &mut AHashMap<&'a str, u32>,
    id_table: &mut Vec<&'a str>,
    scores: &mut Vec<f32>,
    entries: &mut Vec<FusedEntry<'a>>,
) -> usize {
    let mut valid_signal_count = 0usize;

    for (idx, (signal_name, result_set, _orig_weight)) in result_sets.iter().enumerate() {
        let weight = match weights.get(idx) {
            Some(&w) => w,
            None => continue,
        };
        if !weight.is_finite() || weight <= 0.0 {
            continue;
        }
        valid_signal_count += 1;
        let sig_key = SignalKey::from_name(signal_name);
        let signal_kind = sig_key.and_then(|k| match k {
            SignalKey::Known(kind) => Some(kind),
            _ => None,
        });

        for (rank_idx, doc) in result_set.iter().enumerate() {
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
// ANCHOR[contextra-rank:RRF-007]
/// Gewichtete Rangfusion mit margin-basierter mRRF-Gewichtsmodulation.
#[deprecated(note = "Nutze rrf_fuse als kanonische Schnittstelle (SPEC §8.13).")]
pub fn weighted_reciprocal_rank_fusion_mrrf(
    mut result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
    _calibration_contexts: Option<&()>,
    beta: f32,
) -> Vec<SearchResult> {
    if max_results == 0 {
        return Vec::new();
    }

    result_sets.sort_by_key(|(signal_name, _, _)| priority.signal_rank(signal_name));

    let modulated_weights = modulate_and_renormalize_weights(&result_sets, None, beta);

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

/// Fuses search result sets using per-signal fusion strategies (`SignalFusionStrategies`).
// ANCHOR[contextra-rank:RRF-008]
/// Führt Teilergebnisse mit individuellen Strategien je Signalart zusammen.
#[deprecated(note = "Nutze rrf_fuse als kanonische Schnittstelle (SPEC §8.13).")]
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
            FusionStrategy::Rrf => {
                // Begründung: Interner Aufruf zur Delegierung an abwärtskompatible Hülle.
                #[allow(deprecated)]
                weighted_reciprocal_rank_fusion_with_options(
                    result_sets,
                    max_results,
                    priority,
                    include_provenance,
                    resonance_config,
                )
            }
            FusionStrategy::ScoreNormalized => score_normalized_fusion_with_options(
                result_sets,
                max_results,
                priority,
                include_provenance,
                resonance_config,
            ),
        };
    }

    result_sets.sort_by_key(|(signal_name, _, _)| priority.signal_rank(signal_name));
    let weights: Vec<f32> = result_sets.iter().map(|(_, _, w)| *w).collect();

    let k = 60;
    let mut id_to_idx: AHashMap<&str, u32> = AHashMap::new();
    let mut id_table: Vec<&str> = Vec::new();
    let mut scores: Vec<f32> = Vec::new();
    let mut entries: Vec<FusedEntry<'_>> = Vec::new();

    let valid_signal_count = accumulate_mrrf_scores(
        &result_sets,
        &weights,
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

/// Fuses search result sets using the specified `FusionStrategy`.
// ANCHOR[contextra-rank:RRF-009]
/// Universeller Einstiegspunkt zur Signal-Fusion basierend auf einer globalen `FusionStrategy`.
#[deprecated(note = "Nutze rrf_fuse als kanonische Schnittstelle (SPEC §8.13).")]
pub fn fuse_search_results_with_strategy(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
    strategy: FusionStrategy,
) -> Vec<SearchResult> {
    // Begründung: Interner Aufruf zur Delegierung an abwärtskompatible Hülle.
    #[allow(deprecated)]
    fuse_search_results_with_signal_strategies(
        result_sets,
        max_results,
        priority,
        include_provenance,
        resonance_config,
        strategy.into(),
    )
}
