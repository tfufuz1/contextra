use contextra_graph::cascade::cascade_invalidate_entity;
use contextra_graph::community::CommunityAssignment;
use contextra_graph::csr::{CsrGraph, EdgeType};
#[cfg(feature = "edge-reinforcement-learning")]
use contextra_graph::edge_reinforcement_buffer::EdgeReinforcementBuffer;
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_graph::residue_scan::scan_entity_residue;
use contextra_ports::GraphIndex;
use contextra_types::{Edge, Entity, EntityId, TxId};

#[tokio::test]
async fn test_entity_erasure_residue_full_cleanup() {
    let graph = CsrGraph::new();
    #[allow(deprecated)]
    let tx = TxId::new(1);

    let target_entity = EntityId::new(100);
    let neighbor_1 = EntityId::new(200);
    let neighbor_2 = EntityId::new(300);
    let unrelated_entity = EntityId::new(400);

    // 1. Setup Entities
    graph
        .add_entity(tx, Entity::new(target_entity, "Target", "Person"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(neighbor_1, "Neighbor1", "Person"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(neighbor_2, "Neighbor2", "Person"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(unrelated_entity, "Unrelated", "Person"))
        .await
        .unwrap();

    // 2. Setup Edges (CSR outgoing + incoming)
    graph
        .add_edge(tx, Edge::new(target_entity, neighbor_1, "knows"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(neighbor_2, target_entity, "reports_to"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(neighbor_1, neighbor_2, "collaborates"))
        .await
        .unwrap();

    graph.commit(tx).await.unwrap();

    // 3. Setup Hyperedge with RoleBindings
    let r1 = RoleId::new(1);
    let r2 = RoleId::new(2);
    let he_id = HyperEdgeId::new(101);
    let hyperedge = HyperEdge::new(
        he_id,
        EdgeType::Default,
        vec![
            RoleBinding::new(r1, target_entity),
            RoleBinding::new(r2, neighbor_1),
        ],
        1.0,
    );
    graph.insert_hyperedge_direct(hyperedge);

    // 4. Setup Community Assignment
    graph.set_communities_batch(&[
        CommunityAssignment {
            entity_id: target_entity,
            community_id: 42,
            hyperedges_included: false,
        },
        CommunityAssignment {
            entity_id: neighbor_1,
            community_id: 42,
            hyperedges_included: false,
        },
        CommunityAssignment {
            entity_id: neighbor_2,
            community_id: 42,
            hyperedges_included: false,
        },
    ]);

    // 5. Setup EdgeReinforcementBuffer
    #[cfg(feature = "edge-reinforcement-learning")]
    let buffer = EdgeReinforcementBuffer::new();
    #[cfg(feature = "edge-reinforcement-learning")]
    {
        buffer.push_cooccurrence(target_entity, neighbor_1, 0.9);
        buffer.push_traversal(neighbor_2, target_entity, 1);
        buffer.push_cooccurrence(neighbor_1, neighbor_2, 0.5);
    }

    // Verify residue scan before deletion
    let report_before = scan_entity_residue(
        &graph,
        target_entity,
        #[cfg(feature = "edge-reinforcement-learning")]
        Some(&buffer),
        None,
    );
    assert!(
        !report_before.is_empty(),
        "Residue report before erasure must not be empty"
    );
    assert_eq!(report_before.outgoing_edges.len(), 1);
    assert_eq!(report_before.incoming_edges.len(), 1);
    assert_eq!(report_before.hyperedges.len(), 1);
    assert_eq!(report_before.community_assigned, Some(42));
    #[cfg(feature = "edge-reinforcement-learning")]
    assert_eq!(report_before.reinforcement_buffer_signals, 2);

    // 6. Execute Entity Cascade Invalidation
    let cascade_report = cascade_invalidate_entity(
        &graph,
        target_entity,
        2,
        #[cfg(feature = "edge-reinforcement-learning")]
        Some(&buffer),
    )
    .await
    .unwrap();

    assert_eq!(cascade_report.entity_id, target_entity);
    assert!(cascade_report.community_removed);
    assert_eq!(cascade_report.tombstoned_hyperedges, vec![he_id]);
    #[cfg(feature = "edge-reinforcement-learning")]
    assert_eq!(cascade_report.buffer_signals_purged, 2);

    // 7. Verify residue scan after deletion is EMPTY
    let report_after = scan_entity_residue(
        &graph,
        target_entity,
        #[cfg(feature = "edge-reinforcement-learning")]
        Some(&buffer),
        None,
    );
    assert!(
        report_after.is_empty(),
        "Residue report after cascade entity invalidation must be empty: {:?}",
        report_after
    );
}

#[tokio::test]
async fn test_entity_residue_scan_triggers_on_injected_buffer_leftover() {
    let graph = CsrGraph::new();
    let target = EntityId::new(999);

    #[cfg(feature = "edge-reinforcement-learning")]
    let buffer = EdgeReinforcementBuffer::new();
    #[cfg(feature = "edge-reinforcement-learning")]
    buffer.push_cooccurrence(target, EntityId::new(1000), 0.7);

    let report = scan_entity_residue(
        &graph,
        target,
        #[cfg(feature = "edge-reinforcement-learning")]
        Some(&buffer),
        None,
    );

    #[cfg(feature = "edge-reinforcement-learning")]
    {
        assert!(
            !report.is_empty(),
            "Scan must trigger on leftover buffer signal"
        );
        assert_eq!(report.reinforcement_buffer_signals, 1);
    }
    #[cfg(not(feature = "edge-reinforcement-learning"))]
    {
        assert!(report.is_empty());
    }
}

#[tokio::test]
async fn test_neighbors_and_unrelated_entities_remain_unaffected() {
    let graph = CsrGraph::new();
    let ref_graph = CsrGraph::new();
    #[allow(deprecated)]
    let tx = TxId::new(1);

    let target = EntityId::new(10);
    let n1 = EntityId::new(20);
    let n2 = EntityId::new(30);

    // Build populated graph
    for e in [target, n1, n2] {
        graph
            .add_entity(tx, Entity::new(e, "E", "NodeType"))
            .await
            .unwrap();
    }
    graph
        .add_edge(tx, Edge::new(target, n1, "rel1"))
        .await
        .unwrap();
    graph.add_edge(tx, Edge::new(n1, n2, "rel2")).await.unwrap();
    graph.commit(tx).await.unwrap();

    // Build reference graph WITHOUT target
    for e in [n1, n2] {
        ref_graph
            .add_entity(tx, Entity::new(e, "E", "NodeType"))
            .await
            .unwrap();
    }
    ref_graph
        .add_edge(tx, Edge::new(n1, n2, "rel2"))
        .await
        .unwrap();
    ref_graph.commit(tx).await.unwrap();

    // Invalidate target in graph
    cascade_invalidate_entity(
        &graph,
        target,
        2,
        #[cfg(feature = "edge-reinforcement-learning")]
        None,
    )
    .await
    .unwrap();

    // Verify n1 -> n2 relation is intact in both
    let neighbors_n1_graph = graph.neighbors(n1).await.unwrap();
    let neighbors_n1_ref = ref_graph.neighbors(n1).await.unwrap();

    assert_eq!(neighbors_n1_graph, vec![n2]);
    assert_eq!(neighbors_n1_ref, vec![n2]);
}

#[tokio::test]
async fn test_no_global_community_recalculation_on_entity_deletion() {
    let graph = CsrGraph::new();
    #[allow(deprecated)]
    let tx = TxId::new(1);

    let target = EntityId::new(100);
    let neighbor = EntityId::new(200);

    graph
        .add_entity(tx, Entity::new(target, "T", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(neighbor, "N", "Node"))
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    graph.set_communities_batch(&[
        CommunityAssignment {
            entity_id: target,
            community_id: 5,
            hyperedges_included: false,
        },
        CommunityAssignment {
            entity_id: neighbor,
            community_id: 5,
            hyperedges_included: false,
        },
    ]);

    cascade_invalidate_entity(
        &graph,
        target,
        2,
        #[cfg(feature = "edge-reinforcement-learning")]
        None,
    )
    .await
    .unwrap();

    let report_target = scan_entity_residue(
        &graph,
        target,
        #[cfg(feature = "edge-reinforcement-learning")]
        None,
        None,
    );
    let report_neighbor = scan_entity_residue(
        &graph,
        neighbor,
        #[cfg(feature = "edge-reinforcement-learning")]
        None,
        None,
    );

    assert_eq!(
        report_target.community_assigned, None,
        "Target entity community assignment must be removed"
    );
    assert_eq!(
        report_neighbor.community_assigned,
        Some(5),
        "Neighbor entity community assignment must remain untouched without global rebuild"
    );
}
