#![expect(clippy::unwrap_used)]

use memfuse_core::{DocId, EntityId, TxId};
use memfuse_graph::csr::{CsrGraph, EdgeType};
use memfuse_graph::hyperedge::{
    ConsolidationNodesGuard, HyperEdge, HyperEdgeId, RoleBinding, RoleId,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;

#[test]
fn test_hyperedge_compact_race() {
    let graph = Arc::new(CsrGraph::new());
    let doc_id = DocId::from_key("doc-race-test").unwrap();

    const ROLE_1: RoleId = RoleId::new(1);
    const ROLE_2: RoleId = RoleId::new(2);
    const ROLE_3: RoleId = RoleId::new(3);

    let num_threads = 4;
    let inserts_per_thread = 50;
    let total_inserts = num_threads * inserts_per_thread;

    // Spawn concurrent writer threads that acquire ConsolidationNodesGuard and insert hyperedges
    let mut handles = Vec::new();

    for t_idx in 0..num_threads {
        let graph_clone = Arc::clone(&graph);
        let handle = thread::spawn(move || {
            for i in 0..inserts_per_thread {
                let id_val = (t_idx * inserts_per_thread + i + 1) as u64;
                let he_id = HyperEdgeId::new(id_val);

                let e1 = EntityId::new(id_val * 10);
                let e2 = EntityId::new(id_val * 10 + 1);
                let e3 = EntityId::new(id_val * 10 + 2);

                // Use ConsolidationNodesGuard to enforce canonical lock ordering
                let guard = ConsolidationNodesGuard::acquire(&[e1, e2, e3]);

                let bindings = vec![
                    RoleBinding::new(ROLE_1, guard.entities()[0]),
                    RoleBinding::new(ROLE_2, guard.entities()[1]),
                    RoleBinding::new(ROLE_3, guard.entities()[2]),
                ];

                let he = HyperEdge::new(he_id, EdgeType::Default, bindings, 1.0)
                    .with_tx_validity(Some(TxId::new(1)), None)
                    .with_source_doc_id(Some(doc_id));

                graph_clone.insert_hyperedge(he);

                // Interleave graph compaction during concurrent inserts
                if i % 10 == 0 {
                    graph_clone.compact();
                }
            }
        });
        handles.push(handle);
    }

    // Spawn concurrent compactor thread
    let compactor_graph = Arc::clone(&graph);
    let compactor_handle = thread::spawn(move || {
        for _ in 0..20 {
            compactor_graph.compact();
            thread::sleep(std::time::Duration::from_millis(1));
        }
    });

    for handle in handles {
        handle.join().unwrap();
    }
    compactor_handle.join().unwrap();

    // Final compaction to settle CSR state
    graph.compact();

    // Assert that zero hyperedges were lost or duplicated
    let doc_hes = graph.hyperedges_for_doc(doc_id);
    assert_eq!(
        doc_hes.len(),
        total_inserts,
        "Expected exact total hyperedge count for document"
    );

    let doc_he_set: HashSet<_> = doc_hes.into_iter().collect();
    assert_eq!(
        doc_he_set.len(),
        total_inserts,
        "Expected no duplicate hyperedges in document index"
    );

    for id_val in 1..=(total_inserts as u64) {
        let he_id = HyperEdgeId::new(id_val);
        let he = graph
            .get_hyperedge(he_id)
            .unwrap_or_else(|| panic!("Missing hyperedge {he_id:?} after race compaction"));

        assert_eq!(he.id, he_id);
        assert_eq!(he.source_doc_id, Some(doc_id));

        // Check secondary entity index consistency
        for participant in he.participants.iter() {
            let entity_hes = graph.hyperedges_for_entity(participant.entity);
            assert!(
                entity_hes.contains(&he_id),
                "Entity {:?} index missing hyperedge {:?}",
                participant.entity,
                he_id
            );
        }
    }
}
