//! Generator für die Golden-Korpus-Dateien (wird via `golden/regen.sh` aufgerufen).

use contextra_graph::csr::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{DocId, Edge, Entity, EntityId, PprConfig, TxId};
use std::fs;
use std::path::Path;

#[tokio::test]
#[ignore]
async fn generate_golden_files() {
    let out_dir = Path::new("tests/golden");
    fs::create_dir_all(out_dir).unwrap();

    let entity_id = EntityId::new(42);
    let doc_id = DocId::new(100);
    let tx_id = TxId::new(200);

    fs::write(
        out_dir.join("entity_id.json"),
        serde_json::to_string(&entity_id).unwrap(),
    )
    .unwrap();
    fs::write(
        out_dir.join("doc_id.json"),
        serde_json::to_string(&doc_id).unwrap(),
    )
    .unwrap();
    fs::write(
        out_dir.join("tx_id.json"),
        serde_json::to_string(&tx_id).unwrap(),
    )
    .unwrap();

    fs::write(
        out_dir.join("entity_id.bin"),
        bincode::serialize(&entity_id).unwrap(),
    )
    .unwrap();
    fs::write(
        out_dir.join("doc_id.bin"),
        bincode::serialize(&doc_id).unwrap(),
    )
    .unwrap();
    fs::write(
        out_dir.join("tx_id.bin"),
        bincode::serialize(&tx_id).unwrap(),
    )
    .unwrap();

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
    fs::write(
        out_dir.join("ppr_results.json"),
        serde_json::to_string_pretty(&ppr_res).unwrap(),
    )
    .unwrap();

    let mut trav_valid = graph.traverse(EntityId::new(1), 3).await.unwrap();
    trav_valid.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0 .0.cmp(&b.0 .0)));
    fs::write(
        out_dir.join("traversal_results.json"),
        serde_json::to_string_pretty(&trav_valid).unwrap(),
    )
    .unwrap();

    println!("Golden files successfully generated in tests/golden/");
}
