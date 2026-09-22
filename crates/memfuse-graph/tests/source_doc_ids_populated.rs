use memfuse_core::{DocId, Edge, Entity, EntityId, GraphIndex, TxId};
use memfuse_graph::CsrGraph;
use std::sync::Arc;

#[tokio::test]
async fn proof_source_doc_ids_set_after_compact() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::new(1);

    // Add 11 entities for 10 sequential edges
    for i in 0..=10 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(i), format!("Node{i}"), "Type"),
            )
            .await
            .unwrap();
    }

    // Add 10 edges with distinct non-zero source_doc_ids
    for i in 0..10 {
        let id_a = EntityId::new(i);
        let id_b = EntityId::new(i + 1);
        let doc_id = DocId::new(i + 100);
        let edge = Edge::new(id_a, id_b, "relates").with_source_doc_id(doc_id);
        GraphIndex::add_edge(graph.as_ref(), tx, edge)
            .await
            .unwrap();
    }
    graph.commit(tx).await.unwrap();

    // Compact to populate CSR array source_doc_ids
    graph.compact();

    // For each edge index 0..10: get_source_doc_id(idx) must be Some(DocId::new(i + 100))
    for i in 0..10 {
        let idx = i as usize;
        let expected_doc = DocId::new(i + 100);
        assert_eq!(
            graph.get_source_doc_id(idx),
            Some(expected_doc),
            "source_doc_ids at index {idx} must be Some({expected_doc:?}) after compact"
        );
        let id_a = EntityId::new(i);
        let id_b = EntityId::new(i + 1);
        assert_eq!(
            graph.source_doc_id_at(id_a, id_b),
            Some(expected_doc),
            "source_doc_id_at({id_a:?}, {id_b:?}) must be Some({expected_doc:?})"
        );
    }
}

#[tokio::test]
async fn proof_source_doc_ids_empty_before_first_compact() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::new(1);
    let id_a = EntityId::new(1);
    let id_b = EntityId::new(2);
    let doc_id = DocId::new(42);

    graph
        .add_entity(tx, Entity::new(id_a, "NodeA", "Type"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(id_b, "NodeB", "Type"))
        .await
        .unwrap();

    let edge = Edge::new(id_a, id_b, "relates").with_source_doc_id(doc_id);
    GraphIndex::add_edge(graph.as_ref(), tx, edge)
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    // KNOWN: source_doc_ids erst nach compact() befüllt
    // Before compact(), the uncompacted pending_edges buffer contains the payload,
    // but CSR source_doc_ids array is still empty/unpopulated for raw index lookup.
    assert_eq!(
        graph.get_source_doc_id(0),
        None,
        "get_source_doc_id(0) must return None before compact without panic"
    );

    // source_doc_id_at checks pending_edges, so it can resolve before compact
    assert_eq!(graph.source_doc_id_at(id_a, id_b), Some(doc_id));
}

#[tokio::test]
async fn proof_source_doc_ids_consistent_after_multiple_compacts() {
    let graph = Arc::new(CsrGraph::new());

    // Phase 1: 50 edges, then compact
    let tx1 = TxId::new(1);
    for i in 0..=50 {
        graph
            .add_entity(
                tx1,
                Entity::new(EntityId::new(i), format!("P1_Node{i}"), "Type"),
            )
            .await
            .unwrap();
    }
    for i in 0..50 {
        let edge = Edge::new(EntityId::new(i), EntityId::new(i + 1), "rel1")
            .with_source_doc_id(DocId::new(i + 1000));
        GraphIndex::add_edge(graph.as_ref(), tx1, edge)
            .await
            .unwrap();
    }
    graph.commit(tx1).await.unwrap();
    graph.compact();

    // Verify first 50 edges
    for i in 0..50 {
        assert_eq!(
            graph.source_doc_id_at(EntityId::new(i), EntityId::new(i + 1)),
            Some(DocId::new(i + 1000))
        );
    }

    // Phase 2: 50 additional edges, then compact
    let tx2 = TxId::new(2);
    for i in 50..=100 {
        graph
            .add_entity(
                tx2,
                Entity::new(EntityId::new(i), format!("P2_Node{i}"), "Type"),
            )
            .await
            .unwrap();
    }
    for i in 50..100 {
        let edge = Edge::new(EntityId::new(i), EntityId::new(i + 1), "rel2")
            .with_source_doc_id(DocId::new(i + 1000));
        GraphIndex::add_edge(graph.as_ref(), tx2, edge)
            .await
            .unwrap();
    }
    graph.commit(tx2).await.unwrap();
    graph.compact();

    // Verify all 100 edges after second compact
    for i in 0..100 {
        let id_a = EntityId::new(i);
        let id_b = EntityId::new(i + 1);
        let expected_doc = DocId::new(i + 1000);
        assert_eq!(
            graph.source_doc_id_at(id_a, id_b),
            Some(expected_doc),
            "source_doc_id_at for edge {i} ({id_a:?} -> {id_b:?}) must be Some({expected_doc:?})"
        );
        assert_eq!(
            graph.get_source_doc_id(i as usize),
            Some(expected_doc),
            "CSR array index {i} must be Some({expected_doc:?})"
        );
    }
}
