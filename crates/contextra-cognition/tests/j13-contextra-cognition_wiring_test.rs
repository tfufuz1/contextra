// FILE-CONTEXT
// ZWECK: Integrationstest zur Verifikation der Produktionsverdrahtung der 8 Symbole aus J13-contextra-cognition.
// TEST: cargo test -p contextra-cognition --test j13-contextra-cognition_wiring_test --all-features

use contextra_cognition::consolidation_executor::{
    start_consolidation_worker, ConsolidationEngine,
};
use contextra_cognition::context::ContextManager;
use contextra_cognition::maintenance_config::MaintenanceConfig;
use contextra_cognition::maintenance_scheduler::MaintenanceScheduler;
use contextra_cognition::memory_consolidation::{ConsolidationConfig, SynthesisConfig};
use contextra_engine::collection::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::{
    BoxFuture, GroundingAssessment, GroundingValidator, LlmTextGenerator,
};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{ContextChunk, DocId, Result};
use contextra_vector::{HnswConfig, HnswIndex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

struct MockLlm;

impl LlmTextGenerator for MockLlm {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move { Ok("Synthesized test summary.".to_string()) })
    }
}

struct MockRichValidator {
    called: AtomicBool,
}

impl GroundingValidator for MockRichValidator {
    fn validate_grounding<'a>(
        &'a self,
        _response: &'a str,
        _sources: &'a [ContextChunk],
    ) -> BoxFuture<'a, Result<GroundingAssessment>> {
        self.called.store(true, Ordering::SeqCst);
        Box::pin(async move {
            Ok(GroundingAssessment {
                score: 0.95,
                is_grounded: true,
                reason: Some("High grounding score".to_string()),
            })
        })
    }
}

async fn create_test_collection() -> (
    Arc<Collection<LsmStorage, HnswIndex>>,
    tempfile::TempDir,
) {
    let dir = tempdir().expect("tempdir creation");
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .expect("LsmStorage initialization"),
    );
    let index = Arc::new(
        HnswIndex::try_new(HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .expect("HnswIndex initialization"),
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
async fn test_consolidation_engine_builders_and_start_worker() {
    let (col, _dir) = create_test_collection().await;
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let llm: Arc<dyn LlmTextGenerator> = Arc::new(MockLlm);
    let rich_val = Arc::new(MockRichValidator {
        called: AtomicBool::new(false),
    });
    let leanrag_cfg = contextra_cognition::aggregation_phase::AggregationConfig::default();

    // Exercise ConsolidationEngine builder options: with_llm, with_leanrag, with_rich_validator
    let engine = Arc::new(
        ConsolidationEngine::new(
            col.clone(),
            ConsolidationConfig::default(),
            SynthesisConfig::default(),
            Duration::from_millis(50),
            cancel_token.clone(),
        )
        .with_llm(llm)
        .with_leanrag(leanrag_cfg)
        .with_rich_validator(rich_val),
    );

    // Exercise start_worker static method on ConsolidationEngine
    let cancel_token_worker = tokio_util::sync::CancellationToken::new();
    let worker_handle = ConsolidationEngine::start_worker(
        col.clone(),
        ConsolidationConfig::default(),
        SynthesisConfig::default(),
        Duration::from_millis(100),
        cancel_token_worker.clone(),
    );

    // Exercise top-level start_consolidation_worker (delegates to ConsolidationEngine::start_worker)
    let cancel_token_top = tokio_util::sync::CancellationToken::new();
    let top_worker_handle = start_consolidation_worker(
        col.clone(),
        ConsolidationConfig::default(),
        SynthesisConfig::default(),
        Duration::from_millis(100),
        cancel_token_top.clone(),
    );

    // Execute one cycle on the configured engine
    let cycle_res = engine.run_cycle().await;
    assert!(cycle_res.is_ok(), "run_cycle on configured engine should succeed");

    cancel_token_worker.cancel();
    cancel_token_top.cancel();

    let _ = worker_handle.await;
    let _ = top_worker_handle.await;
}

#[test]
fn test_context_manager_set_relevance_threshold() {
    let mut mgr = ContextManager::with_defaults();
    assert_eq!(mgr.relevance_threshold(), 0.0);

    // Exercise set_relevance_threshold
    mgr.set_relevance_threshold(0.7);
    assert_eq!(mgr.relevance_threshold(), 0.7);

    let chunks = vec![
        ContextChunk {
            doc_id: DocId::new(1),
            content: "Low relevance chunk".into(),
            relevance: 0.5,
            token_count: 10,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        },
        ContextChunk {
            doc_id: DocId::new(2),
            content: "High relevance chunk".into(),
            relevance: 0.8,
            token_count: 10,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        },
    ];

    let window = mgr.prepare_context(chunks).expect("prepare_context should succeed");
    assert_eq!(window.chunks.len(), 1);
    assert_eq!(window.chunks[0].doc_id, DocId::new(2));
    assert_eq!(window.chunks[0].relevance, 0.8);
}

#[tokio::test]
async fn test_maintenance_scheduler_active_sessions_tracking() {
    let (col, _dir) = create_test_collection().await;
    let scheduler = MaintenanceScheduler::new(
        MaintenanceConfig::default(),
        col,
        ConsolidationConfig::default(),
    );

    assert_eq!(scheduler.active_agent_sessions(), 0);

    // Exercise increment_active_sessions
    let count_after_inc = scheduler.increment_active_sessions();
    assert_eq!(count_after_inc, 1);
    assert_eq!(scheduler.active_agent_sessions(), 1);

    // Run tick with active sessions > 0 (gating active)
    scheduler.run_tick().await;

    // Exercise decrement_active_sessions
    let count_after_dec = scheduler.decrement_active_sessions();
    assert_eq!(count_after_dec, 0);
    assert_eq!(scheduler.active_agent_sessions(), 0);

    // Run tick with active sessions == 0
    scheduler.run_tick().await;
}

#[cfg(feature = "edge-reinforcement-learning")]
#[tokio::test]
async fn test_maintenance_scheduler_with_edge_reinforcement_buffer() {
    let (col, _dir) = create_test_collection().await;
    let buffer = Arc::new(contextra_graph::EdgeReinforcementBuffer::new());

    // Exercise with_edge_reinforcement_buffer
    let scheduler = MaintenanceScheduler::new(
        MaintenanceConfig::default(),
        col,
        ConsolidationConfig::default(),
    )
    .with_edge_reinforcement_buffer(buffer);

    scheduler.run_tick().await;
}
