//! Integration and parity tests for Thresholded Local Hyper-Flow Diffusion (TL-HFD, AK-16).

use contextra_graph::path_rag::{PathGraph, PprParams};
use contextra_graph::{
    shadow_compare_forward_push_vs_tl_hfd, tl_hfd_local, HyperEdge, HyperEdgeId, RoleBinding,
    RoleId, TlHfdError, TlHfdParams,
};
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
fn test_determinism_20_runs_and_seed_permutations() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);
    let e4 = EntityId::new(4);

    g.add_binary_edge(e1, e2, 1.0);
    g.add_binary_edge(e2, e3, 0.8);
    g.add_hyperedge(100, vec![e1, e3, e4], 1.5);

    let params = TlHfdParams::default();

    // 1. Bit-identical across 20 iterations
    let seeds = vec![e1, e2];
    let base_res = tl_hfd_local(&g, &seeds, &params)?;

    for _ in 0..20 {
        let run_res = tl_hfd_local(&g, &seeds, &params)?;
        assert_eq!(base_res.len(), run_res.len());
        for (k, v) in &base_res {
            if let Some(run_val) = run_res.get(k) {
                assert_eq!(v.to_bits(), run_val.to_bits());
            } else {
                return Err("key missing in run_res".into());
            }
        }
    }

    // 2. Seed order permutation bit-identical
    let permuted_seeds = vec![e2, e1];
    let permuted_res = tl_hfd_local(&g, &permuted_seeds, &params)?;
    assert_eq!(base_res.len(), permuted_res.len());
    for (k, v) in &base_res {
        if let Some(perm_val) = permuted_res.get(k) {
            assert_eq!(v.to_bits(), perm_val.to_bits());
        } else {
            return Err("key missing in permuted_res".into());
        }
    }

    Ok(())
}

#[test]
fn test_locality_bounded_visited_set() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    // 10,000 chain graph
    for i in 1..10000 {
        g.add_binary_edge(EntityId::new(i), EntityId::new(i + 1), 1.0);
    }

    let seeds = vec![EntityId::new(1)];
    let params = TlHfdParams {
        max_iterations: 5,
        max_top_k_expansion: 2,
        ..Default::default()
    };

    let res = tl_hfd_local(&g, &seeds, &params)?;

    // Max active nodes = seeds + max_iterations * max_top_k_expansion = 1 + 5*2 = 11
    let max_expected_nodes = 1 + 5 * 2;
    assert!(
        res.len() <= max_expected_nodes,
        "Visited nodes {} exceeded max locality limit {}",
        res.len(),
        max_expected_nodes
    );

    Ok(())
}

#[test]
fn test_p24_boundary_large_hyperedge() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    // Hyperedge with 5,000 participants
    let participants: Vec<EntityId> = (1..=5000).map(EntityId::new).collect();
    g.add_hyperedge(999, participants, 1.0);

    let seeds = vec![EntityId::new(1)];
    let params = TlHfdParams {
        max_hyperedge_sort_size: 64,
        max_iterations: 5,
        ..Default::default()
    };

    let map = tl_hfd_local(&g, &seeds, &params)?;
    assert!(!map.is_empty());
    for (&node, &score) in &map {
        assert!(
            score.is_finite() && score > 0.0,
            "Node {:?} score {} must be positive and finite",
            node,
            score
        );
    }

    Ok(())
}

