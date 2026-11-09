//! Isolierte Algorithmentests auf Test-Doubles (M3, M11).
//!
//! # SPEC: T-GRAPH-ALGO-DOUBLE
//! # Zweck:
//! Prüft, dass Algorithmen (PathRAG, Leiden-Cluster-Iteratoren, Traversierungsoperatoren)
//! ohne Abhängigkeit von konkreten CsrGraph-Persistenz-Interna auf isolierten
//! Graph-Lese-Traits (`PathGraph`) ausgeführt werden können.

#![expect(clippy::unwrap_used)]

use contextra_graph::path_rag::{PathGraph, PathRAGEngine};
use contextra_graph::types::EntityId;
use std::collections::HashMap;

/// In-Memory Test-Double für den `PathGraph`-Trait.
struct MockPathGraph {
    edges: HashMap<EntityId, Vec<(EntityId, f32)>>,
    rev_edges: HashMap<EntityId, Vec<(EntityId, f32)>>,
}

impl MockPathGraph {
    fn new() -> Self {
        Self {
            edges: HashMap::new(),
            rev_edges: HashMap::new(),
        }
    }

    fn add_directed_edge(&mut self, from: EntityId, to: EntityId, weight: f32) {
        self.edges.entry(from).or_default().push((to, weight));
        self.rev_edges.entry(to).or_default().push((from, weight));
    }
}

impl PathGraph for MockPathGraph {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.edges.get(&node).cloned().unwrap_or_default()
    }

    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.rev_edges.get(&node).cloned().unwrap_or_default()
    }
}

/// SPEC: T-GRAPH-ALGO-DOUBLE-01
/// Prüft bidirektionale Pfadsuche in PathRAG über ein Test-Double ohne CsrGraph.
#[test]
fn test_path_rag_engine_with_mock_double() {
    let mut mock = MockPathGraph::new();

    let node_a = EntityId::new(10);
    let node_b = EntityId::new(20);
    let node_c = EntityId::new(30);

    // Graph: A -(0.8)-> B -(0.9)-> C
    mock.add_directed_edge(node_a, node_b, 0.8);
    mock.add_directed_edge(node_b, node_c, 0.9);

    let engine = PathRAGEngine::new(mock, 3, 0.01);
    let path_opt = engine.find_path(node_a, node_c);

    assert!(path_opt.is_some());
    let path = path_opt.unwrap();
    assert_eq!(path.nodes, vec![node_a, node_b, node_c]);
    assert_eq!(path.edge_weights.len(), 2);
    assert!((path.confidence - (0.8 * 0.9)).abs() < 1e-4);
}
