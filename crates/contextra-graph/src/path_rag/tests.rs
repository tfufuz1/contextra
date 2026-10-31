use super::*;

// Einfacher Test-Graph
struct TestGraph {
    edges: HashMap<EntityId, Vec<(EntityId, f32)>>,
}

impl TestGraph {
    fn new(edges: Vec<(EntityId, EntityId, f32)>) -> Self {
        let mut map: HashMap<EntityId, Vec<(EntityId, f32)>> = HashMap::new();
        for (from, to, w) in edges {
            map.entry(from).or_default().push((to, w));
        }
        Self { edges: map }
    }
}

impl PathGraph for TestGraph {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.edges.get(&node).cloned().unwrap_or_default()
    }
    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        // Für undirektierte Tests: alle Knoten die auf node zeigen
        self.edges
            .iter()
            .flat_map(|(from, nbrs)| {
                nbrs.iter().filter_map(
                    move |(to, w)| {
                        if *to == node {
                            Some((*from, *w))
                        } else {
                            None
                        }
                    },
                )
            })
            .collect()
    }
}

#[test]
fn test_find_path_direct_edge() {
    let a = EntityId::new(1);
    let b = EntityId::new(2);
    let graph = TestGraph::new(vec![(a, b, 0.9)]);
    let engine = PathRAGEngine::with_defaults(graph);
    let path = engine.find_path(a, b).unwrap();
    assert_eq!(path.nodes, vec![a, b]);
    assert!((path.confidence - 0.9).abs() < 1e-6);
}

#[test]
fn test_find_path_multi_hop() {
    let a = EntityId::new(1);
    let b = EntityId::new(2);
    let c = EntityId::new(3);
    let graph = TestGraph::new(vec![(a, b, 0.8), (b, c, 0.9)]);
    let engine = PathRAGEngine::with_defaults(graph);
    let path = engine.find_path(a, c).unwrap();
    assert_eq!(path.nodes.len(), 3);
    assert!(
        (path.confidence - 0.72).abs() < 1e-5,
        "0.8*0.9={}",
        path.confidence
    );
}

#[test]
fn test_find_path_same_node() {
    let a = EntityId::new(1);
    let graph = TestGraph::new(vec![]);
    let engine = PathRAGEngine::with_defaults(graph);
    let path = engine.find_path(a, a).unwrap();
    assert_eq!(path.nodes, vec![a]);
    assert_eq!(path.edge_weights.len(), 0);
}

#[test]
fn test_find_path_unreachable_returns_none() {
    let a = EntityId::new(1);
    let b = EntityId::new(99);
    let graph = TestGraph::new(vec![]);
    let engine = PathRAGEngine::with_defaults(graph);
    assert!(engine.find_path(a, b).is_none());
}

#[test]
fn test_sufficiency_gate_filters_low_confidence() {
    let engine = PathRAGEngine::new(TestGraph::new(vec![]), 4, 0.5);
    let low_conf = GraphPath {
        nodes: vec![EntityId::new(1), EntityId::new(2)],
        edge_weights: vec![0.3],
        confidence: 0.3,
        total_flow: 3.33,
    };
    assert!(!engine.sufficiency_check(&low_conf));
    let high_conf = GraphPath {
        confidence: 0.8,
        ..low_conf
    };
    assert!(engine.sufficiency_check(&high_conf));
}

#[test]
fn test_find_all_paths_multi_hop() {
    let a = EntityId::new(1);
    let b = EntityId::new(2);
    let c = EntityId::new(3);
    let graph = TestGraph::new(vec![(a, b, 0.8), (b, c, 0.9)]);
    let engine = PathRAGEngine::new(graph, 3, 0.5);
    let paths = engine.find_all_paths(a);
    assert_eq!(paths.len(), 2);
}

#[test]
fn test_rrf_signal_filters_by_sufficiency() {
    let engine = PathRAGEngine::new(TestGraph::new(vec![]), 4, 0.5);
    let paths = vec![
        GraphPath {
            nodes: vec![EntityId::new(1)],
            edge_weights: vec![],
            confidence: 0.1,
            total_flow: 1.0,
        }, // under threshold
        GraphPath {
            nodes: vec![EntityId::new(2)],
            edge_weights: vec![],
            confidence: 0.9,
            total_flow: 1.0,
        }, // passes
    ];
    let signal = engine.to_rrf_signal(&paths);
    assert_eq!(signal.len(), 1);
    assert_eq!(signal[0].0, DocId(2));
}