#[test]
fn test_parameter_validation_and_no_panic() {
    let g = MockGraph::new();
    let seeds = vec![EntityId::new(1)];

    let p_delta = TlHfdParams {
        delta: 1.99,
        ..Default::default()
    };
    assert!(matches!(
        tl_hfd_local(&g, &seeds, &p_delta),
        Err(TlHfdError::InvalidParameter(_))
    ));

    let p_sigma = TlHfdParams {
        sigma: 0.0,
        ..Default::default()
    };
    assert!(matches!(
        tl_hfd_local(&g, &seeds, &p_sigma),
        Err(TlHfdError::InvalidParameter(_))
    ));

    let p_gamma = TlHfdParams {
        gamma: -0.1,
        ..Default::default()
    };
    assert!(matches!(
        tl_hfd_local(&g, &seeds, &p_gamma),
        Err(TlHfdError::InvalidParameter(_))
    ));

    let p_sort = TlHfdParams {
        max_hyperedge_sort_size: 1,
        ..Default::default()
    };
    assert!(matches!(
        tl_hfd_local(&g, &seeds, &p_sort),
        Err(TlHfdError::InvalidParameter(_))
    ));
}

#[test]
fn test_symmetric_graph_tie_breaker_order() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let root = EntityId::new(10);
    let n2 = EntityId::new(2);
    let n3 = EntityId::new(3);

    g.add_binary_edge(root, n2, 1.0);
    g.add_binary_edge(root, n3, 1.0);

    let seeds = vec![root];
    let ppr_params = PprParams::default();
    let tl_params = TlHfdParams::default();

    let comp = shadow_compare_forward_push_vs_tl_hfd(&g, &seeds, &ppr_params, &tl_params, 3)?;

    // In symmetric graph, n2 (EntityId 2) must sort before n3 (EntityId 3) when scores match
    let tl_results = comp.tl_hfd_results;
    if tl_results.len() >= 3 {
        let (id2, s2) = tl_results[1];
        let (id3, s3) = tl_results[2];
        if (s2 - s3).abs() < 1e-6 {
            assert_eq!(id2, n2, "Tie-breaker must prioritize smaller EntityId");
            assert_eq!(id3, n3);
        }
    }

    Ok(())
}

#[test]
fn test_shadow_mode_parity_2_cluster_graph() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    // Cluster 1: 1, 2, 3
    let c1 = EntityId::new(1);
    let c2 = EntityId::new(2);
    let c3 = EntityId::new(3);
    g.add_binary_edge(c1, c2, 1.0);
    g.add_binary_edge(c2, c3, 1.0);
    g.add_binary_edge(c1, c3, 1.0);

    // Cluster 2: 4, 5, 6
    let d4 = EntityId::new(4);
    let d5 = EntityId::new(5);
    let d6 = EntityId::new(6);
    g.add_binary_edge(d4, d5, 1.0);
    g.add_binary_edge(d5, d6, 1.0);
    g.add_binary_edge(d4, d6, 1.0);

    // Bridge
    g.add_binary_edge(c3, d4, 0.01);

    let seeds = vec![c1];

    let ppr_params = PprParams {
        epsilon: 0.02,
        ..Default::default()
    };
    let tl_params = TlHfdParams::default();

    let comp = shadow_compare_forward_push_vs_tl_hfd(&g, &seeds, &ppr_params, &tl_params, 3)?;

    assert!(
        comp.top_k_overlap >= 0.66,
        "Top-3 overlap {} must be >= 0.66 on 2-cluster graph",
        comp.top_k_overlap
    );
    assert!(
        !comp.discrepancy,
        "Discrepancy must be false on balanced cluster graph"
    );

    Ok(())
}

#[test]
fn test_shadow_mode_discrepancy_reported() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);

    g.add_binary_edge(e1, e2, 1.0);
    g.add_binary_edge(e2, e3, 1.0);

    let seeds = vec![e1];
    // Strict epsilon forces discrepancy flag
    let ppr_params = PprParams {
        epsilon: 1e-9,
        ..Default::default()
    };
    let tl_params = TlHfdParams {
        max_iterations: 1,
        ..Default::default()
    };

    let comp = shadow_compare_forward_push_vs_tl_hfd(&g, &seeds, &ppr_params, &tl_params, 3)?;
    assert!(
        comp.discrepancy,
        "Discrepancy must be reported when epsilon condition is breached"
    );

    Ok(())
}
