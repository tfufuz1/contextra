use memfuse_core::{DocId, Edge, Entity, EntityId, GraphIndex, TxId};
use memfuse_graph::CsrGraph;
use std::sync::Arc;

#[tokio::test]
async fn proof_source_doc_ids_populated_after_insert() {
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

    // Verify lookup via source_doc_id_at while edge is in pending
    assert_eq!(graph.source_doc_id_at(id_a, id_b), Some(doc_id));

    // Compact to populate CSR array source_doc_ids
    graph.compact();

    // After compact, source_doc_id_at must still return Some(doc_id) from CSR array
    assert_eq!(
        graph.source_doc_id_at(id_a, id_b),
        Some(doc_id),
        "source_doc_id_at must return Some(doc_id) after compact"
    );

    // Also test get_source_doc_id(0) returning Some(doc_id)
    assert_eq!(
        graph.get_source_doc_id(0),
        Some(doc_id),
        "CSR array source_doc_ids at index 0 must contain Some(doc_id)"
    );
}

#[tokio::test]
async fn proof_source_doc_ids_consistent_after_compact() {
    let graph = Arc::new(CsrGraph::new());
    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);
    let tx3 = TxId::new(3);

    let id_a = EntityId::new(10);
    let id_b = EntityId::new(20);
    let id_c = EntityId::new(30);

    let doc_1 = DocId::new(101);
    let doc_2 = DocId::new(102);

    // Add entities in separate committed transactions to ensure deterministic index order
    graph
        .add_entity(tx1, Entity::new(id_a, "A", "Type"))
        .await
        .unwrap();
    graph.commit(tx1).await.unwrap();

    graph
        .add_entity(tx2, Entity::new(id_b, "B", "Type"))
        .await
        .unwrap();
    graph.commit(tx2).await.unwrap();

    graph
        .add_entity(tx3, Entity::new(id_c, "C", "Type"))
        .await
        .unwrap();
    graph.commit(tx3).await.unwrap();

    let tx_edges = TxId::new(4);
    let e1 = Edge::new(id_a, id_b, "rel1").with_source_doc_id(doc_1);
    let e2 = Edge::new(id_b, id_c, "rel2").with_source_doc_id(doc_2);

    GraphIndex::add_edge(graph.as_ref(), tx_edges, e1)
        .await
        .unwrap();
    GraphIndex::add_edge(graph.as_ref(), tx_edges, e2)
        .await
        .unwrap();
    graph.commit(tx_edges).await.unwrap();

    // Perform compact
    graph.compact();

    assert_eq!(graph.source_doc_id_at(id_a, id_b), Some(doc_1));
    assert_eq!(graph.source_doc_id_at(id_b, id_c), Some(doc_2));

    // Verify index lookup for all compacted edges (node 0 (A) -> B is CSR edge 0, node 1 (B) -> C is CSR edge 1)
    assert_eq!(graph.get_source_doc_id(0), Some(doc_1));
    assert_eq!(graph.get_source_doc_id(1), Some(doc_2));
}
