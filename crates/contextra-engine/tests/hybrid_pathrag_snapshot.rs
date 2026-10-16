#![cfg(not(loom))]
use contextra_engine::{Collection, DistanceMetric, Language};
use contextra_graph::CsrGraph;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{EntityId, GraphTraversalStrategy};
use contextra_vector::{HnswConfig, HnswIndex};
use serde_json::json;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::TempDir;

async fn create_test_collection(name: &str) -> (Collection<LsmStorage, HnswIndex>, TempDir) {
    let dir = TempDir::new().unwrap();
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap());
    let hnsw_config = HnswConfig {
        dimension: 4,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config).unwrap());
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        name.to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        Language::English,
    );
    (col, dir)
}

#[tokio::test]
async fn test_pathrag_snapshot_isolation() {
    let (col, _dir) = create_test_collection("pathrag_snapshot_col").await;

    col.insert(
        "doc-base",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({ "text": "base document" })),
    )
    .await
    .unwrap();

    col.insert(
        "doc-v1",
        &[0.9, 0.1, 0.0, 0.0],
        Some(json!({ "text": "initial version document" })),
    )
    .await
    .unwrap();

    let eid_base = EntityId::from_key("doc-base").unwrap();

    col.relate("doc-base", "doc-v1", "LINKS_TO").await.unwrap();

    // Capture sequence number seq1
    let seq1 = col.snapshot_seq().await.unwrap();

    // Insert doc-v2 after seq1
    col.insert(
        "doc-v2",
        &[0.8, 0.2, 0.0, 0.0],
        Some(json!({ "text": "subsequent version document" })),
    )
    .await
    .unwrap();

    col.relate("doc-base", "doc-v2", "LINKS_TO").await.unwrap();

    let path_rag_strat = GraphTraversalStrategy::PathRag {
        max_hops: 2,
        sufficiency_threshold: 0.1,
    };

    // Query PathRag at seq1
    let results_at_seq1 = col
        .query()
        .text("document")
        .anchors(vec![eid_base])
        .strategy(path_rag_strat.clone())
        .seq(seq1)
        .k(10)
        .execute()
        .await
        .unwrap();

    let ids_seq1: Vec<&str> = results_at_seq1.iter().map(|r| r.id.as_str()).collect();

    // doc-v2 inserted after seq1 must NOT be visible in results_at_seq1
    assert!(
        !ids_seq1.contains(&"doc-v2"),
        "Subsequent insertion 'doc-v2' must be invisible at snapshot seq1. Got: {ids_seq1:?}"
    );

    // Query PathRag at latest sequence
    let results_latest = col
        .query()
        .text("document")
        .anchors(vec![eid_base])
        .strategy(path_rag_strat)
        .k(10)
        .execute()
        .await
        .unwrap();

    let ids_latest: Vec<&str> = results_latest.iter().map(|r| r.id.as_str()).collect();

    assert!(
        ids_latest.contains(&"doc-v2"),
        "doc-v2 must be visible at latest sequence. Got: {ids_latest:?}"
    );
}
