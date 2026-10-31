#![allow(clippy::unwrap_used)]

use contextra_graph::cascade::cascade_invalidate_hyperedges_for_superseded_doc;
use contextra_graph::csr::{CsrGraph, EdgeType};
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_types::{DocId, EntityId, TxId};

#[tokio::test]
async fn test_leanrag_cascade_recursive_through_superedges() {
    let graph = CsrGraph::new();
    let doc_a = DocId::from_key("doc-a").unwrap();

    const ROLE_1: RoleId = RoleId::new(1);
    const ROLE_2: RoleId = RoleId::new(2);

    let e1 = HyperEdge::new(
        HyperEdgeId::new(10),
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(100)),
            RoleBinding::new(ROLE_2, EntityId::new(101)),
        ],
        1.0,
    )
    .with_source_doc_id(Some(doc_a));

    let e2 = HyperEdge::new(
        HyperEdgeId::new(11),
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(101)),
            RoleBinding::new(ROLE_2, EntityId::new(102)),
        ],
        1.0,
    )
    .with_source_doc_id(Some(doc_a));

    let super_s = HyperEdge::new(
        HyperEdgeId::new(1000),
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(100)),
            RoleBinding::new(ROLE_2, EntityId::new(102)),
        ],
        0.5,
    )
    .with_child_edge_ids(vec![HyperEdgeId::new(10), HyperEdgeId::new(11)]);

    let super_s2 = HyperEdge::new(
        HyperEdgeId::new(2000),
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(100)),
            RoleBinding::new(ROLE_2, EntityId::new(102)),
        ],
        0.25,
    )
    .with_child_edge_ids(vec![HyperEdgeId::new(1000)]);

    graph.insert_hyperedge_direct(e1);
    graph.insert_hyperedge_direct(e2);
    graph.insert_hyperedge_direct(super_s);
    graph.insert_hyperedge_direct(super_s2);

    // Verify all 4 are alive before cascade
    assert!(graph.get_hyperedge(HyperEdgeId::new(10)).is_some());
    assert!(graph.get_hyperedge(HyperEdgeId::new(11)).is_some());
    assert!(graph.get_hyperedge(HyperEdgeId::new(1000)).is_some());
    assert!(graph.get_hyperedge(HyperEdgeId::new(2000)).is_some());

    let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 2)
        .await
        .unwrap();

    // Verify all 4 are tombstoned
    assert_eq!(report.invalidated.len(), 4);
    assert!(report.deferred.is_empty());

    assert!(graph.get_hyperedge(HyperEdgeId::new(10)).is_none());
    assert!(graph.get_hyperedge(HyperEdgeId::new(11)).is_none());
    assert!(graph.get_hyperedge(HyperEdgeId::new(1000)).is_none());
    assert!(graph.get_hyperedge(HyperEdgeId::new(2000)).is_none());

    assert!(graph.hyperedges_for_entity(EntityId::new(100)).is_empty());

    // Second call is idempotent
    let report2 = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 3)
        .await
        .unwrap();
    assert!(report2.invalidated.is_empty());
    assert!(report2.deferred.is_empty());
}

#[tokio::test]
async fn test_leanrag_cascade_conservative_gdpr_multidoc_superedge() {
    let graph = CsrGraph::new();
    let doc_a = DocId::from_key("doc-a").unwrap();
    let doc_b = DocId::from_key("doc-b").unwrap();

    const ROLE_1: RoleId = RoleId::new(1);
    const ROLE_2: RoleId = RoleId::new(2);

    let e_a = HyperEdge::new(
        HyperEdgeId::new(1),
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(10)),
            RoleBinding::new(ROLE_2, EntityId::new(20)),
        ],
        1.0,
    )
    .with_source_doc_id(Some(doc_a));

    let e_b = HyperEdge::new(
        HyperEdgeId::new(2),
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(20)),
            RoleBinding::new(ROLE_2, EntityId::new(30)),
        ],
        1.0,
    )
    .with_source_doc_id(Some(doc_b));

    let super_ab = HyperEdge::new(
        HyperEdgeId::new(100),
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(10)),
            RoleBinding::new(ROLE_2, EntityId::new(30)),
        ],
        0.5,
    )
    .with_child_edge_ids(vec![HyperEdgeId::new(1), HyperEdgeId::new(2)]);

    graph.insert_hyperedge_direct(e_a);
    graph.insert_hyperedge_direct(e_b);
    graph.insert_hyperedge_direct(super_ab);

    let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 2)
        .await
        .unwrap();

    // Super-edge (100) and e_a (1) tombstoned, e_b (2) remains alive
    assert!(graph.get_hyperedge(HyperEdgeId::new(1)).is_none());
    assert!(graph.get_hyperedge(HyperEdgeId::new(100)).is_none());
    assert!(graph.get_hyperedge(HyperEdgeId::new(2)).is_some());

    assert_eq!(report.invalidated.len(), 2);
    assert!(report.invalidated.contains(&HyperEdgeId::new(1)));
    assert!(report.invalidated.contains(&HyperEdgeId::new(100)));
}

