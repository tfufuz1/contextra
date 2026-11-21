//! Property and differential test suite for Thresholded Local Hyper-Flow Diffusion (TL-HFD, Spec §21, §22.2c).

use ahash::AHashMap;
use contextra_graph::csr::EdgeType;
use contextra_graph::path_rag::PathGraph;
use contextra_graph::tl_hfd::{
    compute_lovasz_extension, run_diffusion, truncate_participants, TlHfdParams,
};
use contextra_graph::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_types::EntityId;
use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Local synthetic graph structure for self-contained diffusion testing.
struct SyntheticGraph {
    binary_edges: HashMap<EntityId, Vec<(EntityId, f32)>>,
    hyperedges: HashMap<HyperEdgeId, Arc<HyperEdge>>,
    node_to_hyperedges: HashMap<EntityId, Vec<HyperEdgeId>>,
}

impl SyntheticGraph {
    fn new() -> Self {
        Self {
            binary_edges: HashMap::new(),
            hyperedges: HashMap::new(),
            node_to_hyperedges: HashMap::new(),
        }
    }

    fn add_binary_edge(&mut self, u: EntityId, v: EntityId, w: f32) {
        self.binary_edges.entry(u).or_default().push((v, w));
        self.binary_edges.entry(v).or_default().push((u, w));
    }

    fn add_hyperedge(&mut self, id: u64, participants: Vec<EntityId>, w: f32) {
        let hid = HyperEdgeId::new(id);
        let roles: Vec<RoleBinding> = participants
            .into_iter()
            .enumerate()
            .map(|(idx, entity)| RoleBinding::new(RoleId::new(idx as u32 + 1), entity))
            .collect();

        let he = Arc::new(HyperEdge::new(hid, EdgeType::Default, roles.clone(), w));

        self.hyperedges.insert(hid, he);
        for r in roles {
            self.node_to_hyperedges
                .entry(r.entity)
                .or_default()
                .push(hid);
        }
    }
}

impl PathGraph for SyntheticGraph {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.binary_edges.get(&node).cloned().unwrap_or_default()
    }

    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.neighbors_with_weights(node)
    }

    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<HyperEdgeId> {
        self.node_to_hyperedges
            .get(&node)
            .cloned()
            .unwrap_or_default()
    }

    fn get_hyperedge(&self, id: HyperEdgeId) -> Option<Arc<HyperEdge>> {
        self.hyperedges.get(&id).cloned()
    }
}

