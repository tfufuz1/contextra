use contextra_core::{EntityId, TxId};
use contextra_graph::csr::EdgeType;
use contextra_graph::error::GraphMutationError;
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_graph::CsrGraph;
use std::sync::Arc;

#[test]
fn test_batch_commit_single_rcu_publish_and_atomic_rollback() {
    let graph = CsrGraph::new();

    // 1. Insert 3 initial hyperedges
    let h1 = HyperEdge::new(
        HyperEdgeId::new(1),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(10)),
            RoleBinding::new(RoleId::new(2), EntityId::new(20)),
        ],
        1.0,
    );
    let h2 = HyperEdge::new(
        HyperEdgeId::new(2),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(20)),
            RoleBinding::new(RoleId::new(2), EntityId::new(30)),
        ],
        1.0,
    );
    let h3 = HyperEdge::new(
        HyperEdgeId::new(3),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(30)),
            RoleBinding::new(RoleId::new(2), EntityId::new(40)),
        ],
        1.0,
    );

    graph.insert_hyperedge_direct(h1);
    graph.insert_hyperedge_direct(h2);
    graph.insert_hyperedge_direct(h3);

    // Capture inner snapshot pointer before batch commit
    let ptr_before = Arc::as_ptr(&graph.inner_read());

    // 2. Prepare batch commit: 3 tombstones + 1 superedge
    let super_edge = HyperEdge::new(
        HyperEdgeId::new(10),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(0), EntityId::new(10)),
            RoleBinding::new(RoleId::new(0), EntityId::new(20)),
            RoleBinding::new(RoleId::new(0), EntityId::new(30)),
            RoleBinding::new(RoleId::new(0), EntityId::new(40)),
        ],
        0.8,
    )
    .with_child_edge_ids(vec![
        HyperEdgeId::new(1),
        HyperEdgeId::new(2),
        HyperEdgeId::new(3),
    ]);

    let tombstone_ids = vec![
        HyperEdgeId::new(1),
        HyperEdgeId::new(2),
        HyperEdgeId::new(3),
    ];
    let wal_tx = TxId::new(100);

    let res = graph.commit_super_edge_batch(&tombstone_ids, vec![super_edge.clone()], wal_tx);
    assert!(res.is_ok(), "Batch commit should succeed");

    // Capture inner snapshot pointer after batch commit
    let ptr_after = Arc::as_ptr(&graph.inner_read());

    // Verify pointer changed (exact one RCU publish took place for all 3 tombstones + 1 superedge)
    assert_ne!(
        ptr_before, ptr_after,
        "Snapshot pointer must change after batch commit publish"
    );

    // Verify state: all 3 edges tombstoned, 1 superedge active
    assert!(
        graph.get_hyperedge(HyperEdgeId::new(1)).is_none(),
        "H1 must be tombstoned"
    );
    assert!(
        graph.get_hyperedge(HyperEdgeId::new(2)).is_none(),
        "H2 must be tombstoned"
    );
    assert!(
        graph.get_hyperedge(HyperEdgeId::new(3)).is_none(),
        "H3 must be tombstoned"
    );

    let retrieved_super = graph.get_hyperedge(HyperEdgeId::new(10));
    assert!(retrieved_super.is_some(), "Superedge must exist");
    if let Some(super_edge) = retrieved_super {
        assert_eq!(super_edge.id, HyperEdgeId::new(10));
        assert_eq!(
            *super_edge.child_edge_ids,
            [HyperEdgeId::new(1), HyperEdgeId::new(2), HyperEdgeId::new(3)]
        );
    }

    // 3. Test ID collision all-or-nothing rollback
    let colliding_edge = HyperEdge::new(
        HyperEdgeId::new(10), // Collides with existing superedge 10
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(0), EntityId::new(50)),
            RoleBinding::new(RoleId::new(0), EntityId::new(60)),
        ],
        0.5,
    );

    // Insert new hyperedge 4 to try tombstoning during failed batch
    let h4 = HyperEdge::new(
        HyperEdgeId::new(4),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(50)),
            RoleBinding::new(RoleId::new(2), EntityId::new(60)),
        ],
        1.0,
    );
    graph.insert_hyperedge_direct(h4);

    let collision_res = graph.commit_super_edge_batch(
        &[HyperEdgeId::new(4)],
        vec![colliding_edge],
        TxId::new(101),
    );

    assert_eq!(
        collision_res,
        Err(GraphMutationError::DuplicateHyperEdgeId(HyperEdgeId::new(10))),
        "Must return DuplicateHyperEdgeId error"
    );

    // Verify H4 was NOT tombstoned (all-or-nothing semantics: no mutations applied)
    assert!(
        graph.get_hyperedge(HyperEdgeId::new(4)).is_some(),
        "H4 must remain active because batch commit failed on collision pre-check"
    );
}
