use contextra_core::{Edge, Entity, EntityId, GraphIndex, PprAlgorithm, PprConfig, TxId};
use contextra_graph::CsrGraph;

#[tokio::test]
async fn test_ppr_snapshot_isolation_temporal_visibility() {
    let graph = CsrGraph::new();

    let node_a = EntityId::new(101);
    let node_b = EntityId::new(102);
    let node_c = EntityId::new(103);

    let tx10 = TxId::new(10);
    graph.add_entity(tx10, Entity::new(node_a, "node_a", "Node")).await.unwrap();
    graph.add_entity(tx10, Entity::new(node_b, "node_b", "Node")).await.unwrap();
    GraphIndex::add_edge(&graph, tx10, Edge::new(node_a, node_b, "rel").with_weight(1.0)).await.unwrap();
    graph.commit(tx10).await.unwrap();

    let tx20 = TxId::new(20);
    graph.add_entity(tx20, Entity::new(node_c, "node_c", "Node")).await.unwrap();
    GraphIndex::add_edge(&graph, tx20, Edge::new(node_a, node_c, "rel").with_weight(1.0)).await.unwrap();
    graph.commit(tx20).await.unwrap();

    let cfg = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: PprAlgorithm::DensePowerIteration,
        warn_on_non_convergence: true,
    };

    // Snapshot at seq 10: only B should be visible from A
    let res_seq10 = graph.personalized_page_rank_at(&[node_a], &cfg, 10).await.unwrap();
    let eids_seq10: Vec<_> = res_seq10.iter().map(|(id, _)| *id).collect();
    assert!(eids_seq10.contains(&node_b), "node_b must be present at seq 10");
    assert!(!eids_seq10.contains(&node_c), "node_c must NOT be present at seq 10");

    // Snapshot at seq 20: both B and C should be visible from A
    let res_seq20 = graph.personalized_page_rank_at(&[node_a], &cfg, 20).await.unwrap();
    let eids_seq20: Vec<_> = res_seq20.iter().map(|(id, _)| *id).collect();
    assert!(eids_seq20.contains(&node_b), "node_b must be present at seq 20");
    assert!(eids_seq20.contains(&node_c), "node_c must be present at seq 20");
}

#[tokio::test]
async fn test_ppr_snapshot_isolation_matches_live_at_latest_seq() {
    let graph = CsrGraph::new();

    let node_a = EntityId::new(201);
    let node_b = EntityId::new(202);
    let node_c = EntityId::new(203);

    let tx100 = TxId::new(100);
    graph.add_entity(tx100, Entity::new(node_a, "node_a", "Node")).await.unwrap();
    graph.add_entity(tx100, Entity::new(node_b, "node_b", "Node")).await.unwrap();
    graph.add_entity(tx100, Entity::new(node_c, "node_c", "Node")).await.unwrap();

    GraphIndex::add_edge(&graph, tx100, Edge::new(node_a, node_b, "rel").with_weight(0.8)).await.unwrap();
    GraphIndex::add_edge(&graph, tx100, Edge::new(node_b, node_c, "rel").with_weight(0.5)).await.unwrap();
    graph.commit(tx100).await.unwrap();

    let cfg = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: PprAlgorithm::ForwardPush,
        warn_on_non_convergence: true,
    };

    let live_res = graph.personalized_page_rank(&[node_a], &cfg).await.unwrap();
    let snap_res = graph.personalized_page_rank_at(&[node_a], &cfg, 100).await.unwrap();

    assert_eq!(live_res.len(), snap_res.len(), "Live and snapshot result count must match");

    for ((live_id, live_score), (snap_id, snap_score)) in live_res.iter().zip(snap_res.iter()) {
        assert_eq!(live_id, snap_id, "Entity ordering must match");
        assert!((live_score - snap_score).abs() < 1e-5, "Scores must match within tolerance: live={live_score}, snap={snap_score}");
    }
}

#[tokio::test]
async fn test_ppr_snapshot_isolation_tombstone_and_validity_range() {
    let graph = CsrGraph::new();

    let node_a = EntityId::new(301);
    let node_b = EntityId::new(302);

    let tx10 = TxId::new(10);
    graph.add_entity(tx10, Entity::new(node_a, "node_a", "Node")).await.unwrap();
    graph.add_entity(tx10, Entity::new(node_b, "node_b", "Node")).await.unwrap();

    // Edge valid from tx10 to tx20 (invalidated at tx20)
    let edge = Edge::new(node_a, node_b, "rel")
        .with_weight(1.0)
        .with_tx_validity(Some(tx10), Some(TxId::new(20)));
    GraphIndex::add_edge(&graph, tx10, edge).await.unwrap();
    graph.commit(tx10).await.unwrap();

    let cfg = PprConfig::default();

    // Snapshot at seq 10 (edge active): node_b visible
    let res_seq10 = graph.personalized_page_rank_at(&[node_a], &cfg, 10).await.unwrap();
    let ids_seq10: Vec<_> = res_seq10.iter().map(|(id, _)| *id).collect();
    assert!(ids_seq10.contains(&node_b), "node_b must be present at seq 10");

    // Snapshot at seq 20 (edge invalidated by tx_valid_to = 20): node_b not visible
    let res_seq20 = graph.personalized_page_rank_at(&[node_a], &cfg, 20).await.unwrap();
    let ids_seq20: Vec<_> = res_seq20.iter().map(|(id, _)| *id).collect();
    assert!(!ids_seq20.contains(&node_b), "node_b must NOT be present at seq 20 after edge validity expiration");
}