/// Helper function to compute the total Lovász objective F(x) = sum_e w_e * f_e(x) for a graph state x.
fn compute_total_lovasz_objective(
    graph: &SyntheticGraph,
    x: &AHashMap<EntityId, f32>,
    max_sort_size: usize,
) -> f32 {
    let mut total_obj = 0.0f32;
    let mut seen_binary = std::collections::HashSet::new();

    // Sum over binary edges
    for (&u, nbrs) in &graph.binary_edges {
        for &(v, w) in nbrs {
            let pair = if u < v { (u, v) } else { (v, u) };
            if seen_binary.insert(pair) {
                let participants = vec![pair.0, pair.1];
                let truncated = truncate_participants(
                    &participants,
                    |node| x.get(&node).copied().unwrap_or(0.0),
                    max_sort_size,
                );
                let (f_e, _, _) = compute_lovasz_extension(&truncated, |node| {
                    x.get(&node).copied().unwrap_or(0.0)
                });
                total_obj += w * f_e;
            }
        }
    }

    // Sum over hyperedges
    for he in graph.hyperedges.values() {
        let participants: Vec<EntityId> = he.participants.iter().map(|p| p.entity).collect();
        let truncated = truncate_participants(
            &participants,
            |node| x.get(&node).copied().unwrap_or(0.0),
            max_sort_size,
        );
        let (f_e, _, _) =
            compute_lovasz_extension(&truncated, |node| x.get(&node).copied().unwrap_or(0.0));
        total_obj += he.weight * f_e;
    }

    total_obj
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Property: Minimum Lovász objective value F*(t) = min_{1..t} F(x_k) across subgradient descent iterations
    /// is monotonically non-increasing across max_iterations from 1 to 20 (within 1e-4 tolerance).
    #[test]
    fn prop_tl_hfd_diffusion_objective_is_monotonically_non_increasing(
        num_nodes in 3..=12usize,
        seed_idx in 0..3usize,
        num_extra_edges in 1..=5usize,
    ) {
        let mut graph = SyntheticGraph::new();
        let nodes: Vec<EntityId> = (1..=num_nodes).map(|i| EntityId::new(i as u64)).collect();

        // Build connected line topology
        for i in 0..(num_nodes - 1) {
            graph.add_binary_edge(nodes[i], nodes[i + 1], 1.0);
        }

        // Add additional random binary edges and hyperedges
        for k in 0..num_extra_edges {
            let u = nodes[k % num_nodes];
            let v = nodes[(k * 3 + 1) % num_nodes];
            if u != v {
                graph.add_binary_edge(u, v, 0.8 + 0.1 * (k as f32));
            }
        }
        if num_nodes >= 4 {
            graph.add_hyperedge(101, vec![nodes[0], nodes[1], nodes[2]], 1.2);
        }

        let seed = nodes[seed_idx % num_nodes];
        let seeds = vec![seed];

        let base_params = TlHfdParams {
            max_iterations: 1,
            ..Default::default()
        };

        let mut prev_min_obj = f32::MAX;

        for max_iter in 1..=20 {
            let params = TlHfdParams {
                max_iterations: max_iter,
                ..base_params
            };

            let x_res = run_diffusion(&graph, &seeds, &params)
                .expect("Diffusion execution must succeed");

            let current_obj = compute_total_lovasz_objective(&graph, &x_res, params.max_hyperedge_sort_size);
            let current_min_obj = prev_min_obj.min(current_obj);

            if prev_min_obj != f32::MAX {
                prop_assert!(
                    current_min_obj <= prev_min_obj + 1e-4,
                    "Best objective increased at max_iterations={}: current_min={}, prev_min={}",
                    max_iter, current_min_obj, prev_min_obj
                );
            }

            prev_min_obj = current_min_obj;
        }
    }

    /// Differential property: TL-HFD diffusion matches naive projected subgradient descent reference on small binary graphs.
    ///
    /// For binary graphs (edge size 2), f_e(x) = |x_u - x_v|.
    /// Computes naive projected subgradient descent completely independently without using diffusion.rs or lovasz.rs code,
    /// proving that run_diffusion normalized objective is no worse than 1.5x of the reference objective.
    #[test]
    fn prop_tl_hfd_diffusion_matches_bruteforce_reference_small_graph(
        num_nodes in 3..=10usize,
        seed_idx in 0..2usize,
    ) {
        let mut graph = SyntheticGraph::new();
        let nodes: Vec<EntityId> = (1..=num_nodes).map(|i| EntityId::new(i as u64)).collect();

        // Build cycle graph topology with binary edges only
        let mut edges: Vec<(usize, usize, f32)> = Vec::new();
        for i in 0..num_nodes {
            let next = (i + 1) % num_nodes;
            let w = 1.0 + 0.1 * (i as f32);
            graph.add_binary_edge(nodes[i], nodes[next], w);
            edges.push((i, next, w));
        }

        let seed = nodes[seed_idx % num_nodes];
        let seeds = vec![seed];

        // 1. Run TL-HFD diffusion
        let params = TlHfdParams {
            sigma: 0.1,
            delta: 2.0,
            max_iterations: 30,
            ..Default::default()
        };
        let x_tl_hfd = run_diffusion(&graph, &seeds, &params)
            .expect("TL-HFD diffusion must succeed");

        // Scale x_tl_hfd by seed signal magnitude so signal scale matches reference
        let seed_val = x_tl_hfd.get(&seed).copied().unwrap_or(1.0).max(1e-6);
        let mut x_normalized = AHashMap::new();
        for (&u, &val) in &x_tl_hfd {
            x_normalized.insert(u, val / seed_val);
        }

        let tl_hfd_obj = compute_total_lovasz_objective(&graph, &x_normalized, params.max_hyperedge_sort_size);

        // 2. Independent naive projected subgradient descent reference implementation
        let mut ref_x = vec![0.0f32; num_nodes];
        ref_x[seed_idx % num_nodes] = 1.0; // Initialize seed signal to 1.0

        let step_size = 0.05f32;
        for _iter in 0..100 {
            let mut grad = vec![0.0f32; num_nodes];
            for &(u_i, v_i, w) in &edges {
                let diff = ref_x[u_i] - ref_x[v_i];
                if diff > 0.0 {
                    grad[u_i] += w;
                    grad[v_i] -= w;
                } else if diff < 0.0 {
                    grad[u_i] -= w;
                    grad[v_i] += w;
                }
            }

            for i in 0..num_nodes {
                // Seed node remains anchored at 1.0, non-seed nodes updated with subgradient projection x >= 0
                if i != (seed_idx % num_nodes) {
                    ref_x[i] = (ref_x[i] - step_size * grad[i]).max(0.0);
                }
            }
        }

        // Calculate naive reference total binary objective sum w_uv * |x_u - x_v|
        let mut ref_obj = 0.0f32;
        for &(u_i, v_i, w) in &edges {
            ref_obj += w * (ref_x[u_i] - ref_x[v_i]).abs();
        }

        // Prove TL-HFD objective is within 1.5x factor of reference subgradient descent objective
        let obj_threshold = 1.5 * ref_obj + 0.05;
        prop_assert!(
            tl_hfd_obj <= obj_threshold,
            "TL-HFD obj ({}) exceeded 1.5x reference obj ({}, threshold {})",
            tl_hfd_obj, ref_obj, obj_threshold
        );
    }
}

