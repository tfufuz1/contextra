use memfuse_core::EntityId;
use memfuse_graph::csr::EdgeType;
use memfuse_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use std::sync::Arc;

#[test]
fn test_hyperedge_payload_sharing_arc_ptr_eq() {
    let bindings: Arc<[RoleBinding]> = Arc::from(vec![
        RoleBinding::new(RoleId::new(1), EntityId::new(101)),
        RoleBinding::new(RoleId::new(2), EntityId::new(102)),
        RoleBinding::new(RoleId::new(3), EntityId::new(103)),
    ]);

    let edge1 = HyperEdge::new(
        HyperEdgeId::new(1001),
        EdgeType::Default,
        Arc::clone(&bindings),
        1.0,
    );

    let edge2 = HyperEdge::new(
        HyperEdgeId::new(1002),
        EdgeType::Default,
        Arc::clone(&bindings),
        2.5,
    );

    // SPECS: Verify that both hyperedges reference the exact same memory allocation for role bindings
    assert!(
        Arc::ptr_eq(&edge1.participants, &edge2.participants),
        "HyperEdge participants must share identical memory heap allocation via Arc::ptr_eq"
    );

    assert_eq!(edge1.participants.len(), 3);
    assert_eq!(edge2.participants.len(), 3);
    assert_eq!(edge1.participants[0], bindings[0]);
}
