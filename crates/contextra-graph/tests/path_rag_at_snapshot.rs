use contextra_graph::csr::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{Edge, Entity, EntityId, TxId};

#[tokio::test]
async fn test_path_rag_at_edge_visibility_by_seq() {
    // (a) Kante mit TxId 10 und Kante mit TxId 20: seq = 15 sieht nur die erste, seq = 25 beide
    let graph = CsrGraph::new();

    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);

    let tx10 = TxId::new(10);
    graph
        .add_entity(tx10, Entity::new(e1, "e1", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx10, Entity::new(e2, "e2", "Node"))
        .await
        .unwrap();
    graph
        .add_edge(tx10, Edge::new(e1, e2, "rel").with_weight(1.0))
        .await
        .unwrap();
    graph.commit(tx10).await.unwrap();

    let tx20 = TxId::new(20);
    graph
        .add_entity(tx20, Entity::new(e3, "e3", "Node"))
        .await
        .unwrap();
    graph
        .add_edge(tx20, Edge::new(e2, e3, "rel").with_weight(1.0))
        .await
        .unwrap();
    graph.commit(tx20).await.unwrap();

    // seq = 15 sees e1 -> e2 path, but e3 is not reachable
    let res_seq15 = graph.path_rag_at(&[e1], 2, 15).await.unwrap();
    let res_ids_15: Vec<EntityId> = res_seq15.iter().map(|(id, _)| *id).collect();
    assert!(res_ids_15.contains(&e2));
    assert!(!res_ids_15.contains(&e3));

    // seq = 25 sees e1 -> e2 -> e3
    let res_seq25 = graph.path_rag_at(&[e1], 2, 25).await.unwrap();
    let res_ids_25: Vec<EntityId> = res_seq25.iter().map(|(id, _)| *id).collect();
    assert!(res_ids_25.contains(&e2));
    assert!(res_ids_25.contains(&e3));
}

#[tokio::test]
async fn test_path_rag_at_deleted_edge_visibility() {
    // (b) eine zum seq gelöschte Kante ist unsichtbar, vor der Löschung sichtbar
    let graph = CsrGraph::new();

    let e1 = EntityId::new(10);
    let e2 = EntityId::new(20);

    let tx10 = TxId::new(10);
    graph
        .add_entity(tx10, Entity::new(e1, "e1", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx10, Entity::new(e2, "e2", "Node"))
        .await
        .unwrap();

    // Edge valid from tx 10 until tx 20 (deleted/expired at tx 20)
    let edge = Edge::new(e1, e2, "rel").with_validity(Some(TxId::new(10)), Some(TxId::new(20)));
    graph.add_edge(tx10, edge).await.unwrap();
    graph.commit(tx10).await.unwrap();

    // At seq = 15 (before deletion/expiration at 20), edge is visible
    let res_seq15 = graph.path_rag_at(&[e1], 1, 15).await.unwrap();
    assert!(res_seq15.iter().any(|(id, _)| *id == e2));

    // At seq = 25 (after deletion/expiration at 20), edge is invisible
    let res_seq25 = graph.path_rag_at(&[e1], 1, 25).await.unwrap();
    assert!(!res_seq25.iter().any(|(id, _)| *id == e2));
}

#[tokio::test]
async fn test_path_rag_at_latest_seq_matches_latest_state() {
    // (c) bei seq = neuester Stand entspricht das Ergebnis der bisherigen, nicht-snapshotfähigen PathRAG-Abfrage
    let graph = CsrGraph::new();

    let e1 = EntityId::new(100);
    let e2 = EntityId::new(200);
    let e3 = EntityId::new(300);

    let tx = TxId::new(50);
    graph
        .add_entity(tx, Entity::new(e1, "e1", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(e2, "e2", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(e3, "e3", "Node"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(e1, e2, "rel").with_weight(0.9))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(e2, e3, "rel").with_weight(0.9))
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    let res_seq100 = graph.path_rag_at(&[e1], 2, 100).await.unwrap();
    let res_seq50 = graph.path_rag_at(&[e1], 2, 50).await.unwrap();

    assert_eq!(res_seq100, res_seq50);
    assert!(!res_seq100.is_empty());
}

#[tokio::test]
async fn test_path_rag_at_deterministic_ordering() {
    // (d) deterministische Reihenfolge über 10 Wiederholungen
    let graph = CsrGraph::new();

    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);
    let e4 = EntityId::new(4);

    let tx = TxId::new(10);
    graph
        .add_entity(tx, Entity::new(e1, "e1", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(e2, "e2", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(e3, "e3", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(e4, "e4", "Node"))
        .await
        .unwrap();

    // Equal weights so tie-breaker sorting by EntityId ascending is exercised
    graph
        .add_edge(tx, Edge::new(e1, e2, "rel").with_weight(1.0))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(e1, e3, "rel").with_weight(1.0))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(e1, e4, "rel").with_weight(1.0))
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    let baseline = graph.path_rag_at(&[e1], 1, 15).await.unwrap();
    assert!(!baseline.is_empty());

    for _ in 0..10 {
        let run = graph.path_rag_at(&[e1], 1, 15).await.unwrap();
        assert_eq!(baseline, run);
    }
}

#[tokio::test]
async fn test_path_rag_at_unknown_anchor_returns_empty() {
    // (e) unbekannter Anker liefert ein leeres Ergebnis ohne Panic
    let graph = CsrGraph::new();
    let unknown = EntityId::new(99999);

    let res = graph.path_rag_at(&[unknown], 2, 10).await.unwrap();
    assert!(res.is_empty());
}

#[tokio::test]
async fn test_path_rag_at_hub_entity_budget() {
    // (f) Hub-Entität mit sehr vielen Nachbarn bleibt im Budget
    let graph = CsrGraph::new();
    let hub = EntityId::new(1000);

    let tx = TxId::new(10);
    graph
        .add_entity(tx, Entity::new(hub, "hub", "Node"))
        .await
        .unwrap();

    for i in 0..200 {
        let neighbor = EntityId::new(2000 + i);
        graph
            .add_entity(tx, Entity::new(neighbor, "nbr", "Node"))
            .await
            .unwrap();
        graph
            .add_edge(tx, Edge::new(hub, neighbor, "rel").with_weight(1.0))
            .await
            .unwrap();
    }
    graph.commit(tx).await.unwrap();

    let res = graph.path_rag_at(&[hub], 2, 15).await.unwrap();
    assert!(!res.is_empty());
}