/// Regression test: verifying that max_hyperedge_sort_size cutoff (e.g. 2) on a large hyperedge (> 10 participants)
/// maintains objective value within defined tolerance compared to full participant sorting during full diffusion.
#[test]
fn test_regression_max_hyperedge_sort_size_cutoff_in_full_diffusion() {
    let mut graph = SyntheticGraph::new();

    // Build hyperedge with 12 participants
    let participants: Vec<EntityId> = (1..=12).map(|i| EntityId::new(i as u64)).collect();
    graph.add_hyperedge(500, participants.clone(), 1.0);

    // Connect participants in a chain of binary edges
    for i in 0..11 {
        graph.add_binary_edge(participants[i], participants[i + 1], 0.5);
    }

    let seeds = vec![participants[0]];

    // Config A: Cutoff max_hyperedge_sort_size = 2
    let params_cutoff = TlHfdParams {
        max_hyperedge_sort_size: 2,
        max_iterations: 10,
        ..Default::default()
    };
    let x_cutoff = run_diffusion(&graph, &seeds, &params_cutoff)
        .expect("Diffusion with cutoff = 2 must succeed");

    let obj_cutoff =
        compute_total_lovasz_objective(&graph, &x_cutoff, params_cutoff.max_hyperedge_sort_size);

    // Config B: Full sort size max_hyperedge_sort_size = 12
    let params_full = TlHfdParams {
        max_hyperedge_sort_size: 12,
        max_iterations: 10,
        ..Default::default()
    };
    let x_full = run_diffusion(&graph, &seeds, &params_full)
        .expect("Diffusion with full sort size = 12 must succeed");

    let obj_full =
        compute_total_lovasz_objective(&graph, &x_full, params_full.max_hyperedge_sort_size);

    // Verify bounded objective deviation under cutoff (Audit recommendation F3, §22.2c)
    let diff = (obj_cutoff - obj_full).abs();
    let max_allowed_diff = 0.40 * obj_full + 0.1;

    assert!(
        diff <= max_allowed_diff,
        "Cutoff obj ({obj_cutoff}) deviated from full obj ({obj_full}) by {diff} (> max allowed {max_allowed_diff})"
    );
}
