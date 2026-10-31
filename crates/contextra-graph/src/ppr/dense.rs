// ANCHOR[contextra-graph:SRC-PPR-DENSE-001]
//! Dichte Matrix-/Vektorberechnung für personalisierten PageRank (PPR).
//!
//! # Zweck
//! Bietet klassische Matrixmultiplikation für kleine oder dicht vernetzte Teilgraphen.

use super::types::{DeletedView, PprContext};
use crate::csr::GraphInner;
use contextra_types::{EntityId, PprConfig};
use std::collections::HashSet;

pub(crate) fn collect_valid_seeds(
    inner: &GraphInner,
    seed_nodes: &[EntityId],
    deleted_nodes: &DeletedView,
    ctx: &mut PprContext,
) -> bool {
    let n = inner.reverse_map.len();
    ctx.valid_seeds.clear();
    let mut seen_seeds = HashSet::new();

    for &seed in seed_nodes {
        if let Some(&idx) = inner.id_map.get(&seed) {
            if idx < n
                && !deleted_nodes.contains(idx)
                && inner.entity_at(idx).is_some()
                && seen_seeds.insert(idx)
            {
                ctx.valid_seeds.push(idx);
            }
        }
    }

    !ctx.valid_seeds.is_empty()
}

fn compute_node_out_weight_sum(
    inner: &GraphInner,
    i: usize,
    n: usize,
    deleted_nodes: &DeletedView,
) -> f32 {
    let start = if i < inner.offsets.len() - 1 {
        inner.offsets.get(i).copied().unwrap_or(0)
    } else {
        0
    };
    let end = if i < inner.offsets.len() - 1 {
        inner.offsets.get(i + 1).copied().unwrap_or(0)
    } else {
        0
    };

    let mut sum = 0.0f32;
    for edge_idx in start..end {
        if let (Some(&target), Some(&weight)) =
            (inner.targets.get(edge_idx), inner.weights.get(edge_idx))
        {
            if !deleted_nodes.contains(target) && inner.entity_at(target).is_some() && weight > 0.0
            {
                sum += weight;
            }
        }
    }

    if let Some(pending) = inner.pending_edges.get(&i) {
        for edge in pending {
            let target = edge.target;
            if !deleted_nodes.contains(target)
                && inner.entity_at(target).is_some()
                && edge.weight > 0.0
            {
                sum += edge.weight;
            }
        }
    }

    if let Some(&u_eid) = inner.reverse_map.get(i) {
        if let Some(hedge_ids) = inner.hyperedge_index.get(&u_eid) {
            for hid in hedge_ids {
                if let Some(hedge) = inner.hyperedges.get(hid) {
                    if hedge.tx_valid_to.is_none() && hedge.participants.len() >= 2 {
                        let w_star =
                            crate::hyperedge::star_weight(hedge.weight, hedge.participants.len());
                        if w_star > 0.0 {
                            for p in hedge.participants.iter() {
                                if p.entity != u_eid {
                                    if let Some(&target_idx) = inner.id_map.get(&p.entity) {
                                        if target_idx < n
                                            && !deleted_nodes.contains(target_idx)
                                            && inner.entity_at(target_idx).is_some()
                                        {
                                            sum += w_star;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    sum
}

pub(crate) fn compute_out_weight_sums(
    inner: &GraphInner,
    n: usize,
    deleted_nodes: &DeletedView,
    ctx: &mut PprContext,
) {
    if ctx.out_weight_sums.len() < n {
        ctx.out_weight_sums.resize(n, 0.0);
    }
    if deleted_nodes.is_empty() && inner.hyperedges.is_empty() && inner.out_weight_sums.len() >= n {
        if let (Some(dst), Some(src)) = (
            ctx.out_weight_sums.get_mut(..n),
            inner.out_weight_sums.get(..n),
        ) {
            dst.copy_from_slice(src);
        }
    } else {
        for i in 0..n {
            if deleted_nodes.contains(i) || inner.entity_at(i).is_none() {
                if let Some(slot) = ctx.out_weight_sums.get_mut(i) {
                    *slot = 0.0;
                }
            } else if let Some(slot) = ctx.out_weight_sums.get_mut(i) {
                *slot = compute_node_out_weight_sum(inner, i, n, deleted_nodes);
            }
        }
    }
    for i in 0..n {
        if deleted_nodes.contains(i) || inner.entity_at(i).is_none() {
            if let Some(slot) = ctx.out_weight_sums.get_mut(i) {
                *slot = 0.0;
            }
        }
    }
}

pub(crate) fn distribute_dense_ranks(
    inner: &GraphInner,
    n: usize,
    damping: f32,
    deleted_nodes: &DeletedView,
    ctx: &mut PprContext,
) {
    let offsets = &inner.offsets;
    let targets = &inner.targets;
    let weights = &inner.weights;

    for i in 0..n {
        if deleted_nodes.contains(i) {
            if let Some(slot) = ctx.next_ranks.get_mut(i) {
                *slot = 0.0;
            }
            continue;
        }
        let sum_w = ctx.out_weight_sums.get(i).copied().unwrap_or(0.0);
        let r_i = ctx.ranks.get(i).copied().unwrap_or(0.0);
        if sum_w > 0.0 && r_i > 0.0 {
            let share = damping * r_i / sum_w;
            let start = if i < offsets.len() - 1 {
                offsets.get(i).copied().unwrap_or(0)
            } else {
                0
            };
            let end = if i < offsets.len() - 1 {
                offsets.get(i + 1).copied().unwrap_or(0)
            } else {
                0
            };

            for edge_idx in start..end {
                if let (Some(&target), Some(&weight)) =
                    (targets.get(edge_idx), weights.get(edge_idx))
                {
                    if !deleted_nodes.contains(target)
                        && inner.entity_at(target).is_some()
                        && weight > 0.0
                    {
                        if let Some(slot) = ctx.next_ranks.get_mut(target) {
                            *slot += share * weight;
                        }
                    }
                }
            }

            if let Some(pending) = inner.pending_edges.get(&i) {
                for edge in pending {
                    let target = edge.target;
                    if !deleted_nodes.contains(target)
                        && inner.entity_at(target).is_some()
                        && edge.weight > 0.0
                    {
                        if let Some(slot) = ctx.next_ranks.get_mut(target) {
                            *slot += share * edge.weight;
                        }
                    }
                }
            }

            if let Some(&u_eid) = inner.reverse_map.get(i) {
                if let Some(hedge_ids) = inner.hyperedge_index.get(&u_eid) {
                    for hid in hedge_ids {
                        if let Some(hedge) = inner.hyperedges.get(hid) {
                            if hedge.tx_valid_to.is_none() && hedge.participants.len() >= 2 {
                                let w_star = crate::hyperedge::star_weight(
                                    hedge.weight,
                                    hedge.participants.len(),
                                );
                                if w_star > 0.0 {
                                    for p in hedge.participants.iter() {
                                        if p.entity != u_eid {
                                            if let Some(&target_idx) = inner.id_map.get(&p.entity) {
                                                if target_idx < n
                                                    && !deleted_nodes.contains(target_idx)
                                                    && inner.entity_at(target_idx).is_some()
                                                {
                                                    if let Some(slot) =
                                                        ctx.next_ranks.get_mut(target_idx)
                                                    {
                                                        *slot += share * w_star;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn run_dense_power_iteration(
    inner: &GraphInner,
    n: usize,
    config: &PprConfig,
    deleted_nodes: &DeletedView,
    ctx: &mut PprContext,
) {
    let damping = if config.damping_factor.is_nan()
        || config.damping_factor <= 0.0
        || config.damping_factor >= 1.0
    {
        0.85
    } else {
        config.damping_factor
    };
    let max_iters = config.max_iterations.min(1000);
    let epsilon = if config.convergence_epsilon.is_nan() || config.convergence_epsilon <= 0.0 {
        1e-6
    } else {
        config.convergence_epsilon
    };

    let seed_count = ctx.valid_seeds.len() as f32;
    let restart_prob = 1.0 / seed_count;

    let mut last_diff = 0.0f32;
    let mut converged = false;

    for _iter in 0..max_iters {
        if let Some(slice) = ctx.next_ranks.get_mut(..n) {
            slice.fill(0.0);
        }

        let mut dangling_sum = 0.0f32;
        for i in 0..n {
            if !deleted_nodes.contains(i)
                && inner.entity_at(i).is_none()
                && ctx.out_weight_sums.get(i).copied() == Some(0.0)
            {
                if let Some(&r) = ctx.ranks.get(i) {
                    dangling_sum += r;
                }
            }
        }

        let teleport_factor = (1.0 - damping) + damping * dangling_sum;
        for &seed_idx in &ctx.valid_seeds {
            if let Some(slot) = ctx.next_ranks.get_mut(seed_idx) {
                *slot += teleport_factor * restart_prob;
            }
        }

        distribute_dense_ranks(inner, n, damping, deleted_nodes, ctx);

        let diff: f32 = match (ctx.ranks.get(..n), ctx.next_ranks.get(..n)) {
            (Some(r1), Some(r2)) => r1.iter().zip(r2.iter()).map(|(a, b)| (a - b).abs()).sum(),
            _ => 0.0,
        };

        if let (Some(dst), Some(src)) = (ctx.ranks.get_mut(..n), ctx.next_ranks.get(..n)) {
            dst.copy_from_slice(src);
        }
        last_diff = diff;

        if diff < epsilon {
            converged = true;
            break;
        }
    }

    if !converged && config.warn_on_non_convergence {
        tracing::warn!(
            max_iterations = max_iters,
            last_diff = last_diff,
            convergence_epsilon = epsilon,
            "Personalized PageRank power iteration reached maximum iterations without reaching convergence; returning best-effort rank allocation"
        );
    }
}

fn extract_dense_results(
    inner: &GraphInner,
    n: usize,
    deleted_nodes: &DeletedView,
    ctx: &mut PprContext,
) -> Vec<(EntityId, f32)> {
    let sum: f32 = match ctx.ranks.get(..n) {
        Some(slice) => slice
            .iter()
            .enumerate()
            .filter(|(idx, _)| !deleted_nodes.contains(*idx))
            .map(|(_, &r)| r)
            .sum(),
        None => 0.0,
    };

    if sum > 0.0 {
        let norm_denom = sum.max(f32::EPSILON);
        if let Some(slice) = ctx.ranks.get_mut(..n) {
            for (idx, r) in slice.iter_mut().enumerate() {
                if !deleted_nodes.contains(idx) {
                    *r /= norm_denom;
                } else {
                    *r = 0.0;
                }
            }
        }
    }

    let mut results = Vec::new();
    if let Some(slice) = ctx.ranks.get(..n) {
        for (idx, &rank) in slice.iter().enumerate() {
            if !deleted_nodes.contains(idx) && rank > 0.0 && inner.entity_at(idx).is_some() {
                if let Some(&id) = inner.reverse_map.get(idx) {
                    results.push((id, rank));
                }
            }
        }
    }

    results.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    results
}

pub(crate) fn compute_ppr_dense(
    inner: &GraphInner,
    seed_nodes: &[EntityId],
    config: &PprConfig,
    deleted_nodes: &DeletedView,
    ctx: &mut PprContext,
) -> Vec<(EntityId, f32)> {
    let n = inner.reverse_map.len();
    if n == 0 || seed_nodes.is_empty() {
        return Vec::new();
    }

    ctx.prepare(n);

    if !collect_valid_seeds(inner, seed_nodes, deleted_nodes, ctx) {
        return Vec::new();
    }

    let seed_count = ctx.valid_seeds.len() as f32;
    let restart_prob = 1.0 / seed_count;
    for &seed_idx in &ctx.valid_seeds {
        if let Some(slot) = ctx.ranks.get_mut(seed_idx) {
            *slot = restart_prob;
        }
    }

    compute_out_weight_sums(inner, n, deleted_nodes, ctx);
    run_dense_power_iteration(inner, n, config, deleted_nodes, ctx);
    extract_dense_results(inner, n, deleted_nodes, ctx)
}