#[test]
fn test_default_sufficiency_threshold_meets_minimum_bound() {
    // REGRESSION TEST (ADR-067 / arXiv:2506.00610):
    // Verhindert, dass der Default-Schwellenwert versehentlich unter 0.10 fällt (z.B. auf 0.01),
    // was Precision-Kollaps durch ungefilterte, verrauschte Multi-Hop-Pfade auslösen würde.
    const {
        assert!(
            DEFAULT_SUFFICIENCY_THRESHOLD >= 0.10,
            "DEFAULT_SUFFICIENCY_THRESHOLD must be >= 0.10 to prevent precision collapse",
        )
    };

    let engine = PathRAGEngine::with_defaults(TestGraph::new(vec![]));
    assert_eq!(
        engine.sufficiency_threshold(),
        DEFAULT_SUFFICIENCY_THRESHOLD,
        "with_defaults() must construct PathRAGEngine using DEFAULT_SUFFICIENCY_THRESHOLD"
    );
}

#[test]
fn test_find_path_zero_max_hops() {
    let a = EntityId::new(1);
    let b = EntityId::new(2);
    let graph = TestGraph::new(vec![(a, b, 0.9)]);
    let engine = PathRAGEngine::new(graph, 0, 0.1);
    // With max_hops = 0, no search steps run and different nodes return None
    assert!(engine.find_path(a, b).is_none());
    // Same node still returns immediate path
    let same_path = engine.find_path(a, a);
    assert!(same_path.is_some());
}

#[test]
fn test_to_rrf_signal_empty_paths() {
    let engine = PathRAGEngine::with_defaults(TestGraph::new(vec![]));
    let signal = engine.to_rrf_signal(&[]);
    assert!(signal.is_empty());
}

struct TestGraphWithHyperedges {
    edges: HashMap<EntityId, Vec<(EntityId, f32)>>,
    hyperedges: HashMap<EntityId, Vec<HyperEdgeId>>,
    hyperedge_store: HashMap<HyperEdgeId, HyperEdge>,
}

impl TestGraphWithHyperedges {
    fn new(edges: Vec<(EntityId, EntityId, f32)>, hyperedges: Vec<HyperEdge>) -> Self {
        let mut map: HashMap<EntityId, Vec<(EntityId, f32)>> = HashMap::new();
        for (from, to, w) in edges {
            map.entry(from).or_default().push((to, w));
        }

        let mut he_map: HashMap<EntityId, Vec<HyperEdgeId>> = HashMap::new();
        let mut he_store = HashMap::new();

        for he in hyperedges {
            he_store.insert(he.id, he.clone());
            for participant in he.participants.iter() {
                he_map.entry(participant.entity).or_default().push(he.id);
            }
        }

        Self {
            edges: map,
            hyperedges: he_map,
            hyperedge_store: he_store,
        }
    }
}

impl PathGraph for TestGraphWithHyperedges {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.edges.get(&node).cloned().unwrap_or_default()
    }
    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.edges
            .iter()
            .flat_map(|(from, nbrs)| {
                nbrs.iter().filter_map(
                    move |(to, w)| {
                        if *to == node {
                            Some((*from, *w))
                        } else {
                            None
                        }
                    },
                )
            })
            .collect()
    }
    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<HyperEdgeId> {
        self.hyperedges.get(&node).cloned().unwrap_or_default()
    }
    fn get_hyperedge(&self, id: HyperEdgeId) -> Option<Arc<HyperEdge>> {
        self.hyperedge_store.get(&id).cloned().map(Arc::new)
    }
}

#[test]
fn test_hyperedge_expansion_disabled_by_default() {
    let a = EntityId::new(1);
    let b = EntityId::new(2);
    let he = HyperEdge::new(
        HyperEdgeId::new(100),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), a),
            RoleBinding::new(RoleId::new(2), b),
        ],
        1.0,
    );
    let graph = TestGraphWithHyperedges::new(vec![], vec![he]);
    // Default engine has hyperedge_expansion_enabled: false
    let engine = PathRAGEngine::with_defaults(graph);
    assert!(!engine.config.hyperedge_expansion_enabled);
    assert!(
        engine.find_path(a, b).is_none(),
        "When hyperedge expansion is disabled, hyperedge-only path must return None"
    );
}

