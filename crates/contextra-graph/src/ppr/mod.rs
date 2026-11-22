pub mod cost;
pub mod dense;
pub mod forward_push;
pub mod shadow_hook;
pub(crate) mod snapshot;
pub mod types;

use self::cost::{estimate_ppr_cost, GraphStats};
use self::dense::compute_ppr_dense;
use self::forward_push::forward_push_ppr;
pub use self::types::{DeletedView, PprContext};
use crate::csr::GraphInner;
use contextra_types::{EntityId, PprAlgorithm, PprConfig};

pub(crate) fn compute_ppr(
    inner: &GraphInner,
    seed_nodes: &[EntityId],
    config: &PprConfig,
    deleted_nodes: &DeletedView,
) -> Vec<(EntityId, f32)> {
    let mut ctx = PprContext::new();
    compute_ppr_with_context(inner, seed_nodes, config, deleted_nodes, &mut ctx)
}

#[cfg(feature = "apprh-diffusion")]
pub fn compute_ppr_with_apprh_selector(
    inner: &GraphInner,
    seed_nodes: &[EntityId],
    config: &PprConfig,
    deleted_nodes: &DeletedView,
    ctx: &mut PprContext,
    selector: &crate::apprh::ApprhSelector,
    apprh_params: &crate::apprh::ApprhParams,
) -> Vec<(EntityId, f32)> {
    let ppr_params = crate::path_rag::PprParams {
        alpha: if config.damping_factor.is_nan()
            || config.damping_factor <= 0.0
            || config.damping_factor >= 1.0
        {
            0.15
        } else {
            1.0 - config.damping_factor
        },
        epsilon: if config.convergence_epsilon.is_nan() || config.convergence_epsilon <= 0.0 {
            1e-6
        } else {
            config.convergence_epsilon
        },
        hyperedge_decay: apprh_params.hyperedge_decay_factor,
    };

    match selector.evaluate_and_run(inner, seed_nodes, &ppr_params, apprh_params) {
        Ok(results) => results,
        Err(err) => {
            tracing::warn!(
                error = %err,
                "APPRH selector dispatch failed; falling back to standard PPR"
            );
            compute_ppr_with_context(inner, seed_nodes, config, deleted_nodes, ctx)
        }
    }
}

pub(crate) fn select_auto_ppr_algorithm(
    inner: &GraphInner,
    seed_count: usize,
    config: &PprConfig,
) -> PprAlgorithm {
    if seed_count == 0 {
        return PprAlgorithm::ForwardPush;
    }

    let node_count = inner.reverse_map.len();
    let edge_count = inner.targets.len() + inner.pending_edge_count;
    let stats = GraphStats::new(node_count, edge_count);

    let (_estimate, choice) = estimate_ppr_cost(Some(&stats), seed_count, config);
    choice
}

fn execute_shadow_mode(
    inner: &GraphInner,
    seed_nodes: &[EntityId],
    config: &PprConfig,
    deleted_nodes: &DeletedView,
    ctx: &mut PprContext,
) -> Vec<(EntityId, f32)> {
    let dense_results = compute_ppr_dense(inner, seed_nodes, config, deleted_nodes, ctx);
    let fp_results = forward_push_ppr(inner, seed_nodes, config, deleted_nodes, ctx);

    let node_count = inner.reverse_map.len();
    let edge_count = inner.targets.len() + inner.pending_edge_count;
    let stats = GraphStats::new(node_count, edge_count);
    let (est, _) = estimate_ppr_cost(Some(&stats), seed_nodes.len(), config);

    let cal = cost::PprCostCalibrator::new();
    cal.record(cost::PprCostSample {
        algorithm: PprAlgorithm::ForwardPush,
        estimated_cost: est.forward_push,
        measured_edge_accesses: (edge_count + node_count) as u64,
    });
    cal.record(cost::PprCostSample {
        algorithm: PprAlgorithm::DensePowerIteration,
        estimated_cost: est.power_iteration,
        measured_edge_accesses: (config.max_iterations as usize * (edge_count + node_count)) as u64,
    });

    let max_diff = dense_results
        .iter()
        .zip(fp_results.iter())
        .map(|((id1, s1), (id2, s2))| if id1 == id2 { (s1 - s2).abs() } else { 1.0 })
        .fold(0.0f32, f32::max);

    let len_diff = (dense_results.len() as isize - fp_results.len() as isize).abs();

    if max_diff > config.convergence_epsilon * 10.0 || len_diff > 0 {
        tracing::warn!(
            max_diff = max_diff,
            dense_count = dense_results.len(),
            forward_push_count = fp_results.len(),
            seed_count = seed_nodes.len(),
            "PPR Shadow Mode discrepancy detected between Dense Power Iteration and Forward Push"
        );
    }

    dense_results
}

