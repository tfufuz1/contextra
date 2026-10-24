#![cfg(not(loom))]
use contextra_engine::fusion::{Signal, SignalFailurePolicy};
use contextra_engine::{Collection, DistanceMetric, Language};
use contextra_graph::CsrGraph;
use contextra_store::{LsmConfig, LsmStorage};
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
async fn test_signal_failure_fail_policy_returns_error() {
    let (col, _dir) = create_test_collection("fail_policy_col").await;

    col.insert(
        "doc-1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({ "text": "rust database systems" })),
    )
    .await
    .unwrap();

    // Query with mismatched vector dimension (3 elements instead of 4)
    let mismatched_vector = vec![1.0, 0.0, 0.0];

    // With default Fail policy, vector signal dimension failure causes query to return Err
    let res = col
        .query()
        .text("rust")
        .vector(&mismatched_vector)
        .on_signal_failure(SignalFailurePolicy::Fail)
        .execute()
        .await;

    assert!(
        res.is_err(),
        "Fail policy must propagate vector signal failure error"
    );
    let err = res.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Dimension mismatch") || err_msg.contains("expected 4"),
        "Error message must identify failing vector signal cause: {err_msg}"
    );
}

#[tokio::test]
async fn test_signal_failure_degrade_policy_delivers_partial_results_with_report() {
    let (col, _dir) = create_test_collection("degrade_policy_col").await;

    col.insert(
        "doc-1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({ "text": "rust database systems" })),
    )
    .await
    .unwrap();

    // Query with mismatched vector dimension (3 elements instead of 4)
    let mismatched_vector = vec![1.0, 0.0, 0.0];

    // With Degrade policy, vector signal failure is skipped and recorded in SearchReport.degraded_signals, while text search results are returned
    let (results, report) = col
        .query()
        .text("rust")
        .vector(&mismatched_vector)
        .on_signal_failure(SignalFailurePolicy::Degrade)
        .execute_with_report()
        .await
        .unwrap();

    assert!(
        !results.is_empty(),
        "Degrade policy must return partial results from non-failing text signal"
    );
    assert_eq!(results[0].id, "doc-1");

    assert_eq!(
        report.degraded_signals,
        vec![Signal::Vector],
        "SearchReport must contain Signal::Vector in degraded_signals"
    );
}