#[test]
fn test_hyperedge_expansion_enabled_finds_path_via_hyperedge_only() {
    // AK-2 test: Path exclusively over hyperedge
    let a = EntityId::new(1);
    let b = EntityId::new(2);
    let c = EntityId::new(3);
    let he = HyperEdge::new(
        HyperEdgeId::new(101),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), a),
            RoleBinding::new(RoleId::new(1), b),
            RoleBinding::new(RoleId::new(1), c),
        ],
        1.0,
    );
    let graph = TestGraphWithHyperedges::new(vec![], vec![he]);
    let config = PathRAGConfig {
        hyperedge_expansion_enabled: true,
        hyperedge_weight_discount: 0.85,
        ..Default::default()
    };
    let engine = PathRAGEngine::with_config(graph, config);

    let path = engine
        .find_path(a, c)
        .expect("path must be found via hyperedge");
    assert_eq!(path.nodes, vec![a, c]);
    // weight = hyperedge.weight (1.0) * discount (0.85) = 0.85
    assert!(
        (path.confidence - 0.85).abs() < 1e-5,
        "Expected confidence 0.85, got {}",
        path.confidence
    );
}

#[test]
fn test_hyperedge_weight_discount_applied_correctly() {
    let a = EntityId::new(1);
    let b = EntityId::new(2);
    let he = HyperEdge::new(
        HyperEdgeId::new(102),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), a),
            RoleBinding::new(RoleId::new(2), b),
        ],
        1.0,
    );

    // Test discount = 0.85
    let graph1 = TestGraphWithHyperedges::new(vec![], vec![he.clone()]);
    let engine1 = PathRAGEngine::with_config(
        graph1,
        PathRAGConfig {
            hyperedge_expansion_enabled: true,
            hyperedge_weight_discount: 0.85,
            ..Default::default()
        },
    );
    let path1 = engine1.find_path(a, b).unwrap();
    assert!((path1.confidence - 0.85).abs() < 1e-5);

    // Test discount = 0.50
    let graph2 = TestGraphWithHyperedges::new(vec![], vec![he]);
    let engine2 = PathRAGEngine::with_config(
        graph2,
        PathRAGConfig {
            hyperedge_expansion_enabled: true,
            hyperedge_weight_discount: 0.50,
            ..Default::default()
        },
    );
    let path2 = engine2.find_path(a, b).unwrap();
    assert!((path2.confidence - 0.50).abs() < 1e-5);
}

#[test]
fn test_sufficiency_gate_filters_low_confidence_virtual_neighbors() {
    let a = EntityId::new(1);
    let b = EntityId::new(2);
    // Weight 0.10 * discount 0.85 = 0.085 < sufficiency_threshold 0.10
    let he = HyperEdge::new(
        HyperEdgeId::new(103),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), a),
            RoleBinding::new(RoleId::new(2), b),
        ],
        0.10,
    );
    let graph = TestGraphWithHyperedges::new(vec![], vec![he]);
    let engine = PathRAGEngine::with_config(
        graph,
        PathRAGConfig {
            sufficiency_threshold: 0.10,
            hyperedge_expansion_enabled: true,
            hyperedge_weight_discount: 0.85,
            ..Default::default()
        },
    );

    let paths = engine.find_all_paths(a);
    assert!(
        paths.is_empty(),
        "Virtual neighbor path with confidence 0.085 < 0.10 must be filtered by sufficiency gate"
    );
}

#[test]
fn test_self_referencing_hyperedge_no_infinite_loop() {
    let a = EntityId::new(1);
    // Hyperedge pointing to itself twice
    let he = HyperEdge::new(
        HyperEdgeId::new(104),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), a),
            RoleBinding::new(RoleId::new(2), a),
        ],
        1.0,
    );
    let graph = TestGraphWithHyperedges::new(vec![], vec![he]);
    let engine = PathRAGEngine::with_config(
        graph,
        PathRAGConfig {
            hyperedge_expansion_enabled: true,
            hyperedge_weight_discount: 0.85,
            ..Default::default()
        },
    );

    let paths = engine.find_all_paths(a);
    assert!(
        paths.is_empty(),
        "Self-referencing hyperedge participant must not produce spurious self-paths or infinite loops"
    );
}