#[tokio::test]
async fn test_ppr_snapshot_isolation_hyperedge_visibility() {
    use contextra_graph::csr::EdgeType;
    use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

    let graph = CsrGraph::new();

    let node_a = EntityId::new(401);
    let node_b = EntityId::new(402);
    let node_c = EntityId::new(403);

    let tx10 = TxId::new(10);
    graph.add_entity(tx10, Entity::new(node_a, "node_a", "Node")).await.unwrap();
    graph.add_entity(tx10, Entity::new(node_b, "node_b", "Node")).await.unwrap();
    graph.add_entity(tx10, Entity::new(node_c, "node_c", "Node")).await.unwrap();
    graph.commit(tx10).await.unwrap();

    let tx20 = TxId::new(20);
    let roles = vec![
        RoleBinding::new(RoleId::new(1), node_a),
        RoleBinding::new(RoleId::new(2), node_b),
        RoleBinding::new(RoleId::new(3), node_c),
    ];
    let hedge = HyperEdge::new(HyperEdgeId::new(1), EdgeType::Default, roles, 1.0)
        .with_tx_validity(Some(tx20), None);
    graph.insert_hyperedge_direct(hedge);

    let cfg = PprConfig::default();

    // Snapshot at seq 10: hyperedge does not exist yet
    let res_seq10 = graph.personalized_page_rank_at(&[node_a], &cfg, 10).await.unwrap();
    let ids_seq10: Vec<_> = res_seq10.iter().map(|(id, _)| *id).collect();
    assert!(!ids_seq10.contains(&node_b), "node_b must NOT be reachable via hyperedge at seq 10");
    assert!(!ids_seq10.contains(&node_c), "node_c must NOT be reachable via hyperedge at seq 10");

    // Snapshot at seq 20: hyperedge is visible
    let res_seq20 = graph.personalized_page_rank_at(&[node_a], &cfg, 20).await.unwrap();
    let ids_seq20: Vec<_> = res_seq20.iter().map(|(id, _)| *id).collect();
    assert!(ids_seq20.contains(&node_b), "node_b must be reachable via hyperedge at seq 20");
    assert!(ids_seq20.contains(&node_c), "node_c must be reachable via hyperedge at seq 20");
}

#[tokio::test]
async fn test_ppr_snapshot_isolation_empty_seeds_and_determinism() {
    let graph = CsrGraph::new();
    let cfg = PprConfig::default();

    // Empty seeds must return empty result without panic
    let empty_res = graph.personalized_page_rank_at(&[], &cfg, 100).await.unwrap();
    assert!(empty_res.is_empty(), "Empty seed list must yield empty results");

    let node_a = EntityId::new(501);
    let node_b = EntityId::new(502);
    let tx10 = TxId::new(10);
    graph.add_entity(tx10, Entity::new(node_a, "a", "N")).await.unwrap();
    graph.add_entity(tx10, Entity::new(node_b, "b", "N")).await.unwrap();
    GraphIndex::add_edge(&graph, tx10, Edge::new(node_a, node_b, "rel").with_weight(1.0)).await.unwrap();
    graph.commit(tx10).await.unwrap();

    // Determinism test: 2 executions must return identical results
    let run1 = graph.personalized_page_rank_at(&[node_a], &cfg, 10).await.unwrap();
    let run2 = graph.personalized_page_rank_at(&[node_a], &cfg, 10).await.unwrap();
    assert_eq!(run1, run2, "Repeated snapshot PPR queries must be bit-identical");
}

#[tokio::test]
async fn test_ppr_snapshot_isolation_algorithms_parity() {
    let graph = CsrGraph::new();

    let node_a = EntityId::new(601);
    let node_b = EntityId::new(602);
    let node_c = EntityId::new(603);

    let tx10 = TxId::new(10);
    graph.add_entity(tx10, Entity::new(node_a, "a", "N")).await.unwrap();
    graph.add_entity(tx10, Entity::new(node_b, "b", "N")).await.unwrap();
    graph.add_entity(tx10, Entity::new(node_c, "c", "N")).await.unwrap();

    GraphIndex::add_edge(&graph, tx10, Edge::new(node_a, node_b, "rel").with_weight(0.8)).await.unwrap();
    GraphIndex::add_edge(&graph, tx10, Edge::new(node_b, node_c, "rel").with_weight(0.6)).await.unwrap();
    graph.commit(tx10).await.unwrap();

    let cfg_fp = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: PprAlgorithm::ForwardPush,
        warn_on_non_convergence: true,
    };

    let cfg_dense = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: PprAlgorithm::DensePowerIteration,
        warn_on_non_convergence: true,
    };

    let fp_res = graph.personalized_page_rank_at(&[node_a], &cfg_fp, 10).await.unwrap();
    let dense_res = graph.personalized_page_rank_at(&[node_a], &cfg_dense, 10).await.unwrap();

    let fp_ids: Vec<_> = fp_res.iter().map(|(id, _)| *id).collect();
    let dense_ids: Vec<_> = dense_res.iter().map(|(id, _)| *id).collect();

    assert_eq!(fp_ids, dense_ids, "ForwardPush and DensePowerIteration must produce the same candidate ordering on snapshot");
}
