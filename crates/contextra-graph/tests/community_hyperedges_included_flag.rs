use contextra_types::{Edge, Entity, EntityId, TxId};
use contextra_ports::GraphIndex;
use contextra_graph::community::{detect_communities, CommunityDetectionConfig};
use contextra_graph::csr::{CsrGraph, EdgeType};
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use std::collections::HashMap;

#[tokio::test]
async fn test_community_hyperedges_included_flag_effect() {
    let graph = CsrGraph::new();
    let tx1 = TxId::new(1);

    // Build Cluster A: Entities 1, 2, 3 with dense internal binary edges
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);

    // Build Cluster B: Entities 10, 11, 12 with dense internal binary edges
    let e10 = EntityId::new(10);
    let e11 = EntityId::new(11);
    let e12 = EntityId::new(12);

    for &eid in &[e1, e2, e3, e10, e11, e12] {
        GraphIndex::add_entity(
            &graph,
            tx1,
            Entity::new(eid, format!("Node-{}", eid.inner()), "TypeA"),
        )
        .await
        .unwrap();
    }

    // Binary edges within Cluster A
    GraphIndex::add_edge(&graph, tx1, Edge::new(e1, e2, "rel"))
        .await
        .unwrap();
    GraphIndex::add_edge(&graph, tx1, Edge::new(e2, e3, "rel"))
        .await
        .unwrap();
    GraphIndex::add_edge(&graph, tx1, Edge::new(e1, e3, "rel"))
        .await
        .unwrap();

    // Binary edges within Cluster B
    GraphIndex::add_edge(&graph, tx1, Edge::new(e10, e11, "rel"))
        .await
        .unwrap();
    GraphIndex::add_edge(&graph, tx1, Edge::new(e11, e12, "rel"))
        .await
        .unwrap();
    GraphIndex::add_edge(&graph, tx1, Edge::new(e10, e12, "rel"))
        .await
        .unwrap();

    GraphIndex::commit(&graph, tx1).await.unwrap();

    // Add a strong hyperedge bridging Cluster A (e1, e2) and Cluster B (e10, e11)
    const ROLE_1: RoleId = RoleId::new(1);
    const ROLE_2: RoleId = RoleId::new(2);
    const ROLE_3: RoleId = RoleId::new(3);
    const ROLE_4: RoleId = RoleId::new(4);

    let bridge_hyperedge = HyperEdge::new(
        HyperEdgeId::new(999),
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, e1),
            RoleBinding::new(ROLE_2, e2),
            RoleBinding::new(ROLE_3, e10),
            RoleBinding::new(ROLE_4, e11),
        ],
        10.0, // High weight to connect the two clusters when hyperedges are included
    );
    graph.insert_hyperedge_direct(bridge_hyperedge);

    // 1. Run community detection with hyperedges_included = false
    let config_excluded = CommunityDetectionConfig {
        max_iterations: 100,
        seed: 42,
        hyperedges_included: false,
    };

    let assignments_excluded = detect_communities(&graph, &config_excluded)
        .await
        .expect("Community detection without hyperedges must succeed");

    assert_eq!(assignments_excluded.len(), 6);
    for assign in &assignments_excluded {
        assert!(
            !assign.hyperedges_included,
            "hyperedges_included flag in assignment must match config false"
        );
    }

    let map_excluded: HashMap<EntityId, u64> = assignments_excluded
        .into_iter()
        .map(|a| (a.entity_id, a.community_id))
        .collect();

    // With hyperedges excluded, Cluster A (1, 2, 3) and Cluster B (10, 11, 12) must be in separate communities
    assert_eq!(map_excluded.get(&e1), map_excluded.get(&e2));
    assert_eq!(map_excluded.get(&e1), map_excluded.get(&e3));

    assert_eq!(map_excluded.get(&e10), map_excluded.get(&e11));
    assert_eq!(map_excluded.get(&e10), map_excluded.get(&e12));

    assert_ne!(
        map_excluded.get(&e1),
        map_excluded.get(&e10),
        "Cluster A and Cluster B must belong to different communities when hyperedges are excluded"
    );

    // 2. Run community detection with hyperedges_included = true
    let config_included = CommunityDetectionConfig {
        max_iterations: 100,
        seed: 42,
        hyperedges_included: true,
    };

    let assignments_included = detect_communities(&graph, &config_included)
        .await
        .expect("Community detection with hyperedges must succeed");

    assert_eq!(assignments_included.len(), 6);
    for assign in &assignments_included {
        assert!(
            assign.hyperedges_included,
            "hyperedges_included flag in assignment must match config true"
        );
    }

    let map_included: HashMap<EntityId, u64> = assignments_included
        .into_iter()
        .map(|a| (a.entity_id, a.community_id))
        .collect();

    // With hyperedges included, the strong hyperedge bridges the clusters via star expansion
    assert_eq!(
        map_included.get(&e1),
        map_included.get(&e10),
        "Cluster A and Cluster B must merge into a single community when hyperedges are included"
    );
}
