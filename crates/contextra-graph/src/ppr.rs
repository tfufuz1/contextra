//! Personalized PageRank (PPR) power iteration implementation for `CsrGraph`.

// FILE-CONTEXT
// STAND: 2026-08-30T18:53:58Z (SESSION: b1234567)
// ZWECK: Personalized PageRank Power Iteration über CSR Graph
// INVARIANTEN: inner MUSS vor Aufruf kompaktiert sein; bitidentischer Determinismus.
// HOTSPOTS: L30-L90 (Power Iteration Matrix-Vector Vector Multiplication)
// SIEHE AUCH: crates/contextra-graph/src/csr.rs

pub mod shadow_hook;
pub(crate) mod snapshot;

use crate::csr::GraphInner;
use contextra_types::{EntityId, PprAlgorithm, PprConfig};
use std::collections::{BTreeMap, HashSet};

/// Information view over graph tombstones/deleted node indices.
///
/// MUST NOT implement `Default`. Constructible only via explicit paths that load tombstone state
/// (e.g. `CsrGraph::deleted_view(&self).await`).
///
/// ```compile_fail
/// use contextra_graph::DeletedView;
/// let view = DeletedView::default(); // DeletedView does NOT implement Default
/// ```
#[derive(Debug, Clone)]
pub struct DeletedView {
    deleted_nodes: HashSet<usize>,
}

impl DeletedView {
    /// Creates a `DeletedView` from a known set of deleted node indices.
    pub(crate) fn from_nodes(deleted_nodes: HashSet<usize>) -> Self {
        Self { deleted_nodes }
    }

    /// Creates an empty `DeletedView` when no nodes are deleted or for testing when explicitly intended.
    #[allow(dead_code)]
    pub(crate) fn empty() -> Self {
        Self {
            deleted_nodes: HashSet::new(),
        }
    }

    /// Checks if internal node index `idx` is marked as deleted.
    #[inline]
    pub fn contains(&self, idx: usize) -> bool {
        self.deleted_nodes.contains(&idx)
    }

    /// Returns the number of deleted node indices contained in this view.
    #[inline]
    pub fn len(&self) -> usize {
        self.deleted_nodes.len()
    }

    /// Returns `true` if this view contains no deleted nodes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.deleted_nodes.is_empty()
    }
}

/// Reusable scratch buffers for Personalized PageRank (PPR) power iteration.
///
/// Pre-allocating `PprContext` eliminates all intermediate heap allocations during repeated
/// PPR queries over the CSR graph structure.
#[derive(Debug, Clone, Default)]
pub struct PprContext {
    valid_seeds: Vec<usize>,
    out_weight_sums: Vec<f32>,
    ranks: Vec<f32>,
    next_ranks: Vec<f32>,
}

impl PprContext {
    /// Creates an empty PPR context buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets and prepares internal vectors for a graph with `n` nodes.
    fn prepare(&mut self, n: usize) {
        self.valid_seeds.clear();

        if self.out_weight_sums.len() < n {
            self.out_weight_sums.resize(n, 0.0);
            self.ranks.resize(n, 0.0);
            self.next_ranks.resize(n, 0.0);
        }
        self.out_weight_sums[..n].fill(0.0);
        self.ranks[..n].fill(0.0);
        self.next_ranks[..n].fill(0.0);
    }
}

/// Calculates Personalized PageRank (PPR) over a compacted `GraphInner` state.
pub(crate) fn compute_ppr(
    inner: &GraphInner,
    seed_nodes: &[EntityId],
    config: &PprConfig,
    deleted_nodes: &DeletedView,
) -> Vec<(EntityId, f32)> {
    let mut ctx = PprContext::new();
    compute_ppr_with_context(inner, seed_nodes, config, deleted_nodes, &mut ctx)
}

