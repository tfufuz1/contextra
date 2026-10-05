// FILE-CONTEXT
// STAND: 2026-10-05
// ZWECK: Testet Kaskaden-Löschung von Graph-Kanten nach Dokument-Löschung (DSGVO-Verpflichtung)
// ORAKEL: Handgerechnetes Mengen-Diagramm:
//         - Dokument D1 hat Kante A (nur D1) und Kante B (D1 und D2).
//         - Dokument D2 hat Kante B (D1 und D2) und Kante C (nur D2).
//         - Dokument D3 hat Kante D (nur D3).
//         Löschen von D1: Kante A wird tombstoniert, Kante B bleibt wegen Quelle D2, Kanten C und D bleiben.
//         Löschen unbekanntes Dokument: No-op ohne Panic.
//         Doppeltes Löschen D1: Idempotent (Keine Wirkung).
//         Löschen von D2: Kante B und Kante C werden tombstoned.

use contextra_graph::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{DocId, Edge, Entity, EntityId, TxId};
use std::sync::Arc;

#[tokio::test]
async fn test_provenance_delete_cascade_oracle() {
    let graph = Arc::new(CsrGraph::new());
    let tx1 = TxId::new(1);

    let doc1 = DocId::from_key("doc-1").unwrap();
    let doc2 = DocId::from_key("doc-2").unwrap();
    let doc3 = DocId::from_key("doc-3").unwrap();

    let n1 = EntityId::new(1);
    let n2 = EntityId::new(2);
    let n3 = EntityId::new(3);
    let n4 = EntityId::new(4);
    let n5 = EntityId::new(5);

    for (id, name) in [(n1, "N1"), (n2, "N2"), (n3, "N3"), (n4, "N4"), (n5, "N5")] {
        graph
            .add_entity(tx1, Entity::new(id, name, "Node"))
            .await
            .unwrap();
    }

    let edge_a = (n1, n2);
    let edge_b = (n2, n3);
    let edge_c = (n3, n4);

    // Kante A: nur von D1
    GraphIndex::add_edge(
        graph.as_ref(),
        tx1,
        Edge::new(n1, n2, "rel_a").with_source_doc_id(doc1),
    )
    .await
    .unwrap();

    // Kante B: von D1 und D2
    GraphIndex::add_edge(
        graph.as_ref(),
        tx1,
        Edge::new(n2, n3, "rel_b").with_source_doc_id(doc1),
    )
    .await
    .unwrap();
    GraphIndex::add_edge(
        graph.as_ref(),
        tx1,
        Edge::new(n2, n3, "rel_b").with_source_doc_id(doc2),
    )
    .await
    .unwrap();

    // Kante C: nur von D2
    GraphIndex::add_edge(
        graph.as_ref(),
        tx1,
        Edge::new(n3, n4, "rel_c").with_source_doc_id(doc2),
    )
    .await
    .unwrap();

    // Kante D: nur von D3
    GraphIndex::add_edge(
        graph.as_ref(),
        tx1,
        Edge::new(n4, n5, "rel_d").with_source_doc_id(doc3),
    )
    .await
    .unwrap();

    GraphIndex::commit(graph.as_ref(), tx1).await.unwrap();

    // Vor Löschen: Alle Kanten aktiv
    assert_eq!(graph.neighbors(n1).await.unwrap().len(), 1);
    assert_eq!(graph.neighbors(n2).await.unwrap().len(), 1);
    assert_eq!(graph.neighbors(n3).await.unwrap().len(), 1);
    assert_eq!(graph.neighbors(n4).await.unwrap().len(), 1);

    // Orakel-Schritt 1: Lösche D1
    // Handgerechnet: Kante A weg, Kante B bleibt (Quelle D2), Kanten C und D bleiben.
    let tx2 = TxId::new(2);
    let tombstoned_edges = graph.remove_doc(doc1, tx2).await.unwrap();

    assert_eq!(
        tombstoned_edges,
        vec![edge_a],
        "Orakel: Nur Kante A darf tombstoniert werden, da Kante B noch von D2 belegt ist"
    );

    // Kante A weg (n1 hat keine Nachbarn)
    assert!(graph.neighbors(n1).await.unwrap().is_empty());
    // Kante B bleibt (n2 -> n3)
    assert_eq!(graph.neighbors(n2).await.unwrap().len(), 1);
    // Kante C bleibt (n3 -> n4)
    assert_eq!(graph.neighbors(n3).await.unwrap().len(), 1);
    // Kante D bleibt (n4 -> n5)
    assert_eq!(graph.neighbors(n4).await.unwrap().len(), 1);

    // Orakel-Schritt 2: Löschen eines unbekannten Dokuments (No-op ohne Panic)
    let doc_unknown = DocId::from_key("doc-unknown").unwrap();
    let res_unknown = graph.remove_doc(doc_unknown, TxId::new(3)).await.unwrap();
    assert!(
        res_unknown.is_empty(),
        "Löschen eines unbekannten Dokuments muss leeres Resultat liefern"
    );

    // Orakel-Schritt 3: Doppeltes Löschen von D1 (Idempotenz)
    let res_repeat = graph.remove_doc(doc1, TxId::new(4)).await.unwrap();
    assert!(
        res_repeat.is_empty(),
        "Doppeltes Löschen von D1 muss idempotent sein"
    );

    // Orakel-Schritt 4: Lösche D2
    // Handgerechnet: Kante B und Kante C werden tombstoned (beide hatten jetzt nur noch D2 als Quelle)
    let tx5 = TxId::new(5);
    let mut tombstoned_d2 = graph.remove_doc(doc2, tx5).await.unwrap();
    tombstoned_d2.sort();
    let mut expected_d2 = vec![edge_b, edge_c];
    expected_d2.sort();

    assert_eq!(
        tombstoned_d2, expected_d2,
        "Orakel: Nach Löschen von D2 müssen Kanten B und C tombstoniert werden"
    );

    // Kante B und C weg
    assert!(graph.neighbors(n2).await.unwrap().is_empty());
    assert!(graph.neighbors(n3).await.unwrap().is_empty());
    // Kante D bleibt
    assert_eq!(graph.neighbors(n4).await.unwrap().len(), 1);
}
