//! Shadow comparison module comparing Forward-Push PPR against TL-HFD (AK-16).

use super::diffusion::run_diffusion;
use super::error::TlHfdError;
use super::params::TlHfdParams;
use crate::path_rag::{forward_push_ppr, PathGraph, PprParams};
use ahash::AHashSet;
use contextra_types::EntityId;
use std::cmp::Ordering;

/// Result structure of shadow mode comparison between Forward-Push PPR and TL-HFD.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowComparison {
    /// Maximum absolute score difference across top-k union nodes.
    pub max_abs_diff: f32,
    /// Jaccard overlap ratio between top-k entity ID sets [0.0, 1.0].
    pub top_k_overlap: f32,
    /// Boolean indicator whether discrepancy threshold was breached.
    pub discrepancy: bool,
    /// Authoritative L1-normalized Forward-Push PPR results.
    pub forward_push_results: Vec<(EntityId, f32)>,
    /// L1-normalized TL-HFD diffusion results.
    pub tl_hfd_results: Vec<(EntityId, f32)>,
}

/// Computes shadow comparison between Forward-Push PPR and TL-HFD.
///
/// # Invariants
/// - Authoritative result is Forward-Push PPR.
/// - Emits `tracing::warn!` when discrepancy threshold is breached.
///
/// # Complexity
/// Time complexity $O(\text{iterations} \cdot k \cdot |\text{Seeds}| \cdot \text{max\_edge\_size} + |V| \log |V|)$.
pub fn shadow_compare_forward_push_vs_tl_hfd<G: PathGraph>(
    graph: &G,
    seeds: &[EntityId],
    ppr: &PprParams,
    tl: &TlHfdParams,
    top_k: usize,
) -> Result<ShadowComparison, TlHfdError> {
    let fp_map = forward_push_ppr(graph, seeds, ppr);
    let tl_map = run_diffusion(graph, seeds, tl)?;

    let fp_sorted = normalize_and_sort(&fp_map);
    let tl_sorted = normalize_and_sort(&tl_map);

    let k_fp = top_k.min(fp_sorted.len());
    let k_tl = top_k.min(tl_sorted.len());

    let top_fp = &fp_sorted[..k_fp];
    let top_tl = &tl_sorted[..k_tl];

    let set_fp: AHashSet<EntityId> = top_fp.iter().map(|(id, _)| *id).collect();
    let set_tl: AHashSet<EntityId> = top_tl.iter().map(|(id, _)| *id).collect();

    let intersection_count = set_fp.intersection(&set_tl).count();
    let union_set: AHashSet<EntityId> = set_fp.union(&set_tl).copied().collect();

    let top_k_overlap = if union_set.is_empty() {
        1.0
    } else {
        intersection_count as f32 / union_set.len() as f32
    };

    let fp_score_map: ahash::AHashMap<EntityId, f32> = fp_sorted.iter().copied().collect();
    let tl_score_map: ahash::AHashMap<EntityId, f32> = tl_sorted.iter().copied().collect();

    let mut max_abs_diff = 0.0f32;
    for &node in &union_set {
        let s_fp = fp_score_map.get(&node).copied().unwrap_or(0.0);
        let s_tl = tl_score_map.get(&node).copied().unwrap_or(0.0);
        let diff = (s_fp - s_tl).abs();
        if diff > max_abs_diff {
            max_abs_diff = diff;
        }
    }

    let discrepancy = max_abs_diff > ppr.epsilon * 10.0 || top_fp.len() != top_tl.len();

    if discrepancy {
        tracing::warn!(
            max_diff = max_abs_diff,
            forward_push_count = fp_sorted.len(),
            tl_hfd_count = tl_sorted.len(),
            seed_count = seeds.len(),
            "TL-HFD Shadow Mode discrepancy detected between Forward Push and TL-HFD"
        );
    }

    Ok(ShadowComparison {
        max_abs_diff,
        top_k_overlap,
        discrepancy,
        forward_push_results: fp_sorted,
        tl_hfd_results: tl_sorted,
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