#[tokio::test]
async fn test_leanrag_cascade_fanout_limit_and_background_queue() {
    let graph = CsrGraph::new();
    let doc_a = DocId::from_key("doc-a").unwrap();

    const ROLE_1: RoleId = RoleId::new(1);
    const ROLE_2: RoleId = RoleId::new(2);

    let mut child_ids = Vec::new();
    for i in 1..=1_001 {
        let hid = HyperEdgeId::new(i as u64);
        child_ids.push(hid);
        let he = HyperEdge::new(
            hid,
            EdgeType::Default,
            vec![
                RoleBinding::new(ROLE_1, EntityId::new(1)),
                RoleBinding::new(ROLE_2, EntityId::new(2)),
            ],
            1.0,
        )
        .with_source_doc_id(Some(doc_a));
        graph.insert_hyperedge_direct(he);
    }

    let super_edge = HyperEdge::new(
        HyperEdgeId::new(999_999),
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(1)),
            RoleBinding::new(ROLE_2, EntityId::new(2)),
        ],
        0.5,
    )
    .with_child_edge_ids(child_ids);

    graph.insert_hyperedge_direct(super_edge);

    let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 2)
        .await
        .unwrap();

    // Total nodes = 1,001 children + 1 super-edge = 1,002
    assert_eq!(report.invalidated.len(), 1_000);
    assert_eq!(report.deferred.len(), 2);
    assert_eq!(report.queued_for_background, 2);

    // Super-edge (999_999) was tombstoned synchronously (topologically before children)
    assert!(report.invalidated.contains(&HyperEdgeId::new(999_999)));
    assert!(graph.get_hyperedge(HyperEdgeId::new(999_999)).is_none());

    // Process background queue
    let processed = graph.process_cascade_queue(10, TxId::new(3)).await.unwrap();
    assert_eq!(processed, 2);
    assert_eq!(graph.pending_cascade_queue_len(), 0);
}

#[tokio::test]
async fn test_leanrag_cascade_cycle_termination() {
    let graph = CsrGraph::new();
    let doc_a = DocId::from_key("doc-a").unwrap();

    const ROLE_1: RoleId = RoleId::new(1);
    const ROLE_2: RoleId = RoleId::new(2);

    let s1_id = HyperEdgeId::new(101);
    let s2_id = HyperEdgeId::new(102);

    let s1 = HyperEdge::new(
        s1_id,
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(1)),
            RoleBinding::new(ROLE_2, EntityId::new(2)),
        ],
        1.0,
    )
    .with_source_doc_id(Some(doc_a))
    .with_child_edge_ids(vec![s2_id]);

    let s2 = HyperEdge::new(
        s2_id,
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(2)),
            RoleBinding::new(ROLE_2, EntityId::new(3)),
        ],
        1.0,
    )
    .with_child_edge_ids(vec![s1_id]);

    graph.insert_hyperedge_direct(s1);
    graph.insert_hyperedge_direct(s2);

    let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 2)
        .await
        .unwrap();

    assert_eq!(report.invalidated.len(), 2);
    assert!(graph.get_hyperedge(s1_id).is_none());
    assert!(graph.get_hyperedge(s2_id).is_none());
}

#[tokio::test]
async fn test_leanrag_cascade_topological_ancestor_before_child_order() {
    let graph = CsrGraph::new();
    let doc_a = DocId::from_key("doc-a").unwrap();

    const ROLE_1: RoleId = RoleId::new(1);
    const ROLE_2: RoleId = RoleId::new(2);

    let child_id = HyperEdgeId::new(50);
    let parent_id = HyperEdgeId::new(10);
    let grand_parent_id = HyperEdgeId::new(5);

    let child = HyperEdge::new(
        child_id,
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(1)),
            RoleBinding::new(ROLE_2, EntityId::new(2)),
        ],
        1.0,
    )
    .with_source_doc_id(Some(doc_a));

    let parent = HyperEdge::new(
        parent_id,
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(1)),
            RoleBinding::new(ROLE_2, EntityId::new(2)),
        ],
        0.5,
    )
    .with_child_edge_ids(vec![child_id]);

    let grand_parent = HyperEdge::new(
        grand_parent_id,
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_1, EntityId::new(1)),
            RoleBinding::new(ROLE_2, EntityId::new(2)),
        ],
        0.25,
    )
    .with_child_edge_ids(vec![parent_id]);

    graph.insert_hyperedge_direct(child);
    graph.insert_hyperedge_direct(parent);
    graph.insert_hyperedge_direct(grand_parent);

    let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 2)
        .await
        .unwrap();

    assert_eq!(
        report.invalidated,
        vec![grand_parent_id, parent_id, child_id],
        "Ancestors must be tombstoned before their children"
    );
}

#[test]
fn test_leanrag_legacy_bincode_deserialization() {
    let edge_id = HyperEdgeId::new(42);
    let edge = HyperEdge::new(
        edge_id,
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(10)),
            RoleBinding::new(RoleId::new(2), EntityId::new(20)),
        ],
        1.0,
    );

    let bytes = edge.serialize().unwrap();
    let deserialized = HyperEdge::deserialize(&bytes).unwrap();
    assert_eq!(edge, deserialized);
    assert!(deserialized.child_edge_ids.is_empty());
}
