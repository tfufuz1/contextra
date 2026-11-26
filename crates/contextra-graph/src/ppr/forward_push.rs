// ANCHOR[contextra-graph:SRC-PPR-FORWARD-PUSH-001]
//! Forward-Push-Algorithmus (Andersen et al.) für personalisierten PageRank.
//!
//! # Zweck
//! Ermöglicht hochgradig lokalisierte Diffusionsberechnungen mit sublinearer Laufzeitkomplexität.

use super::dense::{collect_valid_seeds, compute_out_weight_sums};
use super::types::{DeletedView, PprContext};
use crate::csr::GraphInner;
use contextra_types::{EntityId, PprConfig};
use std::collections::BTreeMap;

pub(crate) fn push_mass_to_neighbors(
    inner: &GraphInner,
    u: usize,
    push_mass: f32,
    w_u: f32,
    deleted_nodes: &DeletedView,
    r: &mut BTreeMap<usize, f32>,
    n: usize,
) {
    let offsets = &inner.offsets;
    let targets = &inner.targets;
    let weights = &inner.weights;

    let start = if u < offsets.len() - 1 {
        offsets.get(u).copied().unwrap_or(0)
    } else {
        0
    };
    let end = if u < offsets.len() - 1 {
        offsets.get(u + 1).copied().unwrap_or(0)
    } else {
        0
    };

    for edge_idx in start..end {
        if let (Some(&target), Some(&weight)) = (targets.get(edge_idx), weights.get(edge_idx)) {
            if !deleted_nodes.contains(target) && inner.entity_at(target).is_some() && weight > 0.0
            {
                let share = push_mass * (weight / w_u);
                *r.entry(target).or_insert(0.0) += share;
            }
        }
    }

    if let Some(pending) = inner.pending_edges.get(&u) {
        for edge in pending {
            let target = edge.target;
            if !deleted_nodes.contains(target)
                && inner.entity_at(target).is_some()
                && edge.weight > 0.0
            {
                let share = push_mass * (edge.weight / w_u);
                *r.entry(target).or_insert(0.0) += share;
            }
        }
    }

    if let Some(&u_eid) = inner.reverse_map.get(u) {
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
                                            let share = push_mass * (w_star / w_u);
                                            *r.entry(target_idx).or_insert(0.0) += share;
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

fn select_best_push_candidate(
    r: &BTreeMap<usize, f32>,
    out_weight_sums: &[f32],
    epsilon: f32,
) -> Option<(usize, f32, f32)> {
    let mut best_candidate: Option<(usize, f32, f32, f32)> = None;

    for (&u, &res_u) in r {
        if res_u <= 0.0 {
            continue;
        }
        let w_u = out_weight_sums.get(u).copied().unwrap_or(0.0);
        let ratio = if w_u > 0.0 { res_u / w_u } else { res_u };

        if ratio > epsilon {
            match best_candidate {
                Some((_, _, _, best_ratio)) => {
                    if ratio > best_ratio + 1e-7 {
                        best_candidate = Some((u, res_u, w_u, ratio));
                    }
                }
                None => {
                    best_candidate = Some((u, res_u, w_u, ratio));
                }
            }
        }
    }

    best_candidate.map(|(u, res_u, w_u, _)| (u, res_u, w_u))
}

struct ForwardPushLoopArgs<'a> {
    inner: &'a GraphInner,
    n: usize,
    alpha: f32,
    epsilon: f32,
    max_push_steps: usize,
    deleted_nodes: &'a DeletedView,
    ctx: &'a PprContext,
}

fn run_forward_push_loop(
    args: ForwardPushLoopArgs<'_>,
    p: &mut BTreeMap<usize, f32>,
    r: &mut BTreeMap<usize, f32>,
) -> bool {
    let seed_count = args.ctx.valid_seeds.len() as f32;
    let mut push_steps = 0;

    loop {
        if push_steps >= args.max_push_steps {
            return true;
        }

        let (u, res_u, w_u) =
            match select_best_push_candidate(r, &args.ctx.out_weight_sums, args.epsilon) {
                Some(cand) => cand,
                None => break,
            };

        push_steps += 1;
        r.remove(&u);

        *p.entry(u).or_insert(0.0) += args.alpha * res_u;

        let push_mass = (1.0 - args.alpha) * res_u;
        if push_mass <= 0.0 {
            continue;
        }

        if w_u > 0.0 {
            push_mass_to_neighbors(args.inner, u, push_mass, w_u, args.deleted_nodes, r, args.n);
        } else {
            let seed_share = push_mass / seed_count;
            for &s in &args.ctx.valid_seeds {
                *r.entry(s).or_insert(0.0) += seed_share;
            }
        }
    }

    false
}

fn extract_push_results(
    inner: &GraphInner,
    mut p: BTreeMap<usize, f32>,
    deleted_nodes: &DeletedView,
) -> Vec<(EntityId, f32)> {
    let total_sum: f32 = p.values().sum();
    if total_sum > 0.0 {
        for rank in p.values_mut() {
            *rank /= total_sum;
        }
    }

    let mut results = Vec::new();
    for (idx, rank) in p {
        if !deleted_nodes.contains(idx) && rank > 0.0 && inner.entity_at(idx).is_some() {
            if let Some(&id) = inner.reverse_map.get(idx) {
                results.push((id, rank));
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

pub(crate) fn forward_push_ppr(
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

    if !collect_valid_seeds(inner, seed_nodes, deleted_nodes, ctx) {
        return Vec::new();
    }

    compute_out_weight_sums(inner, n, deleted_nodes, ctx);

    let alpha = if config.damping_factor.is_nan()
        || config.damping_factor <= 0.0
        || config.damping_factor >= 1.0
    {
        0.15f32
    } else {
        1.0f32 - config.damping_factor
    };

    let epsilon = if config.convergence_epsilon.is_nan() || config.convergence_epsilon <= 0.0 {
        1e-6
    } else {
        config.convergence_epsilon
    };

    let max_push_steps = (config.max_iterations.min(1000) as usize) * 10;

    let mut p: BTreeMap<usize, f32> = BTreeMap::new();
    let mut r: BTreeMap<usize, f32> = BTreeMap::new();

    let seed_count = ctx.valid_seeds.len() as f32;
    let initial_r = 1.0 / seed_count;
    for &s in &ctx.valid_seeds {
        r.insert(s, initial_r);
    }

    let loop_args = ForwardPushLoopArgs {
        inner,
        n,
        alpha,
        epsilon,
        max_push_steps,
        deleted_nodes,
        ctx,
    };

    let max_steps_reached = run_forward_push_loop(loop_args, &mut p, &mut r);

    if max_steps_reached && config.warn_on_non_convergence {
        tracing::warn!(
            max_iterations = config.max_iterations,
            convergence_epsilon = epsilon,
            "Personalized PageRank power iteration reached maximum iterations without reaching convergence; returning best-effort rank allocation"
        );
    }

    extract_push_results(inner, p, deleted_nodes)
}
