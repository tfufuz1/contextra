// FILE-CONTEXT
// ZWECK: Prüft die PID-regulatorische Einbindung in den Multi-Step Candidate Pool und Grenzwerte [50, 200].

use contextra_adapt::PidController;
use contextra_db::multistep::{MultiStepConfig, MultiStepEngine};
use contextra_db::Collection;
use contextra_graph::CsrGraph;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_vector::{HnswConfig, HnswIndex};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

async fn create_test_collection() -> Arc<Collection<LsmStorage>> {
    let dir = tempdir().expect("tempdir");
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.expect("lsm storage"));
    let hnsw_config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config).expect("hnsw index"));
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    Arc::new(col)
}

#[test]
fn test_pid_regulated_candidate_pool_direct_call_and_bounds() {
    let mut pid = PidController::new(100.0, 50, 200, Some(100));
    let dt = Duration::from_millis(100);

    // Initial value is 100
    assert_eq!(pid.current_pool_size(), Some(100));

    // Severe latency spike (500 ms vs 100 ms target) -> pool size contracts
    let new_pool_high = pid.update(dt, 500.0);
    assert!(new_pool_high < 100);
    assert!(new_pool_high >= 50);

    // Continuous high latency -> settles at hard floor 50
    for _ in 0..20 {
        pid.update(dt, 1000.0);
    }
    assert_eq!(pid.current_pool_size(), Some(50));

    // Continuous low latency (10 ms vs 100 ms target) -> pool size expands up to 200
    for _ in 0..30 {
        pid.update(dt, 10.0);
    }
    assert_eq!(pid.current_pool_size(), Some(200));
}

#[tokio::test]
async fn test_multistep_engine_uses_pid_candidate_pool() {
    let col = create_test_collection().await;
    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "test doc"})),
    )
    .await
    .expect("insert");

    let config = MultiStepConfig {
        max_rounds: 1,
        quality_threshold: 0.0,
        min_quality_hits: 1,
        latency_budget_ms: 100.0,
        ..Default::default()
    };

    let engine = MultiStepEngine::new(col, config);

    let res = engine
        .search("test", &[1.0, 0.0, 0.0, 0.0], 10, None)
        .await
        .expect("search");

    assert_eq!(res.rounds_executed, 1);
    assert!(!res.results.is_empty());
}
