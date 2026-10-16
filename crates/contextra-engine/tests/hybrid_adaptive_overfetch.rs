#![cfg(not(loom))]
use contextra_engine::{Collection, DistanceMetric, Language};
use contextra_graph::CsrGraph;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{DocId, FilterExpr};
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
async fn test_adaptive_overfetch_1_percent_selectivity() {
    let (col, _dir) = create_test_collection("overfetch_col").await;

    // Populate 300 documents, where exactly 3 documents (1% selectivity) match target=true
    let mut matching_doc_ids = std::collections::HashSet::new();
    for i in 1..=300 {
        let id = format!("doc-{:03}", i);
        let is_target = i % 100 == 0; // 3 target docs: doc-100, doc-200, doc-300 (~1% selectivity)
        col.insert(
            &id,
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({ "text": format!("item {:03} content", i), "target": is_target })),
        )
        .await
        .unwrap();

        if is_target {
            let doc_id = DocId::from_key(&id).unwrap();
            matching_doc_ids.insert(doc_id);
        }
    }

    assert_eq!(
        matching_doc_ids.len(),
        3,
        "Exactly 3 target documents inserted"
    );

    let filter = FilterExpr::Eq {
        field: "target".to_string(),
        value: json!(true),
    };

    // Query with 1% selective metadata filter, requesting k=3
    let (results, report) = col
        .query()
        .text("item content")
        .vector([1.0, 0.0, 0.0, 0.0])
        .filter(filter)
        .k(3)
        .execute_with_report()
        .await
        .unwrap();

    // Verification: exactly 3 matching hits returned
    assert_eq!(
        results.len(),
        3,
        "1% selective query requesting k=3 must return all 3 matching documents"
    );

    for res in &results {
        let doc_id = DocId::from_key(&res.id).unwrap();
        assert!(
            matching_doc_ids.contains(&doc_id),
            "Returned result {} must be in 1% target set",
            res.id
        );
    }

    // Stage count verification: adaptive overfetching executed multiple stages to find all 3 items
    assert!(
        report.overfetch_stages >= 1 && report.overfetch_stages <= 3,
        "overfetch_stages must be in range 1..=3, got {}",
        report.overfetch_stages
    );
}

#[tokio::test]
async fn test_adaptive_overfetch_stage_progression_reporting() {
    let (col, _dir) = create_test_collection("overfetch_stages_col").await;

    for i in 1..=100 {
        let id = format!("doc-{:03}", i);
        col.insert(
            &id,
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({ "text": format!("content {:03}", i) })),
        )
        .await
        .unwrap();
    }

    // Unfiltered query executes stage 1
    let (_res, report) = col
        .query()
        .text("content")
        .vector([1.0, 0.0, 0.0, 0.0])
        .k(5)
        .execute_with_report()
        .await
        .unwrap();

    assert_eq!(
        report.overfetch_stages, 1,
        "Unfiltered search should report overfetch_stages = 1"
    );
}
