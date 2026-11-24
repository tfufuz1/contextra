//! Golden-Korpus Prüftest für `contextra-graph` (GR6.1).

use contextra_graph::csr::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{DocId, Edge, Entity, EntityId, PprConfig, TxId};
use std::fs;
use std::path::Path;

#[test]
fn test_golden_id_serialization() {
    let golden_dir = Path::new("tests/golden");
    if !golden_dir.exists() {
        return;
    }

    let entity_id = EntityId::new(42);
    let doc_id = DocId::new(100);
    let tx_id = TxId::new(200);

    let json_entity = serde_json::to_string(&entity_id).unwrap();
    let json_doc = serde_json::to_string(&doc_id).unwrap();
    let json_tx = serde_json::to_string(&tx_id).unwrap();

    let bincode_entity = bincode::serialize(&entity_id).unwrap();
    let bincode_doc = bincode::serialize(&doc_id).unwrap();
    let bincode_tx = bincode::serialize(&tx_id).unwrap();

    let expected_entity_json = fs::read_to_string(golden_dir.join("entity_id.json")).unwrap();
    assert_eq!(json_entity.trim(), expected_entity_json.trim());

    let expected_doc_json = fs::read_to_string(golden_dir.join("doc_id.json")).unwrap();
    assert_eq!(json_doc.trim(), expected_doc_json.trim());

    let expected_tx_json = fs::read_to_string(golden_dir.join("tx_id.json")).unwrap();
    assert_eq!(json_tx.trim(), expected_tx_json.trim());

    let expected_entity_bin = fs::read(golden_dir.join("entity_id.bin")).unwrap();
    assert_eq!(bincode_entity, expected_entity_bin);

    let expected_doc_bin = fs::read(golden_dir.join("doc_id.bin")).unwrap();
    assert_eq!(bincode_doc, expected_doc_bin);

    let expected_tx_bin = fs::read(golden_dir.join("tx_id.bin")).unwrap();
    assert_eq!(bincode_tx, expected_tx_bin);
}

#[tokio::test]
async fn test_golden_ppr_and_traversal() {
    let golden_dir = Path::new("tests/golden");
    if !golden_dir.exists() {
        return;
    }

    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=5 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(i), format!("Node{i}"), "Test"),
            )
            .await
            .unwrap();
    }

    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "link"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(3), "link"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(EntityId::new(3), EntityId::new(4), "link"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(EntityId::new(4), EntityId::new(5), "link"))
        .await
        .unwrap();
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(2), EntityId::new(4), "shortcut"),
        )
        .await
        .unwrap();
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(3), EntityId::new(1), "backlink"),
        )
        .await
        .unwrap();

    graph.commit(tx).await.unwrap();

    let ppr_cfg = PprConfig::default();
    let ppr_res = graph
        .personalized_page_rank(&[EntityId::new(1)], &ppr_cfg)
        .await
        .unwrap();
    let ppr_json = serde_json::to_string_pretty(&ppr_res).unwrap();

    let expected_ppr = fs::read_to_string(golden_dir.join("ppr_results.json")).unwrap();
    assert_eq!(ppr_json.trim(), expected_ppr.trim());

    let mut trav_valid = graph.traverse(EntityId::new(1), 3).await.unwrap();
    trav_valid.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0 .0.cmp(&b.0 .0)));
    let trav_json = serde_json::to_string_pretty(&trav_valid).unwrap();
    let expected_trav = fs::read_to_string(golden_dir.join("traversal_results.json")).unwrap();
    assert_eq!(trav_json.trim(), expected_trav.trim());
}
