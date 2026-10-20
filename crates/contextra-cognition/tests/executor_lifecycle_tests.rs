// FILE-CONTEXT
// ZWECK: Lifecycle- und Edge-Case-Tests für ConsolidationEngine und ConsolidationExecutor (J15).
// INVARIANTEN: Deterministische Synchronisation ohne sleep()-Annahmen; RNG nur geseedet; Orakel-Assertions.
// STAND: TS:2026-10-21T12:00:00Z

use contextra_cognition::consolidation_executor::{
    execute_consolidation_pass, ConsolidationEngine, ConsolidationLockGuard,
};
use contextra_cognition::memory_consolidation::{ConsolidationConfig, SynthesisConfig};
use contextra_engine::collection::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::{BoxFuture, LlmTextGenerator};
use contextra_store::LsmStorage;
use contextra_types::DocId;
use contextra_vector::HnswIndex;
use serde_json::json;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

struct TestMockLlm {
    should_fail: AtomicBool,
    call_count: AtomicU64,
}

impl TestMockLlm {
    fn new() -> Self {
        Self {
            should_fail: AtomicBool::new(false),
            call_count: AtomicU64::new(0),
        }
    }
}

impl LlmTextGenerator for TestMockLlm {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, contextra_types::Result<String>> {
        Box::pin(async move {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            if self.should_fail.load(Ordering::SeqCst) {
                Err(contextra_types::ContextraError::Internal(
                    "Simulated LLM Failure".to_string(),
                ))
            } else {
                Ok("Synthesized summary text".to_string())
            }
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
async fn test_executor_double_stop_and_drop_races() {
    let (col, _dir) = create_test_collection().await;
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let engine = Arc::new(ConsolidationEngine::new(
        col,
        ConsolidationConfig::default(),
        SynthesisConfig::default(),
        Duration::from_millis(10),
        cancel_token.clone(),
    ));

    let handle = engine.start();

    // Double cancel / stop must not panic or block
    cancel_token.cancel();
    cancel_token.cancel();

    let join_res = handle.await;
    assert!(join_res.is_ok(), "Task must join cleanly on cancellation");
}

#[tokio::test]
async fn test_executor_stop_during_active_job() {
    let (col, _dir) = create_test_collection().await;
    let cancel_token = tokio_util::sync::CancellationToken::new();

    // Insert turns
    for i in 1..=6 {
        let id = format!("doc_{}", i);
        let emb = vec![1.0, 0.0, 0.0, 0.0];
        col.insert(&id, &emb, Some(json!({"i": i})))
            .await
            .expect("insert");
    }

    let engine = Arc::new(ConsolidationEngine::new(
        col.clone(),
        ConsolidationConfig {
            near_duplicate_cosine_threshold: 0.95,
            ..Default::default()
        },
        SynthesisConfig::default(),
        Duration::from_millis(5),
        cancel_token.clone(),
    ));

    let handle = engine.clone().start();

    // Trigger cycle manually while task is running
    let cycle_res = engine.run_cycle().await;
    assert!(cycle_res.is_ok());

    cancel_token.cancel();
    let _ = handle.await;
}

#[tokio::test]
async fn test_parallel_writes_during_consolidation_idempotency() {
    let (col, _dir) = create_test_collection().await;
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let engine = Arc::new(ConsolidationEngine::new(
        col.clone(),
        ConsolidationConfig::default(),
        SynthesisConfig::default(),
        Duration::from_millis(10),
        cancel_token.clone(),
    ));

    // Parallel writers inserting unique items
    let mut write_handles = Vec::new();
    for i in 0..10 {
        let col_c = col.clone();
        write_handles.push(tokio::spawn(async move {
            let id = format!("writer_doc_{}", i);
            let emb = vec![i as f32 * 0.1, 1.0, 0.0, 0.0];
            let _ = col_c.insert(&id, &emb, None).await;
        }));
    }

    // Run cycle in parallel
    let cycle_res = engine.run_cycle().await;
    assert!(cycle_res.is_ok());

    for h in write_handles {
        h.await.expect("writer handle");
    }

    // Repeated cycle for idempotency
    let cycle_res_repeat = engine.run_cycle().await;
    assert!(cycle_res_repeat.is_ok());

    assert_eq!(col.len().await, 10);
}

#[tokio::test]
async fn test_edge_cases_empty_and_single_turn() {
    let (col, _dir) = create_test_collection().await;

    // 1. Empty turn queue
    let empty_res = execute_consolidation_pass(col.as_ref(), &[], &ConsolidationConfig::default())
        .await
        .expect("empty pass");
    assert_eq!(empty_res.segments_created, 0);
    assert!(empty_res.duplicates_tombstoned.is_empty());

    // 2. Single turn
    let single_turn = vec![(DocId::new(100), vec![1.0, 0.0, 0.0, 0.0])];
    let single_res =
        execute_consolidation_pass(col.as_ref(), &single_turn, &ConsolidationConfig::default())
            .await
            .expect("single turn pass");
    assert_eq!(single_res.segments_created, 1);
    assert!(single_res.duplicates_tombstoned.is_empty());
}

#[tokio::test]
async fn test_job_error_no_executor_poisoning() {
    let (col, _dir) = create_test_collection().await;
    let cancel_token = tokio_util::sync::CancellationToken::new();
    let llm = Arc::new(TestMockLlm::new());

    let engine = Arc::new(
        ConsolidationEngine::new(
            col.clone(),
            ConsolidationConfig::default(),
            SynthesisConfig::default(),
            Duration::from_millis(10),
            cancel_token.clone(),
        )
        .with_llm(llm.clone()),
    );

    // Inject LLM failure
    llm.should_fail.store(true, Ordering::SeqCst);

    let res1 = engine.run_cycle().await;
    assert!(res1.is_ok(), "Cycle 1 must not crash or poison executor");

    // Clear failure
    llm.should_fail.store(false, Ordering::SeqCst);

    let res2 = engine.run_cycle().await;
    assert!(res2.is_ok(), "Cycle 2 must execute normally after failure");
}

#[test]
fn test_lock_guard_try_acquire_and_drop() {
    let flag = Arc::new(AtomicBool::new(false));

    {
        let guard = ConsolidationLockGuard::try_acquire(&flag);
        assert!(guard.is_some());
        assert!(flag.load(Ordering::Acquire));

        // Second acquisition must fail while held
        let guard2 = ConsolidationLockGuard::try_acquire(&flag);
        assert!(guard2.is_none());
    }

    // After drop, flag must be false and re-acquireable
    assert!(!flag.load(Ordering::Acquire));
    let guard3 = ConsolidationLockGuard::try_acquire(&flag);
    assert!(guard3.is_some());
}