/// Calculates Personalized PageRank (PPR) over a compacted `GraphInner` state using a reusable [`PprContext`].
///
/// # Invarianten
/// - `inner` MUSS vor dem Aufruf kompaktiert sein (`inner.compact()`).
/// - Determinismus: Bitidentische Sortierung bei identischem Graph & Inputs.
/// - Zero-Hang: Bounded execution by `config.max_iterations`.
/// - Zero O(N) Allocations when reusing `ctx`.
pub(crate) fn compute_ppr_with_context(
    inner: &GraphInner,
    seed_nodes: &[EntityId],
    config: &PprConfig,
    deleted_nodes: &DeletedView,
    ctx: &mut PprContext,
) -> Vec<(EntityId, f32)> {
    let chosen_algorithm = match config.algorithm {
        PprAlgorithm::Auto => {
            if seed_nodes.len() <= 100 {
                PprAlgorithm::ForwardPush
            } else {
                PprAlgorithm::DensePowerIteration
            }
        }
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
            let dense_results = compute_ppr_dense(inner, seed_nodes, config, deleted_nodes, ctx);
            let fp_results = forward_push_ppr(inner, seed_nodes, config, deleted_nodes, ctx);

            // Compare top results and log any substantial discrepancy
            let max_diff = dense_results
                .iter()
                .zip(fp_results.iter())
                .map(
                    |((id1, s1), (id2, s2))| {
                        if id1 == id2 {
                            (s1 - s2).abs()
                        } else {
                            1.0
                        }
                    },
                )
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
        PprAlgorithm::TlHfd(params) => {
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
        PprAlgorithm::ShadowModeTlHfd(params) => {
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
        PprAlgorithm::Auto => unreachable!("Auto resolved above"),
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
            let start_edge = self.offsets[node_idx];
            let end_edge = self.offsets[node_idx + 1];
            for edge_idx in start_edge..end_edge {
                let neighbor_idx = self.targets[edge_idx];
                if !self.tombstoned_edges.contains(&(node_idx, neighbor_idx))
                    && self.entity_at(neighbor_idx).is_some()
                {
                    if let Some(&id) = self.reverse_map.get(neighbor_idx) {
                        if seen.insert(id) {
                            result.push((id, self.weights[edge_idx]));
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
                let start_edge = self.offsets[u_idx];
                let end_edge = self.offsets[u_idx + 1];
                for edge_idx in start_edge..end_edge {
                    if self.targets[edge_idx] == target_idx
                        && !self.tombstoned_edges.contains(&(u_idx, target_idx))
                        && seen.insert(u_id)
                    {
                        result.push((u_id, self.weights[edge_idx]));
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

/// Computes Personalized PageRank using Andersen-Chung-Lang Forward-Push local random walk algorithm.
///
/// Complexities: $O(1 / (\epsilon \cdot \alpha))$ time complexity, sparse memory $O(|\text{Supp}(p)| + |\text{Supp}(r)|)$.
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

    // 1. Identify valid seed internal indices
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

    if ctx.valid_seeds.is_empty() {
        return Vec::new();
    }

    // Prepare out_weight_sums in context
    if ctx.out_weight_sums.len() < n {
        ctx.out_weight_sums.resize(n, 0.0);
    }
    if deleted_nodes.is_empty() && inner.hyperedges.is_empty() && inner.out_weight_sums.len() >= n {
        ctx.out_weight_sums[..n].copy_from_slice(&inner.out_weight_sums[..n]);
    } else {
        for i in 0..n {
            if deleted_nodes.contains(i) || inner.entity_at(i).is_none() {
                ctx.out_weight_sums[i] = 0.0;
                continue;
            }

            let start = if i < inner.offsets.len() - 1 {
                inner.offsets[i]
            } else {
                0
            };
            let end = if i < inner.offsets.len() - 1 {
                inner.offsets[i + 1]
            } else {
                0
            };

            let mut sum = 0.0f32;
            for edge_idx in start..end {
                let target = inner.targets[edge_idx];
                let weight = inner.weights[edge_idx];

                if !deleted_nodes.contains(target)
                    && inner.entity_at(target).is_some()
                    && weight > 0.0
                {
                    sum += weight;
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

            ctx.out_weight_sums[i] = sum;
        }
    }
    for i in 0..n {
        if deleted_nodes.contains(i) || inner.entity_at(i).is_none() {
            ctx.out_weight_sums[i] = 0.0;
        }
    }

    // Config parameters
    let alpha = if config.damping_factor.is_nan()
        || config.damping_factor <= 0.0
        || config.damping_factor >= 1.0
    {
        0.15f32 // teleport / restart probability alpha = 1 - damping
    } else {
        1.0f32 - config.damping_factor
    };

    let epsilon = if config.convergence_epsilon.is_nan() || config.convergence_epsilon <= 0.0 {
        1e-6
    } else {
        config.convergence_epsilon
    };

    let max_push_steps = (config.max_iterations.min(1000) as usize) * 10;

    // Initialize sparse BTreeMap state for bit-identical determinism
    let mut p: BTreeMap<usize, f32> = BTreeMap::new();
    let mut r: BTreeMap<usize, f32> = BTreeMap::new();

    let seed_count = ctx.valid_seeds.len() as f32;
    let initial_r = 1.0 / seed_count;
    for &s in &ctx.valid_seeds {
        r.insert(s, initial_r);
    }

    let offsets = &inner.offsets;
    let targets = &inner.targets;
    let weights = &inner.weights;

    let mut push_steps = 0;
    let mut max_steps_reached = false;

    loop {
        if push_steps >= max_push_steps {
            max_steps_reached = true;
            break;
        }

        // Find candidate u that MAXIMIZES normalized residual ratio above epsilon (greedy max-residual push).
        // Tie-break equal ratios by smallest internal index u for 100% score symmetry on symmetric subgraphs.
        let mut best_candidate: Option<(usize, f32, f32, f32)> = None; // (u, res_u, w_u, ratio)

        for (&u, &res_u) in &r {
            if res_u <= 0.0 {
                continue;
            }
            let w_u = ctx.out_weight_sums[u];
            let ratio = if w_u > 0.0 { res_u / w_u } else { res_u };

            if ratio > epsilon {
                match best_candidate {
                    Some((_, _, _, best_ratio)) => {
                        // Maximize ratio (allow 1e-7 float epsilon for exact score symmetry)
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

        let (u, res_u, w_u) = match best_candidate {
            Some((u, res_u, w_u, _)) => (u, res_u, w_u),
            None => break, // Convergence reached: no node exceeds threshold
        };

        push_steps += 1;
        r.remove(&u);

        // Convert alpha * res_u to PageRank estimate p[u]
        *p.entry(u).or_insert(0.0) += alpha * res_u;

        let push_mass = (1.0 - alpha) * res_u;
        if push_mass <= 0.0 {
            continue;
        }

        if w_u > 0.0 {
            // Push remaining mass to neighbors
            let start = if u < offsets.len() - 1 { offsets[u] } else { 0 };
            let end = if u < offsets.len() - 1 {
                offsets[u + 1]
            } else {
                0
            };

            for edge_idx in start..end {
                let target = targets[edge_idx];
                let weight = weights[edge_idx];

                if !deleted_nodes.contains(target)
                    && inner.entity_at(target).is_some()
                    && weight > 0.0
                {
                    let share = push_mass * (weight / w_u);
                    let entry = r.entry(target).or_insert(0.0);
                    *entry += share;
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
                        let entry = r.entry(target).or_insert(0.0);
                        *entry += share;
                    }
                }
            }

            if let Some(&u_eid) = inner.reverse_map.get(u) {
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
                                                    let share = push_mass * (w_star / w_u);
                                                    let entry = r.entry(target_idx).or_insert(0.0);
                                                    *entry += share;
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
        } else {
            // Dead-end / dangling node: redistribute push_mass to seeds
            let seed_share = push_mass / seed_count;
            for &s in &ctx.valid_seeds {
                let entry = r.entry(s).or_insert(0.0);
                *entry += seed_share;
            }
        }
    }

    if max_steps_reached && config.warn_on_non_convergence {
        tracing::warn!(
            max_iterations = config.max_iterations,
            push_steps = push_steps,
            convergence_epsilon = epsilon,
            "Personalized PageRank power iteration reached maximum iterations without reaching convergence; returning best-effort rank allocation"
        );
    }

    // Re-normalize ranks so total non-deleted rank sum equals 1.0
    let total_sum: f32 = p.values().sum();
    if total_sum > 0.0 {
        for rank in p.values_mut() {
            *rank /= total_sum;
        }
    }

    // Build and sort result vector
    let mut results = Vec::new();
    for (idx, rank) in p {
        if !deleted_nodes.contains(idx) && rank > 0.0 && inner.entity_at(idx).is_some() {
            if let Some(&id) = inner.reverse_map.get(idx) {
                results.push((id, rank));
            }
        }
    }

    // Deterministic sort: score descending, tie-break by EntityId ascending
    results.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    results
}

/// Calculates Personalized PageRank (PPR) using dense power iteration.
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

    // 1. Identify valid seed internal indices (must exist, have committed entity, and not be deleted)
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

    if ctx.valid_seeds.is_empty() {
        return Vec::new();
    }

    // 2. Build restart / teleport probabilities
    let seed_count = ctx.valid_seeds.len() as f32;
    let restart_prob = 1.0 / seed_count;
    for &seed_idx in &ctx.valid_seeds {
        ctx.ranks[seed_idx] = restart_prob;
    }

    // 3. Populate outgoing weight sum per node directly from GraphInner precomputed out_weight_sums.
    if deleted_nodes.is_empty() && inner.hyperedges.is_empty() && inner.out_weight_sums.len() >= n {
        ctx.out_weight_sums[..n].copy_from_slice(&inner.out_weight_sums[..n]);
    } else {
        for i in 0..n {
            if deleted_nodes.contains(i) || inner.entity_at(i).is_none() {
                ctx.out_weight_sums[i] = 0.0;
                continue;
            }

            let start = if i < inner.offsets.len() - 1 {
                inner.offsets[i]
            } else {
                0
            };
            let end = if i < inner.offsets.len() - 1 {
                inner.offsets[i + 1]
            } else {
                0
            };

            let mut sum = 0.0f32;
            for edge_idx in start..end {
                let target = inner.targets[edge_idx];
                let weight = inner.weights[edge_idx];

                if !deleted_nodes.contains(target)
                    && inner.entity_at(target).is_some()
                    && weight > 0.0
                {
                    sum += weight;
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

            ctx.out_weight_sums[i] = sum;
        }
    }

    // Zero-out out_weight_sums for deleted or non-entity nodes
    for i in 0..n {
        if deleted_nodes.contains(i) || inner.entity_at(i).is_none() {
            ctx.out_weight_sums[i] = 0.0;
        }
    }

    // Validate config parameters defensively
    let damping = if config.damping_factor.is_nan()
        || config.damping_factor <= 0.0
        || config.damping_factor >= 1.0
    {
        0.85
    } else {
        config.damping_factor
    };

    let max_iters = config.max_iterations.min(1000); // hard ceiling
    let epsilon = if config.convergence_epsilon.is_nan() || config.convergence_epsilon <= 0.0 {
        1e-6
    } else {
        config.convergence_epsilon
    };

    // Bind CSR slices for power iteration
    let offsets = &inner.offsets;
    let targets = &inner.targets;
    let weights = &inner.weights;

    // 4. Power Iteration operating directly on CSR slices
    let mut last_diff = 0.0f32;
    let mut converged = false;

    for _iter in 0..max_iters {
        ctx.next_ranks[..n].fill(0.0);

        // Rank mass accumulated at dead-end (dangling) nodes (excluding deleted nodes)
        let mut dangling_sum = 0.0f32;
        for i in 0..n {
            if !deleted_nodes.contains(i)
                && inner.entity_at(i).is_some()
                && ctx.out_weight_sums[i] == 0.0
            {
                dangling_sum += ctx.ranks[i];
            }
        }

        // Teleport / restart contribution (including redistributed dangling rank mass)
        let teleport_factor = (1.0 - damping) + damping * dangling_sum;
        for &seed_idx in &ctx.valid_seeds {
            ctx.next_ranks[seed_idx] += teleport_factor * restart_prob;
        }

        // Rank distribution across outgoing edges directly from CSR
        for i in 0..n {
            if deleted_nodes.contains(i) {
                ctx.next_ranks[i] = 0.0; // Phantom-Node erhält keinen Rang
                continue;
            }
            let sum_w = ctx.out_weight_sums[i];
            let r_i = ctx.ranks[i];
            if sum_w > 0.0 && r_i > 0.0 {
                let share = damping * r_i / sum_w;
                let start = if i < offsets.len() - 1 { offsets[i] } else { 0 };
                let end = if i < offsets.len() - 1 {
                    offsets[i + 1]
                } else {
                    0
                };

                for edge_idx in start..end {
                    let target = targets[edge_idx];
                    let weight = weights[edge_idx];

                    if !deleted_nodes.contains(target)
                        && inner.entity_at(target).is_some()
                        && weight > 0.0
                    {
                        ctx.next_ranks[target] += share * weight;
                    }
                }

                if let Some(pending) = inner.pending_edges.get(&i) {
                    for edge in pending {
                        let target = edge.target;
                        if !deleted_nodes.contains(target)
                            && inner.entity_at(target).is_some()
                            && edge.weight > 0.0
                        {
                            ctx.next_ranks[target] += share * edge.weight;
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
                                                if let Some(&target_idx) =
                                                    inner.id_map.get(&p.entity)
                                                {
                                                    if target_idx < n
                                                        && !deleted_nodes.contains(target_idx)
                                                        && inner.entity_at(target_idx).is_some()
                                                    {
                                                        ctx.next_ranks[target_idx] +=
                                                            share * w_star;
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

        // Convergence check via L1 norm diff
        let diff: f32 = ctx.ranks[..n]
            .iter()
            .zip(ctx.next_ranks[..n].iter())
            .map(|(a, b)| (a - b).abs())
            .sum();

        ctx.ranks[..n].copy_from_slice(&ctx.next_ranks[..n]);
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

    // 5. Re-normalize ranks so total non-deleted rank sum equals 1.0
    let sum: f32 = ctx.ranks[..n]
        .iter()
        .enumerate()
        .filter(|(idx, _)| !deleted_nodes.contains(*idx))
        .map(|(_, &r)| r)
        .sum();
    if sum > 0.0 {
        let norm_denom = sum.max(f32::EPSILON);
        for (idx, r) in ctx.ranks[..n].iter_mut().enumerate() {
            if !deleted_nodes.contains(idx) {
                *r /= norm_denom;
            } else {
                *r = 0.0;
            }
        }
    }

    // 6. Build and sort result vector (excluding deleted nodes)
    let mut results = Vec::new();
    for (idx, &rank) in ctx.ranks[..n].iter().enumerate() {
        if !deleted_nodes.contains(idx) && rank > 0.0 && inner.entity_at(idx).is_some() {
            if let Some(&id) = inner.reverse_map.get(idx) {
                results.push((id, rank));
            }
        }
    }

    // Deterministic sort: score descending, tie-break by EntityId ascending
    results.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    results
}

#[cfg(test)]
mod tests;
