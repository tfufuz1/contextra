use contextra_cognition::{
    aggregation_phase::AggregationConfig,
    consolidation_executor::start_consolidation_worker_full,
    context::ContextManager,
    maintenance_config::MaintenanceConfig,
    maintenance_scheduler::{ActiveSessionGuard, MaintenanceScheduler},
    memory_consolidation::{ConsolidationConfig, SynthesisConfig},
};
use contextra_engine::collection::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::{BoxFuture, ContextChunk, GroundingAssessment, GroundingValidator, LlmTextGenerator};
use contextra_store::LsmStorage;
use contextra_types::{Result, TokenBudget};
use contextra_vector::HnswIndex;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

struct DummyLlm;

impl LlmTextGenerator for DummyLlm {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move { Ok("Dummy response".to_string()) })
    }
}

struct DummyRichValidator;

impl GroundingValidator for DummyRichValidator {
    fn validate_grounding<'a>(
        &'a self,
        _response: &'a str,
        _sources: &'a [ContextChunk],
    ) -> BoxFuture<'a, Result<GroundingAssessment>> {
        Box::pin(async move {
            Ok(GroundingAssessment {
                score: 0.95,
                is_grounded: true,
                reason: Some("Valid grounding".to_string()),
            })
        })
    }
}

async fn create_test_collection() -> (Arc<Collection<LsmStorage, HnswIndex>>, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .expect("LsmStorage"),
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .expect("HnswIndex"),
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Arc::new(Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_engine::Language::English,
    ));
    (col, dir)
}

#[tokio::test]
async fn test_start_consolidation_worker_full_wiring() {
    let (col, _dir) = create_test_collection().await;
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let llm: Arc<dyn LlmTextGenerator> = Arc::new(DummyLlm);
    let rich_val: Arc<dyn GroundingValidator> = Arc::new(DummyRichValidator);
    let leanrag_cfg = AggregationConfig::default();

    let worker_handle = start_consolidation_worker_full(
        col,
        ConsolidationConfig::default(),
        SynthesisConfig::default(),
        Duration::from_millis(50),
        cancel_token.clone(),
        Some(llm),
        Some(rich_val),
        Some(leanrag_cfg),
    );

    tokio::time::sleep(Duration::from_millis(10)).await;
    cancel_token.cancel();

    let res = worker_handle.await;
    assert!(res.is_ok(), "consolidation worker full should exit cleanly on cancellation");
}

#[test]
fn test_context_manager_relevance_threshold_wiring() {
    let mgr = ContextManager::with_defaults()
        .with_relevance_threshold(0.85);
    assert_eq!(mgr.relevance_threshold(), 0.85);

    let mgr2 = ContextManager::new(TokenBudget::default())
        .with_threshold(0.65);
    assert_eq!(mgr2.relevance_threshold(), 0.65);
}

#[tokio::test]
async fn test_maintenance_scheduler_session_guard_wiring() {
    let (col, _dir) = create_test_collection().await;

    let scheduler = MaintenanceScheduler::new(
        MaintenanceConfig::default(),
        col,
        ConsolidationConfig::default(),
    );

    assert_eq!(scheduler.active_agent_sessions(), 0);
    {
        let _guard: ActiveSessionGuard<'_, _, _> = scheduler.acquire_session_guard();
        assert_eq!(scheduler.active_agent_sessions(), 1);
        {
            let _guard2 = scheduler.acquire_session_guard();
            assert_eq!(scheduler.active_agent_sessions(), 2);
        }
        assert_eq!(scheduler.active_agent_sessions(), 1);
    }
    assert_eq!(scheduler.active_agent_sessions(), 0);
}

#[cfg(feature = "edge-reinforcement-learning")]
#[tokio::test]
async fn test_maintenance_scheduler_edge_reinforcement_constructor_wiring() {
    let (col, _dir) = create_test_collection().await;
    let buffer = Arc::new(contextra_graph::EdgeReinforcementBuffer::new(100));

    let scheduler = MaintenanceScheduler::new_with_edge_reinforcement(
        MaintenanceConfig::default(),
        col,
        ConsolidationConfig::default(),
        buffer,
    );

    assert_eq!(scheduler.active_agent_sessions(), 0);
}
