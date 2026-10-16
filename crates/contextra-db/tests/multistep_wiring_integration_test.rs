use contextra_db::wiring::{MultiStepWiring, MultiStepWiringDeps, PipelineStage};
use contextra_db::{Collection, MultiStepConfig};
use contextra_engine::Language;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_vector::{HnswConfig, HnswIndex};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_multistep_wiring_build_and_execute_cascade() {
    let dir = tempdir().expect("tempdir");
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .expect("lsm storage"),
    );
    let index = Arc::new(
        HnswIndex::try_new(HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .expect("hnsw index"),
    );
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Arc::new(Collection::new_without_graph(
        "default".to_string(),
        storage,
        index,
        next_tx,
        4,
        Language::English,
    ));

    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust programming"})),
    )
    .await
    .expect("insert");

    col.insert(
        "doc2",
        &[0.0, 1.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "secret sensitive python"})),
    )
    .await
    .expect("insert");

    // Egress filter rejecting items containing "secret"
    let egress_filter = Arc::new(|item: &contextra_engine::SearchResult| {
        if let Some(ref meta) = item.metadata {
            if let Some(text) = meta.get("text").and_then(|v| v.as_str()) {
                return !text.contains("secret");
            }
        }
        true
    });

    let deps = MultiStepWiringDeps {
        collection: col,
        config: MultiStepConfig::default(),
        clock: None,
        egress_filter: Some(egress_filter),
    };

    let wiring = MultiStepWiring::build(deps).expect("build");
    let stages = wiring.stages();
    assert!(stages.verify_order());
    assert_eq!(stages.stages[3], PipelineStage::EgressFilter);

    let res = wiring
        .execute_cascade("rust", &[1.0, 0.0, 0.0, 0.0], 5, None, None)
        .await
        .expect("execute_cascade");

    assert!(!res.results.is_empty());
    assert!(res.results.iter().all(|r| r.id != "doc2"));
}
