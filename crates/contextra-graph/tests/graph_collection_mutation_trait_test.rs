use contextra_graph::CsrGraph;
use contextra_ports::graph::{
    GraphCollectionMutation, GraphMutationError, HyperEdgeId, RoleBinding, RoleId,
};
use contextra_types::{DocId, EntityId};
use std::sync::Arc;

#[test]
fn test_csr_graph_collection_mutation_via_dyn_trait() {
    let graph = Arc::new(CsrGraph::new());
    let dyn_mutator: Arc<dyn GraphCollectionMutation> = graph.clone();

    let doc_id = DocId::from(100u64);
    let entity1 = EntityId::from(10u64);
    let entity2 = EntityId::from(20u64);
    let entity3 = EntityId::from(30u64);

    let bindings = vec![
        RoleBinding::new(RoleId::new(1), entity1),
        RoleBinding::new(RoleId::new(2), entity2),
        RoleBinding::new(RoleId::new(3), entity3),
    ];

    // 1. Successful mutation through dyn GraphCollectionMutation trait
    let predicate_tag = 42u32;
    let res = dyn_mutator.relate_n_ary(predicate_tag, &bindings, doc_id);
    assert!(res.is_ok());

    let hyperedge_id = res.unwrap();
    assert_eq!(hyperedge_id, HyperEdgeId::new(1));

    // 2. Post-mutation consistency checks directly on the CsrGraph instance
    let stored_hyperedge = graph
        .get_hyperedge(contextra_graph::hyperedge::HyperEdgeId::new(
            hyperedge_id.inner(),
        ))
        .expect("Hyperedge should be present in CsrGraph after trait mutation");

    assert_eq!(stored_hyperedge.id.inner(), hyperedge_id.inner());
    assert_eq!(stored_hyperedge.participants.len(), 3);
    assert_eq!(stored_hyperedge.source_doc_id, Some(doc_id));
    assert_eq!(stored_hyperedge.weight, 1.0);

    // Verify entity index lookup on the graph
    let entity1_hyperedges = graph.hyperedges_for_entity(entity1);
    assert_eq!(entity1_hyperedges.len(), 1);
    assert_eq!(entity1_hyperedges[0].inner(), hyperedge_id.inner());

    // Verify doc_id lookup on the graph
    let doc_hyperedges = graph.hyperedges_for_doc(doc_id);
    assert_eq!(doc_hyperedges.len(), 1);
    assert_eq!(doc_hyperedges[0].inner(), hyperedge_id.inner());

    // 3. Invalid mutation through trait (< 2 participants) expecting InsufficientParticipants error
    let invalid_bindings = vec![RoleBinding::new(RoleId::new(1), entity1)];
    let err_res = dyn_mutator.relate_n_ary(predicate_tag, &invalid_bindings, doc_id);

    assert!(matches!(
        err_res,
        Err(GraphMutationError::InsufficientParticipants {
            expected: 2,
            found: 1
        })
    ));

    // 4. Second successful mutation to test auto-incrementing hyperedge IDs
    let bindings2 = vec![
        RoleBinding::new(RoleId::new(1), entity2),
        RoleBinding::new(RoleId::new(2), entity3),
    ];
    let res2 = dyn_mutator.relate_n_ary(predicate_tag, &bindings2, doc_id);
    assert!(res2.is_ok());
    let hyperedge_id2 = res2.unwrap();
    assert_eq!(hyperedge_id2, HyperEdgeId::new(2));
}
