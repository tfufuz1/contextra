use contextra_graph::consistency_enforcement::{EdgeAssertion, ExactPredicateConflictDetector};
use contextra_graph::csr::{CsrGraph, EdgeType};
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding};
use contextra_types::{DocId, EntityId, TxId};
use std::sync::Arc;

#[tokio::test]
async fn test_wiring_consistency_enforcer_and_conflict_patterns() {
    let graph = CsrGraph::with_consistency_enforcer(2);
    let detector = ExactPredicateConflictDetector;

    let assertion_a = EdgeAssertion {
        subject: 1,
        predicate_hash: [10u8; 32],
        object_repr: b"value_a".to_vec(),
    };
    let assertion_b = EdgeAssertion {
        subject: 1,
        predicate_hash: [10u8; 32],
        object_repr: b"value_b".to_vec(),
    };

    assert!(graph.detect_contradiction(&detector, &assertion_a, &assertion_b));

    let hash = assertion_a.pattern_hash();
    assert!(!graph.is_conflict_suppressed(hash));
    assert!(graph.get_conflict_pattern(&hash).is_none());

    // Trigger conflict record in enforcer directly or via check
    if let Some(ref lock) = graph.consistency_enforcer {
        let mut enforcer = lock.write();
        enforcer.record_contradiction(hash, TxId::new(1));
        enforcer.record_contradiction(hash, TxId::new(2));
    }

    assert!(graph.is_conflict_suppressed(hash));
    let pattern = graph.get_conflict_pattern(&hash).expect("pattern found");
    assert_eq!(pattern.contradiction_count, 2);

    let active = graph.active_conflict_patterns();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].pattern_hash, hash);

    let candidates = vec![(EntityId::new(1), EntityId::new(2))];
    let suggestions = graph.suggest_tombstone_candidates_for_pattern(&candidates);
    assert_eq!(suggestions, candidates);
}

#[tokio::test]
async fn test_wiring_direct_edge_insertions_and_source_doc_lookup() {
    let graph = Arc::new(CsrGraph::new());

    // Test insert_edge_direct
    graph
        .insert_edge_direct(EntityId::new(1), EntityId::new(2), 1.0)
        .await
        .unwrap();

    // Test insert_edge_direct_with_validity
    graph
        .insert_edge_direct_with_validity(
            EntityId::new(2),
            EntityId::new(3),
            0.8,
            Some(TxId::new(10)),
            None,
        )
        .await
        .unwrap();

    // Test insert_edge_direct_with_bitemporal_validity
    graph
        .insert_edge_direct_with_bitemporal_validity(
            EntityId::new(3),
            EntityId::new(4),
            0.5,
            Some(TxId::new(10)),
            None,
            Some(100),
            Some(200),
            Some(DocId::new(999)),
        )
        .await
        .unwrap();

    // Force compaction so edges enter CSR arrays
    graph.compact_async().await.unwrap();

    // Test get_source_doc_id via source_doc_id_at
    let source_doc = graph.source_doc_id_at(EntityId::new(3), EntityId::new(4));
    assert_eq!(source_doc, Some(DocId::new(999)));
}

#[tokio::test]
async fn test_wiring_cascade_queue_processing() {
    let graph = CsrGraph::new();

    let doc_id = DocId::new(42);
    let hid = HyperEdgeId::new(101);

    graph.enqueue_cascade_deferred(doc_id, &[hid]);
    assert_eq!(graph.pending_cascade_queue_len(), 1);

    let processed = graph.process_cascade_queue(10, TxId::new(1)).await.unwrap();
    assert_eq!(processed, 0); // hyperedge 101 was not in hyperedges map
    assert_eq!(graph.pending_cascade_queue_len(), 0);

    // Test cascade draining during remove_doc
    graph.enqueue_cascade_deferred(doc_id, &[HyperEdgeId::new(202)]);
    assert_eq!(graph.pending_cascade_queue_len(), 1);

    let _ = graph.remove_doc(doc_id, TxId::new(2)).await.unwrap();
    assert_eq!(graph.pending_cascade_queue_len(), 0);
}

#[cfg(feature = "edge-reinforcement-learning")]
#[tokio::test]
async fn test_wiring_edge_reinforcement_buffer() {
    let graph = CsrGraph::new();

    graph.push_cooccurrence_signal(EntityId::new(10), EntityId::new(20), 0.9);
    graph.push_traversal_signal(EntityId::new(10), EntityId::new(30), 2);

    let (co, tr) = graph.reinforcement_signal_counts();
    assert_eq!(co, 1);
    assert_eq!(tr, 1);

    let config = contextra_graph::edge_reinforcement::EdgeReinforcementConfig::default();
    graph.flush_reinforcement_buffer(&config);

    let (co_after, tr_after) = graph.reinforcement_signal_counts();
    assert_eq!(co_after, 0);
    assert_eq!(tr_after, 0);
}

#[tokio::test]
async fn test_wiring_role_interner_and_hyperedge_views() {
    let graph = CsrGraph::new();

    assert!(!graph.contains_role("subject"));
    let role_id = graph.intern_role("subject");

    assert!(graph.contains_role("subject"));
    assert!(graph.contains_role_id(role_id));
    assert_eq!(
        graph.resolve_role_string(role_id),
        Some("subject".to_string())
    );

    let role_object = graph.intern_role("object");
    let role_predicate = graph.intern_role("predicate");

    let hyperedge = HyperEdge::new(
        HyperEdgeId::new(500),
        EdgeType::Default,
        vec![
            RoleBinding::new(role_id, EntityId::new(100)),
            RoleBinding::new(role_object, EntityId::new(200)),
            RoleBinding::new(role_predicate, EntityId::new(300)),
        ],
        1.0,
    );

    let view = hyperedge.view();

    let sliced = view.slice_participants(0..2);
    assert_eq!(sliced.len(), 2);

    let primary = view.primary_participants(2);
    assert_eq!(primary.len(), 2);
    assert_eq!(primary[0].role, role_id);
}
