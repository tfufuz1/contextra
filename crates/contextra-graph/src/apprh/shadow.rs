//! Shadow comparison module comparing Forward-Push PPR against APPRH.

use super::diffusion::run_apprh_push;
use super::error::ApprhError;
use super::params::ApprhParams;
use crate::path_rag::{forward_push_ppr, PathGraph, PprParams};
use ahash::AHashSet;
use contextra_types::EntityId;
use std::cmp::Ordering;

/// Formal default-flip gate trait evaluating whether APPRH can replace ForwardPush as default.
pub trait ApprhFlipGate {
    /// Evaluates whether APPRH meets default flip criteria.
    fn should_flip(&self, report: &ApprhShadowComparison) -> bool;
}

/// Standard default flip gate evaluator for APPRH.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DefaultApprhFlipGate;

impl ApprhFlipGate for DefaultApprhFlipGate {
    fn should_flip(&self, report: &ApprhShadowComparison) -> bool {
        !report.discrepancy && report.top_k_overlap >= 0.85
    }
}

/// Result structure of shadow mode comparison between Forward-Push PPR and APPRH.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprhShadowComparison {
    /// Maximum absolute score difference across top-k union nodes.
    pub max_abs_diff: f32,
    /// Jaccard overlap ratio between top-k entity ID sets [0.0, 1.0].
    pub top_k_overlap: f32,
    /// Boolean indicator whether discrepancy threshold was breached.
    pub discrepancy: bool,
    /// Authoritative L1-normalized Forward-Push PPR results.
    pub forward_push_results: Vec<(EntityId, f32)>,
    /// L1-normalized APPRH results.
    pub apprh_results: Vec<(EntityId, f32)>,
}

/// Computes shadow comparison between Forward-Push PPR and APPRH.
///
/// # Invariants
/// - Authoritative result is Forward-Push PPR.
/// - Emits `tracing::warn!` when discrepancy threshold is breached.
///
/// # Errors
/// Returns [`ApprhError`] if APPRH execution fails.
pub fn shadow_compare_forward_push_vs_apprh<G: PathGraph>(
    graph: &G,
    seeds: &[EntityId],
    ppr: &PprParams,
    apprh_params: &ApprhParams,
    top_k: usize,
) -> Result<ApprhShadowComparison, ApprhError> {
    let fp_map = forward_push_ppr(graph, seeds, ppr);
    let apprh_map = run_apprh_push(graph, seeds, apprh_params)?;

    let fp_sorted = normalize_and_sort(&fp_map);
    let apprh_sorted = normalize_and_sort(&apprh_map);

    let k_fp = top_k.min(fp_sorted.len());
    let k_apprh = top_k.min(apprh_sorted.len());

    let top_fp = &fp_sorted[..k_fp];
    let top_apprh = &apprh_sorted[..k_apprh];

    let set_fp: AHashSet<EntityId> = top_fp.iter().map(|(id, _)| *id).collect();
    let set_apprh: AHashSet<EntityId> = top_apprh.iter().map(|(id, _)| *id).collect();

    let intersection_count = set_fp.intersection(&set_apprh).count();
    let union_set: AHashSet<EntityId> = set_fp.union(&set_apprh).copied().collect();

    let top_k_overlap = if union_set.is_empty() {
        1.0
    } else {
        intersection_count as f32 / union_set.len() as f32
    };

    let fp_score_map: ahash::AHashMap<EntityId, f32> = fp_sorted.iter().copied().collect();
    let apprh_score_map: ahash::AHashMap<EntityId, f32> = apprh_sorted.iter().copied().collect();

    let mut max_abs_diff = 0.0f32;
    for &node in &union_set {
        let s_fp = fp_score_map.get(&node).copied().unwrap_or(0.0);
        let s_apprh = apprh_score_map.get(&node).copied().unwrap_or(0.0);
        let diff = (s_fp - s_apprh).abs();
        if diff > max_abs_diff {
            max_abs_diff = diff;
        }
    }

    let discrepancy = max_abs_diff > ppr.epsilon * 10.0 || top_fp.len() != top_apprh.len();

    if discrepancy {
        tracing::warn!(
            max_diff = max_abs_diff,
            forward_push_count = fp_sorted.len(),
            apprh_count = apprh_sorted.len(),
            seed_count = seeds.len(),
            "APPRH Shadow Mode discrepancy detected between Forward Push and APPRH"
        );
    }

    Ok(ApprhShadowComparison {
        max_abs_diff,
        top_k_overlap,
        discrepancy,
        forward_push_results: fp_sorted,
        apprh_results: apprh_sorted,
    })
}

fn normalize_and_sort(map: &ahash::AHashMap<EntityId, f32>) -> Vec<(EntityId, f32)> {
    let sum: f32 = map.values().sum();
    let mut vec: Vec<(EntityId, f32)> = if sum > 0.0 {
        map.iter().map(|(&id, &val)| (id, val / sum)).collect()
    } else {
        map.iter().map(|(&id, &val)| (id, val)).collect()
    };

    vec.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    vec
}
