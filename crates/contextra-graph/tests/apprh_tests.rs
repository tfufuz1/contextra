//! Integration and unit tests for Averaging-based Personalized PageRank for Hypergraphs (APPRH).

#![cfg(feature = "apprh-diffusion")]

use contextra_graph::apprh::{
    apprh_local, shadow_compare_forward_push_vs_apprh, ApprhError, ApprhParams,
};
use contextra_graph::path_rag::{PathGraph, PprParams};
use contextra_graph::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_types::EntityId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct MockGraph {
    binary_edges: HashMap<EntityId, Vec<(EntityId, f32)>>,
    hyperedges: HashMap<HyperEdgeId, Arc<HyperEdge>>,
    node_to_hyperedges: HashMap<EntityId, Vec<HyperEdgeId>>,
    neighbor_calls: AtomicUsize,
    hyperedge_calls: AtomicUsize,
}

impl MockGraph {
    fn new() -> Self {
        Self {
            binary_edges: HashMap::new(),
            hyperedges: HashMap::new(),
            node_to_hyperedges: HashMap::new(),
            neighbor_calls: AtomicUsize::new(0),
            hyperedge_calls: AtomicUsize::new(0),
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

        let he = Arc::new(HyperEdge::new(
            hid,
            contextra_graph::csr::EdgeType::Default,
            roles.clone(),
            w,
        ));

        self.hyperedges.insert(hid, he);
        for r in roles {
            self.node_to_hyperedges
                .entry(r.entity)
                .or_default()
                .push(hid);
        }
    }
}

impl PathGraph for MockGraph {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.neighbor_calls.fetch_add(1, Ordering::Relaxed);
        self.binary_edges.get(&node).cloned().unwrap_or_default()
    }

    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.neighbors_with_weights(node)
    }

    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<HyperEdgeId> {
        self.hyperedge_calls.fetch_add(1, Ordering::Relaxed);
        self.node_to_hyperedges
            .get(&node)
            .cloned()
            .unwrap_or_default()
    }

    fn get_hyperedge(&self, id: HyperEdgeId) -> Option<Arc<HyperEdge>> {
        self.hyperedges.get(&id).cloned()
    }
}

#[test]
fn test_apprh_determinism_and_seed_permutations() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);
    let e4 = EntityId::new(4);

    g.add_binary_edge(e1, e2, 1.0);
    g.add_binary_edge(e2, e3, 0.8);
    g.add_hyperedge(100, vec![e1, e3, e4], 1.5);

    let params = ApprhParams::default();

    // 1. Bit-identical across 20 iterations
    let seeds = vec![e1, e2];
    let base_res = apprh_local(&g, &seeds, &params)?;

    for _ in 0..20 {
        let run_res = apprh_local(&g, &seeds, &params)?;
        assert_eq!(base_res.len(), run_res.len());
        for (k, v) in &base_res {
            let run_val = run_res.get(k).ok_or("key missing in run_res")?;
            assert_eq!(v.to_bits(), run_val.to_bits());
        }
    }

    // 2. Seed order permutation bit-identical
    let permuted_seeds = vec![e2, e1];
    let permuted_res = apprh_local(&g, &permuted_seeds, &params)?;
    assert_eq!(base_res.len(), permuted_res.len());
    for (k, v) in &base_res {
        let perm_val = permuted_res.get(k).ok_or("key missing in permuted_res")?;
        assert_eq!(v.to_bits(), perm_val.to_bits());
    }

    Ok(())
}

#[test]
fn test_apprh_convergence_sum_and_finiteness() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);
    let e4 = EntityId::new(4);

    g.add_binary_edge(e1, e2, 1.0);
    g.add_binary_edge(e2, e3, 1.0);
    g.add_hyperedge(1, vec![e1, e3, e4], 1.0);

    let seeds = vec![e1];
    let params = ApprhParams::default();

    let res = apprh_local(&g, &seeds, &params)?;
    assert!(!res.is_empty());

    for (&node, &score) in &res {
        assert!(
            score.is_finite() && score > 0.0,
            "Node {:?} score {} must be positive and finite",
            node,
            score
        );
    }

    let sum: f32 = res.values().sum();
    assert!(
        sum > 0.0 && sum.is_finite(),
        "Score sum {} must be positive and finite",
        sum
    );

    Ok(())
}

#[test]
fn test_p24_locality_bounded_visited_set() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    // 10,000 node chain graph
    for i in 1..10000 {
        g.add_binary_edge(EntityId::new(i), EntityId::new(i + 1), 1.0);
    }

    let seeds = vec![EntityId::new(1)];
    let params = ApprhParams {
        max_iterations: 10,
        ..Default::default()
    };

    let res = apprh_local(&g, &seeds, &params)?;

    // Bounded search space scale with max_iterations budget, independent of 10,000 nodes
    assert!(
        res.len() <= 20,
        "Visited nodes count {} exceeded P24 locality bound 20",
        res.len()
    );

    Ok(())
}

#[test]
fn test_apprh_parameter_validation() {
    let g = MockGraph::new();
    let seeds = vec![EntityId::new(1)];

    let p_empty_seeds: Vec<EntityId> = vec![];
    assert!(matches!(
        apprh_local(&g, &p_empty_seeds, &ApprhParams::default()),
        Err(ApprhError::EmptySeeds)
    ));

    let p_alpha = ApprhParams {
        ppr: PprParams {
            alpha: 1.0,
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(matches!(
        apprh_local(&g, &seeds, &p_alpha),
        Err(ApprhError::InvalidParameter(_))
    ));

    let p_decay = ApprhParams {
        hyperedge_decay_factor: 0.0,
        ..Default::default()
    };
    assert!(matches!(
        apprh_local(&g, &seeds, &p_decay),
        Err(ApprhError::InvalidParameter(_))
    ));

    let p_iter = ApprhParams {
        max_iterations: 0,
        ..Default::default()
    };
    assert!(matches!(
        apprh_local(&g, &seeds, &p_iter),
        Err(ApprhError::InvalidParameter(_))
    ));
}

#[test]
fn test_apprh_shadow_mode_comparison() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);

    g.add_binary_edge(e1, e2, 1.0);
    g.add_binary_edge(e2, e3, 1.0);

    let seeds = vec![e1];
    let ppr_params = PprParams::default();
    let apprh_params = ApprhParams::default();

    let comp = shadow_compare_forward_push_vs_apprh(&g, &seeds, &ppr_params, &apprh_params, 3)?;

    assert!(!comp.forward_push_results.is_empty());
    assert!(!comp.apprh_results.is_empty());
    assert!(comp.top_k_overlap >= 0.0 && comp.top_k_overlap <= 1.0);

    Ok(())
}
