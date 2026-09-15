use memfuse_core::{DocId, Edge, Entity, EntityId, GraphIndex, TxId};
use memfuse_graph::CsrGraph;
use std::sync::Arc;

#[tokio::test]
async fn proof_source_doc_ids_set_after_compact() {
    let graph = Arc::new(CsrGraph::new());

    // Create 11 entities in separate committed transactions to ensure deterministic internal index order
    for i in 0..=10 {
        let tx_e = TxId::new(i + 1);
        let eid = EntityId::new(i);
        graph
            .add_entity(tx_e, Entity::new(eid, format!("Node{}", i), "Type"))
            .await
            .unwrap();
        graph.commit(tx_e).await.unwrap();
    }

    let tx_edges = TxId::new(100);
    for i in 0..10 {
        let id_a = EntityId::new(i);
        let id_b = EntityId::new(i + 1);
        let doc_id = DocId::new(i);
        let edge = Edge::new(id_a, id_b, "relates").with_source_doc_id(doc_id);
        GraphIndex::add_edge(graph.as_ref(), tx_edges, edge)
            .await
            .unwrap();
    }
    graph.commit(tx_edges).await.unwrap();

    // Verify lookup via source_doc_id_at while edge is in pending
    for i in 0..10 {
        let id_a = EntityId::new(i);
        let id_b = EntityId::new(i + 1);
        let doc_id = DocId::new(i);
        assert_eq!(graph.source_doc_id_at(id_a, id_b), Some(doc_id));
    }

    // Compact to populate CSR array source_doc_ids
    graph.compact();

    // Verify CSR array source_doc_ids via get_source_doc_id for each node/edge index
    for i in 0..10 {
        let id_a = EntityId::new(i);
        let id_b = EntityId::new(i + 1);
        let doc_id = DocId::new(i);

        assert_eq!(
            graph.get_source_doc_id(i as usize),
            Some(doc_id),
            "CSR array source_doc_ids at index {} must contain Some(doc_id)",
            i
        );
        assert_eq!(
            graph.source_doc_id_at(id_a, id_b),
            Some(doc_id),
            "source_doc_id_at must return Some(doc_id) after compact for edge {}->{}",
            i,
            i + 1
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
    // Accessing CSR source_doc_ids array before compact must be None without panicking
    assert_eq!(
        graph.get_source_doc_id(0),
        None,
        "CSR array source_doc_ids must be None/empty before first compact"
    );
    assert_eq!(
        graph.get_source_doc_id(999),
        None,
        "Out of bounds access to uncompacted source_doc_ids must return None without panic"
    );

    // source_doc_id_at still works via pending_edges buffer before compact
    assert_eq!(
        graph.source_doc_id_at(id_a, id_b),
        Some(doc_id),
        "source_doc_id_at must look up pending_edges before compact without panic"
    );
}

#[tokio::test]
async fn proof_source_doc_ids_consistent_after_multiple_compacts() {
    let graph = Arc::new(CsrGraph::new());

    // Create 101 entities in separate committed transactions to ensure deterministic internal index order
    for i in 0..=100 {
        let tx_e = TxId::new(i + 1);
        let eid = EntityId::new(i);
        graph
            .add_entity(tx_e, Entity::new(eid, format!("N{}", i), "Type"))
            .await
            .unwrap();
        graph.commit(tx_e).await.unwrap();
    }

    // Batch 1: 50 edges
    let tx_batch1 = TxId::new(200);
    for i in 0..50 {
        let id_a = EntityId::new(i);
        let id_b = EntityId::new(i + 1);
        let doc_id = DocId::new(i);
        let edge = Edge::new(id_a, id_b, "rel").with_source_doc_id(doc_id);
        GraphIndex::add_edge(graph.as_ref(), tx_batch1, edge)
            .await
            .unwrap();
    }
    graph.commit(tx_batch1).await.unwrap();

    // First compact
    graph.compact();

    // Verify first 50 edges
    for i in 0..50 {
        let doc_id = DocId::new(i);
        assert_eq!(
            graph.get_source_doc_id(i as usize),
            Some(doc_id),
            "Batch 1 source_doc_id at index {} must match",
            i
        );
    }

    // Batch 2: 50 more edges
    let tx_batch2 = TxId::new(300);
    for i in 50..100 {
        let id_a = EntityId::new(i);
        let id_b = EntityId::new(i + 1);
        let doc_id = DocId::new(i);
        let edge = Edge::new(id_a, id_b, "rel").with_source_doc_id(doc_id);
        GraphIndex::add_edge(graph.as_ref(), tx_batch2, edge)
            .await
            .unwrap();
    }
    graph.commit(tx_batch2).await.unwrap();

    // Second compact
    graph.compact();

    // Verify all 100 edges after second compact
    for i in 0..100 {
        let id_a = EntityId::new(i);
        let id_b = EntityId::new(i + 1);
        let doc_id = DocId::new(i);

        assert_eq!(
            graph.get_source_doc_id(i as usize),
            Some(doc_id),
            "All 100 source_doc_ids must be consistent after second compact at index {}",
            i
        );
        assert_eq!(
            graph.source_doc_id_at(id_a, id_b),
            Some(doc_id),
            "source_doc_id_at must be consistent after second compact for edge {}->{}",
            i,
            i + 1
        );
    }
}