fn execute_tl_hfd(
    inner: &GraphInner,
    seed_nodes: &[EntityId],
    config: &PprConfig,
    deleted_nodes: &DeletedView,
    ctx: &mut PprContext,
    params: &contextra_types::TlHfdParams,
) -> Vec<(EntityId, f32)> {
    let g_params = crate::tl_hfd::TlHfdParams {
        sigma: params.sigma,
        delta: params.delta,
        gamma: params.gamma,
        max_iterations: params.max_iterations,
        max_top_k_expansion: params.max_top_k_expansion,
        max_hyperedge_sort_size: params.max_hyperedge_sort_size,
    };
    match crate::tl_hfd::tl_hfd_local(inner, seed_nodes, &g_params) {
        Ok(map) => {
            let sum: f32 = map.values().sum();
            let mut vec: Vec<(EntityId, f32)> = if sum > 0.0 {
                map.into_iter().map(|(id, val)| (id, val / sum)).collect()
            } else {
                map.into_iter().collect()
            };
            vec.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            });
            vec
        }
        Err(err) => {
            tracing::warn!(
                error = %err,
                "TL-HFD execution failed; falling back to dense power iteration"
            );
            compute_ppr_dense(inner, seed_nodes, config, deleted_nodes, ctx)
        }
    }
}

pub(crate) fn compute_ppr_with_context(
    inner: &GraphInner,
    seed_nodes: &[EntityId],
    config: &PprConfig,
    deleted_nodes: &DeletedView,
    ctx: &mut PprContext,
) -> Vec<(EntityId, f32)> {
    let chosen_algorithm = match config.algorithm {
        PprAlgorithm::Auto => select_auto_ppr_algorithm(inner, seed_nodes.len(), config),
        other => other,
    };

    match chosen_algorithm {
        PprAlgorithm::ForwardPush => {
            forward_push_ppr(inner, seed_nodes, config, deleted_nodes, ctx)
        }
        PprAlgorithm::DensePowerIteration => {
            compute_ppr_dense(inner, seed_nodes, config, deleted_nodes, ctx)
        }
        PprAlgorithm::ShadowMode => {
            execute_shadow_mode(inner, seed_nodes, config, deleted_nodes, ctx)
        }
        PprAlgorithm::TlHfd(ref params) => {
            execute_tl_hfd(inner, seed_nodes, config, deleted_nodes, ctx, params)
        }
        PprAlgorithm::ShadowModeTlHfd(ref params) => {
            let g_params = crate::tl_hfd::TlHfdParams {
                sigma: params.sigma,
                delta: params.delta,
                gamma: params.gamma,
                max_iterations: params.max_iterations,
                max_top_k_expansion: params.max_top_k_expansion,
                max_hyperedge_sort_size: params.max_hyperedge_sort_size,
            };
            let ppr_params = crate::path_rag::PprParams {
                alpha: if config.damping_factor.is_nan()
                    || config.damping_factor <= 0.0
                    || config.damping_factor >= 1.0
                {
                    0.15
                } else {
                    1.0 - config.damping_factor
                },
                epsilon: if config.convergence_epsilon.is_nan() || config.convergence_epsilon <= 0.0
                {
                    1e-6
                } else {
                    config.convergence_epsilon
                },
                hyperedge_decay: 0.85,
            };
            let top_k = g_params.max_top_k_expansion;
            match crate::tl_hfd::shadow_compare_forward_push_vs_tl_hfd(
                inner,
                seed_nodes,
                &ppr_params,
                &g_params,
                top_k,
            ) {
                Ok(comp) => comp.forward_push_results,
                Err(err) => {
                    tracing::warn!(
                        error = %err,
                        "TL-HFD shadow mode comparison failed; falling back to forward push"
                    );
                    forward_push_ppr(inner, seed_nodes, config, deleted_nodes, ctx)
                }
            }
        }
        PprAlgorithm::Auto => match select_auto_ppr_algorithm(inner, seed_nodes.len(), config) {
            PprAlgorithm::ForwardPush => {
                forward_push_ppr(inner, seed_nodes, config, deleted_nodes, ctx)
            }
            _ => compute_ppr_dense(inner, seed_nodes, config, deleted_nodes, ctx),
        },
    }
}

