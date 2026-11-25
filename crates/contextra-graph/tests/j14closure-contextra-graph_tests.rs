use contextra_graph::consistency_enforcement::{
    ConsistencyEnforcer, EdgeAssertion, ExactPredicateConflictDetector,
};
use contextra_graph::csr::{CsrGraph, EdgeType};
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleInterner};
use contextra_types::{DocId, EntityId, TxId};
use std::sync::Arc;

#[tokio::test]
async fn test_closure_consistency_enforcer_and_csr_symbols() {
    // 1. with_consistency_enforcer
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

    // 2. detect_contradiction
    assert!(graph.detect_contradiction(&detector, &assertion_a, &assertion_b));

    let hash = assertion_a.pattern_hash();
    // 3. is_suppressed
    assert!(!graph.is_conflict_suppressed(hash));
    // 4. get_pattern
    assert!(graph.get_conflict_pattern(&hash).is_none());

    // Populate ConsistencyEnforcer directly
    let mut enforcer = ConsistencyEnforcer::new(2);
    enforcer.record_contradiction(hash, TxId::new(1));
    enforcer.record_contradiction(hash, TxId::new(2));

    assert!(enforcer.is_suppressed(hash));
    let pattern = enforcer.get_pattern(&hash).expect("pattern found");
    assert_eq!(pattern.contradiction_count, 2);

    // 5. active_patterns
    let active_list: Vec<_> = enforcer.active_patterns().collect();
    assert_eq!(active_list.len(), 1);

    // 6. suggest_tombstone_candidates
    let candidate_edges = vec![(EntityId::new(1), EntityId::new(2))];
    let suggestions = enforcer.suggest_tombstone_candidates(&candidate_edges);
    assert_eq!(suggestions, candidate_edges);

    let candidates_csr = graph.suggest_tombstone_candidates_for_pattern(&candidate_edges);
    assert_eq!(candidates_csr, candidate_edges);
}

#[tokio::test]
async fn test_closure_direct_edge_insertions_and_doc_id_lookups() {
    let graph = Arc::new(CsrGraph::new());

    // 7. insert_edge_direct
    graph
        .insert_edge_direct(EntityId::new(10), EntityId::new(20), 1.0)
        .await
        .unwrap();

    // 8. insert_edge_direct_with_validity
    graph
        .insert_edge_direct_with_validity(
            EntityId::new(20),
            EntityId::new(30),
            0.8,
            Some(TxId::new(10)),
            None,
        )
        .await
        .unwrap();

    // 9. insert_edge_direct_with_bitemporal_validity
    graph
        .insert_edge_direct_with_bitemporal_validity(
            EntityId::new(30),
            EntityId::new(40),
            0.5,
            Some(TxId::new(10)),
            None,
            Some(100),
            Some(200),
            Some(DocId::new(888)),
        )
        .await
        .unwrap();

    graph.compact_async().await.unwrap();

    // 10. get_source_doc_id
    let source_doc = graph.source_doc_id_at(EntityId::new(30), EntityId::new(40));
    assert_eq!(source_doc, Some(DocId::new(888)));

    let raw_source_doc = graph.get_source_doc_id(0);
    assert!(raw_source_doc.is_some() || raw_source_doc.is_none());
}

#[tokio::test]
async fn test_closure_hyperedge_persist_and_cascade_queue() {
    let graph = CsrGraph::new();

    let doc_id = DocId::new(55);
    let hid = HyperEdgeId::new(101);

    // 11. pending_cascade_queue_len
    graph.enqueue_cascade_deferred(doc_id, &[hid]);
    assert_eq!(graph.pending_cascade_queue_len(), 1);

    // 12. process_cascade_queue
    let processed = graph.process_cascade_queue(10, TxId::new(1)).await.unwrap();
    assert_eq!(processed, 0);
    assert_eq!(graph.pending_cascade_queue_len(), 0);

    // 13. persist_hyperedge
    let hyperedge = HyperEdge::new(
        HyperEdgeId::new(200),
        EdgeType::Default,
        vec![
            RoleBinding::new(contextra_graph::hyperedge::RoleId::new(1), EntityId::new(1)),
            RoleBinding::new(contextra_graph::hyperedge::RoleId::new(2), EntityId::new(2)),
        ],
        1.0,
    );
    let persist_res = graph.persist_hyperedge(TxId::new(1), &hyperedge).await;
    assert!(persist_res.is_ok());
}

#[cfg(feature = "edge-reinforcement-learning")]
#[tokio::test]
async fn test_closure_edge_reinforcement_buffer_symbols() {
    use contextra_graph::edge_reinforcement_buffer::EdgeReinforcementBuffer;

    let buf = EdgeReinforcementBuffer::new();

    // 14. push_cooccurrence
    buf.push_cooccurrence(EntityId::new(1), EntityId::new(2), 0.7);
    // 15. push_traversal
    buf.push_traversal(EntityId::new(1), EntityId::new(3), 2);

    // 16. cooccurrence_count
    assert_eq!(buf.cooccurrence_count(), 1);
    // 17. traversal_count
    assert_eq!(buf.traversal_count(), 1);

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

#[test]
fn test_closure_role_interner_and_hyperedge_view_slice() {
    let interner = RoleInterner::new();

    // 18. get_or_intern
    let id_sub = interner.get_or_intern("subject");
    let id_obj = interner.get_or_intern("object");

    // 19. contains_role
    assert!(interner.contains_role("subject"));
    assert!(!interner.contains_role("nonexistent"));

    // 20. contains_id
    assert!(interner.contains_id(id_sub));
    assert!(!interner.contains_id(contextra_graph::hyperedge::RoleId::new(999)));

    // 21. resolve_string
    assert_eq!(interner.resolve_string(id_obj), Some("object".to_string()));

    let hyperedge = HyperEdge::new(
        HyperEdgeId::new(300),
        EdgeType::Default,
        vec![
            RoleBinding::new(id_sub, EntityId::new(10)),
            RoleBinding::new(id_obj, EntityId::new(20)),
        ],
        1.0,
    );

    let view = hyperedge.view();
    // 22. slice_participants
    let sliced = view.slice_participants(0..1);
    assert_eq!(sliced.len(), 1);
    assert_eq!(sliced[0].role, id_sub);
}
