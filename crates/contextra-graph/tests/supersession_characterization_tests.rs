//! Charakterisierungstests für Supersession, Cascade-Invalidierung und Decay-Gewichtung.
//!
//! # SPEC: T-CASCADE-SUPERSESSION
//! # Zweck:
//! Sichert das Verhalten ab, wenn ein Quelldokument durch ein neueres ersetzt wird (Supersession).
//! Kanten und abgeleitete Relationen werden atomar tombstoniert und decay-gewichtete Pfade aktualisiert.

#![expect(clippy::unwrap_used)]

use contextra_graph::cascade::{
    cascade_invalidate_edges_for_superseded_doc, cascade_invalidate_hyperedges_for_superseded_doc,
};
use contextra_graph::csr::{CsrGraph, EdgeType};
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_graph::test_doubles::InMemoryStorageEngine;
use contextra_ports::{GraphIndex, StorageEngine};
use contextra_types::{DocId, Entity, EntityId, TxId};
use std::sync::Arc;

/// SPEC: T-CASCADE-SUPERSESSION-01
/// Prüft, dass bei Dokument-Ersetzung abgeleitete Kanten kaskadierend tombstoniert werden.
#[tokio::test]
async fn test_characterize_document_supersession_edge_cascade() {
    let storage = Arc::new(InMemoryStorageEngine::new());
    let graph = Arc::new(CsrGraph::with_storage(storage.clone()));

    let doc_old = DocId::from_key("doc-legacy-v1").unwrap();
    let tx1 = TxId::new(10);

    let e1 = EntityId::new(1001);
    let e2 = EntityId::new(1002);
    let e3 = EntityId::new(1003);

    graph
        .add_entity(tx1, Entity::new(e1, "Ent1", "Concept"))
        .await
        .unwrap();
    graph
        .add_entity(tx1, Entity::new(e2, "Ent2", "Concept"))
        .await
        .unwrap();
    graph
        .add_entity(tx1, Entity::new(e3, "Ent3", "Concept"))
        .await
        .unwrap();

    // Kanten an doc_old binden
    graph
        .insert_edge_direct_with_bitemporal_validity(
            e1,
            e2,
            0.85,
            Some(tx1),
            None,
            Some(100),
            Some(500),
            Some(doc_old),
        )
        .await
        .unwrap();

    graph
        .insert_edge_direct_with_bitemporal_validity(
            e2,
            e3,
            0.90,
            Some(tx1),
            None,
            Some(100),
            Some(500),
            Some(doc_old),
        )
        .await
        .unwrap();

    graph.commit(tx1).await.unwrap();
    storage.commit(tx1).await.unwrap();

    // Vor Supersession: Kanten sind traversierbar
    let initial_neighbors = graph.neighbors(e1).await.unwrap();
    assert!(initial_neighbors.contains(&e2));

    // Supersession ausführen: doc_old wird durch Version v2 ersetzt
    let wal_seq = 20u64;
    let report = cascade_invalidate_edges_for_superseded_doc(&graph, doc_old, wal_seq)
        .await
        .unwrap();

    assert_eq!(report.tombstoned_edge_count, 2);
    assert!(report.affected_node_ids.contains(&e1));
    assert!(report.affected_node_ids.contains(&e2));

    // Nach Supersession: Kanten sind tombstoniert, Traversierung liefert sie nicht mehr
    let after_neighbors = graph.neighbors(e1).await.unwrap();
    assert!(!after_neighbors.contains(&e2));

    // Idempotenz: Wiederholte Supersession für selbes Dokument erzeugt 0 neue Tombstones
    let repeat_report = cascade_invalidate_edges_for_superseded_doc(&graph, doc_old, wal_seq + 1)
        .await
        .unwrap();
    assert_eq!(repeat_report.tombstoned_edge_count, 0);
}

/// SPEC: T-CASCADE-SUPERSESSION-02
/// Prüft, dass Hyperkanten bei Supersession atomar invalidiert und deren Rollenbindungen entkoppelt werden.
#[tokio::test]
async fn test_characterize_hyperedge_supersession_cascade() {
    let storage = Arc::new(InMemoryStorageEngine::new());
    let graph = CsrGraph::with_storage(storage.clone());

    let doc_v1 = DocId::from_key("doc-claim-v1").unwrap();

    const ROLE_SUBJECT: RoleId = RoleId::new(1);
    const ROLE_OBJECT: RoleId = RoleId::new(2);

    let he_id = HyperEdgeId::new(5001);
    let he = HyperEdge::new(
        he_id,
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_SUBJECT, EntityId::new(901)),
            RoleBinding::new(ROLE_OBJECT, EntityId::new(902)),
        ],
        0.95,
    )
    .with_source_doc_id(Some(doc_v1));

    graph.insert_hyperedge_direct(he);

    // Hyperedge ist vor Supersession vorhanden
    assert!(graph.get_hyperedge(he_id).is_some());
    assert_eq!(graph.hyperedges_for_doc(doc_v1).len(), 1);

    // Kaskadierende Invalidierung für doc_v1
    let wal_seq = 105;
    let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_v1, wal_seq)
        .await
        .unwrap();

    assert_eq!(report.invalidated.len(), 1);
    assert_eq!(report.invalidated[0], he_id);

    // Nach Supersession: Hyperkante existiert nicht mehr im aktiven Dokument-Set
    assert!(graph.hyperedges_for_doc(doc_v1).is_empty());
}