impl crate::path_rag::PathGraph for GraphInner {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        let node_idx = match self.id_map.get(&node) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };
        if self.entity_at(node_idx).is_none() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();

        if node_idx < self.offsets.len() - 1 {
            let start_edge = self.offsets.get(node_idx).copied().unwrap_or(0);
            let end_edge = self.offsets.get(node_idx + 1).copied().unwrap_or(0);
            for edge_idx in start_edge..end_edge {
                if let Some(&neighbor_idx) = self.targets.get(edge_idx) {
                    if !self.tombstoned_edges.contains(&(node_idx, neighbor_idx))
                        && self.entity_at(neighbor_idx).is_some()
                    {
                        if let Some(&id) = self.reverse_map.get(neighbor_idx) {
                            if seen.insert(id) {
                                if let Some(&w) = self.weights.get(edge_idx) {
                                    result.push((id, w));
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(pending) = self.pending_edges.get(&node_idx) {
            for edge in pending {
                let neighbor_idx = edge.target;
                if !self.tombstoned_edges.contains(&(node_idx, neighbor_idx))
                    && self.entity_at(neighbor_idx).is_some()
                {
                    if let Some(&id) = self.reverse_map.get(neighbor_idx) {
                        if seen.insert(id) {
                            result.push((id, edge.weight));
                        }
                    }
                }
            }
        }

        result
    }

    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        let target_idx = match self.id_map.get(&node) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };
        if self.entity_at(target_idx).is_none() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let num_nodes = self.reverse_map.len();

        for u_idx in 0..num_nodes {
            if self.entity_at(u_idx).is_none() {
                continue;
            }
            let u_id = match self.reverse_map.get(u_idx) {
                Some(&id) => id,
                None => continue,
            };

            if u_idx < self.offsets.len() - 1 {
                let start_edge = self.offsets.get(u_idx).copied().unwrap_or(0);
                let end_edge = self.offsets.get(u_idx + 1).copied().unwrap_or(0);
                for edge_idx in start_edge..end_edge {
                    if self.targets.get(edge_idx) == Some(&target_idx)
                        && !self.tombstoned_edges.contains(&(u_idx, target_idx))
                        && seen.insert(u_id)
                    {
                        if let Some(&w) = self.weights.get(edge_idx) {
                            result.push((u_id, w));
                        }
                    }
                }
            }

            if let Some(pending) = self.pending_edges.get(&u_idx) {
                for edge in pending {
                    if edge.target == target_idx
                        && !self.tombstoned_edges.contains(&(u_idx, target_idx))
                        && seen.insert(u_id)
                    {
                        result.push((u_id, edge.weight));
                    }
                }
            }
        }

        result
    }

    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<crate::hyperedge::HyperEdgeId> {
        self.hyperedge_index
            .get(&node)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    fn get_hyperedge(
        &self,
        id: crate::hyperedge::HyperEdgeId,
    ) -> Option<std::sync::Arc<crate::hyperedge::HyperEdge>> {
        self.hyperedges.get(&id).cloned()
    }
}

#[cfg(test)]
mod tests;
