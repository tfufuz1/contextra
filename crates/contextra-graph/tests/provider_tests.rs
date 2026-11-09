//! Integration test verifying isolated execution of algorithms on `InMemoryGraphProvider`.
//! Ensures `contextra-graph` operates as an independent graph math library without storage or database crates.

use contextra_graph::community::CommunityDetectionConfig;
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_graph::provider::{
    compute_pagerank, detect_communities_on_provider, GraphProvider, InMemoryGraphProvider,
};
use contextra_graph::types::EntityId;
use contextra_graph::PathRAGEngine;

#[test]
fn test_in_memory_graph_provider_pagerank() {
    let mut graph = InMemoryGraphProvider::new();

    let node1 = EntityId::new(1);
    let node2 = EntityId::new(2);
    let node3 = EntityId::new(3);

    // 1 -> 2, 2 -> 3, 3 -> 1 (ring)
    graph.add_edge(node1, node2, 1.0);
    graph.add_edge(node2, node3, 1.0);
    graph.add_edge(node3, node1, 1.0);

    let ranks = compute_pagerank(&graph, 0.85, 100, 1e-6);
    assert_eq!(ranks.len(), 3);

    let r1 = ranks.get(&node1).copied().unwrap();
    let r2 = ranks.get(&node2).copied().unwrap();
    let r3 = ranks.get(&node3).copied().unwrap();

    assert!((r1 - r2).abs() < 1e-4);
    assert!((r2 - r3).abs() < 1e-4);
    let sum: f32 = ranks.values().sum();
    assert!((sum - 1.0).abs() < 1e-3);
}

#[test]
fn test_in_memory_graph_provider_community_detection() {
    let mut graph = InMemoryGraphProvider::new();

    // Cluster 1: 1, 2, 3
    graph.add_edge(EntityId::new(1), EntityId::new(2), 1.0);
    graph.add_edge(EntityId::new(2), EntityId::new(1), 1.0);
    graph.add_edge(EntityId::new(2), EntityId::new(3), 1.0);
    graph.add_edge(EntityId::new(3), EntityId::new(2), 1.0);

    // Cluster 2: 10, 20, 30
    graph.add_edge(EntityId::new(10), EntityId::new(20), 1.0);
    graph.add_edge(EntityId::new(20), EntityId::new(10), 1.0);
    graph.add_edge(EntityId::new(20), EntityId::new(30), 1.0);
    graph.add_edge(EntityId::new(30), EntityId::new(20), 1.0);

    let config = CommunityDetectionConfig {
        max_iterations: 100,
        seed: 42,
        hyperedges_included: false,
    };

    let communities = detect_communities_on_provider(&graph, &config).unwrap();
    assert_eq!(communities.len(), 6);

    let mut comm_map = std::collections::HashMap::new();
    for assignment in communities {
        comm_map.insert(assignment.entity_id, assignment.community_id);
    }

    assert_eq!(
        comm_map.get(&EntityId::new(1)),
        comm_map.get(&EntityId::new(2))
    );
    assert_eq!(
        comm_map.get(&EntityId::new(2)),
        comm_map.get(&EntityId::new(3))
    );

    assert_eq!(
        comm_map.get(&EntityId::new(10)),
        comm_map.get(&EntityId::new(20))
    );
    assert_eq!(
        comm_map.get(&EntityId::new(20)),
        comm_map.get(&EntityId::new(30))
    );

    assert_ne!(
        comm_map.get(&EntityId::new(1)),
        comm_map.get(&EntityId::new(10))
    );
}

#[test]
fn test_in_memory_graph_provider_path_rag() {
    let mut graph = InMemoryGraphProvider::new();

    let node1 = EntityId::new(1);
    let node2 = EntityId::new(2);
    let node3 = EntityId::new(3);

    graph.add_edge(node1, node2, 0.9);
    graph.add_edge(node2, node3, 0.8);

    let engine = PathRAGEngine::new(graph, 3, 0.5);
    let path = engine.find_path(node1, node3);
    assert!(path.is_some());

    let p = path.unwrap();
    assert_eq!(p.nodes, vec![node1, node2, node3]);
    assert!((p.confidence - 0.72).abs() < 1e-4);
}

#[test]
fn test_in_memory_graph_provider_hyperedges() {
    let mut graph = InMemoryGraphProvider::new();

    let node1 = EntityId::new(10);
    let node2 = EntityId::new(20);
    let node3 = EntityId::new(30);

    const ROLE: RoleId = RoleId::new(1);
    let he_id = HyperEdgeId::new(100);
    let hyperedge = HyperEdge::new(
        he_id,
        contextra_graph::csr::EdgeType::Default,
        vec![
            RoleBinding::new(ROLE, node1),
            RoleBinding::new(ROLE, node2),
            RoleBinding::new(ROLE, node3),
        ],
        1.5,
    );

    graph.add_hyperedge(hyperedge);

    assert_eq!(graph.hyperedges_for_entity(node1), vec![he_id]);
    assert_eq!(graph.hyperedges_for_entity(node2), vec![he_id]);
    assert_eq!(graph.hyperedges_for_entity(node3), vec![he_id]);

    let resolved = graph.get_hyperedge(he_id);
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().participants.len(), 3);
}
