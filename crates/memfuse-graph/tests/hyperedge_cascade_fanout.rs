use memfuse_core::{DocId, EntityId};
use memfuse_graph::cascade::{
    cascade_invalidate_hyperedges_for_superseded_doc, MAX_HYPEREDGE_CASCADE_FANOUT,
};
use memfuse_graph::csr::{CsrGraph, EdgeType};
use memfuse_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use std::time::Instant;

#[tokio::test]
async fn test_hyperedge_cascade_fanout_limit_and_termination() {
    let graph = CsrGraph::new();
    let doc_a = DocId::from_key("superseded-doc-1500").unwrap();

    const ROLE_1: RoleId = RoleId::new(1);
    const ROLE_2: RoleId = RoleId::new(2);

    // Create a high fanout scenario with 1,500 hyperedges derived from doc_a (> MAX_HYPEREDGE_CASCADE_FANOUT = 1,000)
    let total_hyperedges = 1_500;
    for i in 1..=total_hyperedges {
        let id = HyperEdgeId::new(i as u64);
        let bindings = vec![
            RoleBinding::new(ROLE_1, EntityId::new(i as u64 * 10)),
            RoleBinding::new(ROLE_2, EntityId::new(i as u64 * 10 + 1)),
        ];
        let he =
            HyperEdge::new(id, EdgeType::Default, bindings, 1.0).with_source_doc_id(Some(doc_a));
        graph.insert_hyperedge_direct(he);
    }

    assert_eq!(graph.hyperedges_for_doc(doc_a).len(), total_hyperedges);

    // Measure time to verify cascade invalidation terminates quickly (< 2 seconds)
    let start_time = Instant::now();
    let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 100)
        .await
        .expect("Cascade invalidation must succeed");
    let elapsed = start_time.elapsed();

    assert!(
        elapsed < std::time::Duration::from_millis(2000),
        "Cascade invalidation must terminate within 2000ms, took {:?}",
        elapsed
    );

    // Verify MAX_HYPEREDGE_CASCADE_FANOUT clamping and deferred counts
    assert_eq!(
        report.invalidated.len(),
        MAX_HYPEREDGE_CASCADE_FANOUT,
        "Synchronous invalidations must be clamped to MAX_HYPEREDGE_CASCADE_FANOUT"
    );
    assert_eq!(
        report.deferred.len(),
        total_hyperedges - MAX_HYPEREDGE_CASCADE_FANOUT,
        "Remaining hyperedges must be returned in deferred list"
    );
    assert_eq!(
        report.queued_for_background,
        total_hyperedges - MAX_HYPEREDGE_CASCADE_FANOUT,
        "Queued for background count must match deferred count"
    );

    // Verify idempotency on second run
    let report_second = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 101)
        .await
        .expect("Second run must succeed");

    assert_eq!(
        report_second.invalidated.len(),
        total_hyperedges - MAX_HYPEREDGE_CASCADE_FANOUT,
        "Second run processes previously deferred candidates"
    );
    assert!(
        report_second.deferred.is_empty(),
        "No remaining deferred hyperedges after second pass"
    );

    // Third run should be completely empty (idempotent)
    let report_third = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 102)
        .await
        .expect("Third run must succeed");
    assert!(report_third.invalidated.is_empty());
    assert!(report_third.deferred.is_empty());
}