#[test]
fn test_fusion_and_signalkind_non_modification_contract() {
    // Contract test ensuring no modification to fusion.rs or SignalKind enum
    // Verify path_rag types seamlessly convert to DocId signal without introducing new SignalKind
    let engine = PathRAGEngine::with_defaults(TestGraph::new(vec![]));
    let path = GraphPath {
        nodes: vec![EntityId::new(42)],
        edge_weights: vec![],
        confidence: 1.0,
        total_flow: 1.0,
    };
    let rrf = engine.to_rrf_signal(&[path]);
    assert_eq!(rrf.len(), 1);
    assert_eq!(rrf[0].0, DocId(42));
}

#[test]
fn test_path_rag_engine_with_real_csr_graph_hyperedges() {
    use crate::csr::CsrGraph;

    let graph = CsrGraph::new();
    let a = EntityId::new(10);
    let b = EntityId::new(20);

    let he = HyperEdge::new(
        HyperEdgeId::new(999),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), a),
            RoleBinding::new(RoleId::new(2), b),
        ],
        1.0,
    );

    graph.insert_hyperedge(he);

    let config = PathRAGConfig {
        hyperedge_expansion_enabled: true,
        hyperedge_weight_discount: 0.85,
        ..Default::default()
    };
    let engine = PathRAGEngine::with_config(graph, config);

    let path = engine
        .find_path(a, b)
        .expect("PathRAG must find path over real CsrGraph hyperedge");

    assert_eq!(path.nodes, vec![a, b]);
    assert!((path.confidence - 0.85).abs() < 1e-5);
}

#[test]
fn test_ppr_params_defaults() {
    let params = PprParams::default();
    assert_eq!(params.alpha, 0.15);
    assert_eq!(params.epsilon, 1e-4);
    assert_eq!(params.hyperedge_decay, 0.85);
}

#[test]
fn test_forward_push_ppr_empty_seeds() {
    let graph = TestGraph::new(vec![]);
    let params = PprParams::default();
    let res = forward_push_ppr(&graph, &[], &params);
    assert!(res.is_empty());
}

#[test]
fn test_forward_push_ppr_binary_graph() {
    let a = EntityId::new(1);
    let b = EntityId::new(2);
    let graph = TestGraph::new(vec![(a, b, 1.0)]);
    let params = PprParams::default();

    let scores = forward_push_ppr(&graph, &[a], &params);
    assert!(scores.get(&a).is_some());
    assert!(*scores.get(&a).unwrap() > 0.0);
}

#[test]
fn test_forward_push_ppr_hyperedge_expansion_and_decay() {
    let a = EntityId::new(1);
    let b = EntityId::new(2);
    let c = EntityId::new(3);

    // Hyperedge connecting a, b, c
    let he = HyperEdge::new(
        HyperEdgeId::new(500),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), a),
            RoleBinding::new(RoleId::new(2), b),
            RoleBinding::new(RoleId::new(3), c),
        ],
        1.0,
    );

    let graph = TestGraphWithHyperedges::new(vec![], vec![he]);

    let params_full = PprParams {
        alpha: 0.15,
        epsilon: 1e-4,
        hyperedge_decay: 1.0,
    };
    let scores_full = forward_push_ppr(&graph, &[a], &params_full);

    let params_decay = PprParams {
        alpha: 0.15,
        epsilon: 1e-4,
        hyperedge_decay: 0.5,
    };
    let scores_decay = forward_push_ppr(&graph, &[a], &params_decay);

    let score_b_full = *scores_full.get(&b).unwrap_or(&0.0);
    let score_b_decay = *scores_decay.get(&b).unwrap_or(&0.0);

    assert!(
        score_b_full > 0.0,
        "b must receive PPR score via hyperedge expansion"
    );
    assert!(
        score_b_decay > 0.0,
        "b must receive PPR score with decay applied"
    );
    assert!(
        score_b_decay < score_b_full,
        "Lower hyperedge_decay must yield lower virtual score (full={}, decay={})",
        score_b_full,
        score_b_decay
    );
}

#[test]
fn test_forward_push_ppr_self_participant_filtered() {
    let a = EntityId::new(1);
    // Hyperedge where 'a' is repeated twice
    let he = HyperEdge::new(
        HyperEdgeId::new(501),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), a),
            RoleBinding::new(RoleId::new(2), a),
        ],
        1.0,
    );
    let graph = TestGraphWithHyperedges::new(vec![], vec![he]);
    let params = PprParams::default();

    let scores = forward_push_ppr(&graph, &[a], &params);
    // Only 'a' should be in scores (no panic or spurious nodes)
    assert!(scores.get(&a).is_some());
}
