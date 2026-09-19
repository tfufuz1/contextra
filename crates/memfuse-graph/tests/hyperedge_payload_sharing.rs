#![expect(clippy::expect_used)]

use memfuse_core::{EntityId, TxId};
use memfuse_graph::csr::{CsrGraph, EdgeType};
use memfuse_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use std::sync::Arc;

#[test]
fn test_hyperedge_participants_payload_sharing() {
    let bindings = vec![
        RoleBinding::new(RoleId::new(1), EntityId::new(100)),
        RoleBinding::new(RoleId::new(2), EntityId::new(200)),
        RoleBinding::new(RoleId::new(3), EntityId::new(300)),
    ];

    let edge1 = HyperEdge::new(HyperEdgeId::new(1), EdgeType::Default, bindings, 1.0);
    let edge2 = edge1.clone();

    // Cloning a HyperEdge shares the participants Arc without re-allocating RoleBindings
    assert!(Arc::ptr_eq(&edge1.participants, &edge2.participants));
    assert_eq!(Arc::strong_count(&edge1.participants), 2);
}

#[test]
fn test_csr_graph_hyperedge_payload_sharing_on_rcu_clone() {
    let graph = CsrGraph::new();
    let id = HyperEdgeId::new(42);

    let bindings = vec![
        RoleBinding::new(RoleId::new(10), EntityId::new(1)),
        RoleBinding::new(RoleId::new(20), EntityId::new(2)),
    ];

    let edge = HyperEdge::new(id, EdgeType::Default, bindings, 1.0);
    graph.insert_hyperedge_direct(edge);

    let retrieved1 = graph.get_hyperedge(id).expect("Hyperedge must exist");
    let retrieved2 = graph.get_hyperedge(id).expect("Hyperedge must exist");

    // Multiple lookups return Arcs pointing to the same underlying HyperEdge allocation
    assert!(Arc::ptr_eq(&retrieved1, &retrieved2));
    assert!(Arc::ptr_eq(
        &retrieved1.participants,
        &retrieved2.participants
    ));

    // Tombstoning creates a new HyperEdge Arc but reuses the participants Arc slice
    graph.tombstone_hyperedge(id, TxId::new(10));
    assert_eq!(retrieved1.participants.len(), 2);
}
