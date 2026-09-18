use memfuse_agent::{
    AgentContext, AgentTool, DeadLetterReason, NodeType, OrchestratorEngine, StateGraph, StepResult,
};
use memfuse_core::{BoxFuture, MemFuseError, Result, TokenBudget};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_tool_timeout_creates_dead_letter() -> Result<()> {
    struct HangingTool;
    impl AgentTool for HangingTool {
        fn name(&self) -> &str {
            "hanging"
        }
        fn timeout_ms(&self) -> u64 {
            50 // 50ms Timeout
        }
        fn execute<'a>(
            &'a self,
            _: &'a AgentContext,
            _: serde_json::Value,
        ) -> BoxFuture<'a, Result<StepResult>> {
            Box::pin(async move {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                unreachable!()
            })
        }
    }

    let temp_dir = tempfile::TempDir::new()?;
    let config = memfuse_db::MemFuseConfig::default();
    let db = Arc::new(memfuse_db::MemFuse::open_with_config(temp_dir.path(), config).await?);
    let state_coll = db.collection("dlq_test_col").await?;

    let mut engine = OrchestratorEngine::try_from_db(&db)?;
    engine.try_register_tool(Box::new(HangingTool))?;

    let mut graph = StateGraph::new();
    graph.try_add_node("start", "Start Node", NodeType::Start, None)?;
    graph.try_add_node("task_a", "Hanging Task", NodeType::Task, Some("hanging"))?;
    graph.try_add_node("end", "End Node", NodeType::End, None)?;
    graph.try_add_edge("start", "task_a", None, 1)?;
    graph.try_add_edge("task_a", "end", None, 1)?;

    let mut ctx = AgentContext::try_new(
        "timeout_session",
        "start",
        db,
        state_coll,
        TokenBudget::new(1000, 0),
    )?;

    let res = engine.run(&mut ctx, &graph).await;
    assert!(res.is_err());
    match res {
        Err(MemFuseError::Timeout {
            operation,
            timeout_ms,
        }) => {
            assert_eq!(operation, "tool:hanging");
            assert_eq!(timeout_ms, 50);
        }
        _ => panic!("Expected MemFuseError::Timeout, got {:?}", res),
    }

    let dlq = engine.dead_letter_queue.as_ref().unwrap();
    let letters = dlq.list().await?;
    assert!(!letters.is_empty(), "DLQ should contain at least one entry");

    let letter = &letters[0];
    assert_eq!(letter.session_id, "timeout_session");
    assert_eq!(letter.node_id, "task_a");
    match &letter.failure_reason {
        DeadLetterReason::Timeout { timeout_ms } => {
            assert_eq!(*timeout_ms, 50);
        }
        _ => panic!(
            "Expected DeadLetterReason::Timeout, got {:?}",
            letter.failure_reason
        ),
    }

    let drained = dlq.drain().await?;
    assert_eq!(drained.len(), letters.len());

    let list_after_drain = dlq.list().await?;
    assert!(
        list_after_drain.is_empty(),
        "DLQ should be empty after drain"
    );

    Ok(())
}

#[tokio::test]
async fn test_dlq_replay_idempotency_scenarios() -> Result<()> {
    use memfuse_agent::step::StepDeadLetter;
    use memfuse_core::TxId;

    let temp_dir = tempfile::TempDir::new()?;
    let config = memfuse_db::MemFuseConfig::default();
    let db = Arc::new(memfuse_db::MemFuse::open_with_config(temp_dir.path(), config).await?);
    let state_coll = db.collection("dlq_idempotency_col").await?;

    let dlq = memfuse_agent::DeadLetterQueue::new(db.inner_storage());

    // Scenario (b): DLQ entry with TxId NOT committed in store
    let uncommitted_letter = StepDeadLetter {
        session_id: "idempotency_session".to_string(),
        node_id: "node_step_1".to_string(),
        step_index: 0,
        tx_id: Some(TxId::new(9999)),
        failure_reason: DeadLetterReason::ToolError {
            message: "Failed before commit".to_string(),
        },
        input: serde_json::json!({"action": "test"}),
        attempt: 0,
        failed_at_secs: 1000,
    };

    let is_committed_before = dlq.is_already_committed(&uncommitted_letter).await?;
    assert!(
        !is_committed_before,
        "Uncommitted TxId must return false for is_already_committed"
    );

    // Commit a state document into state_coll with TxId 500
    let state_doc_id = "task:idempotency_session:step:0";
    state_coll
        .put_kv(
            state_doc_id,
            &serde_json::json!({
                "stage": "commit",
                "node": "node_step_1",
                "tx_id": 500
            }),
        )
        .await?;

    // Scenario (a): DLQ entry with TxId matching committed TxId in store
    let committed_letter = StepDeadLetter {
        session_id: "idempotency_session".to_string(),
        node_id: "node_step_1".to_string(),
        step_index: 0,
        tx_id: Some(TxId::new(500)),
        failure_reason: DeadLetterReason::ToolError {
            message: "Failed after commit".to_string(),
        },
        input: serde_json::json!({"action": "test"}),
        attempt: 0,
        failed_at_secs: 1000,
    };

    let is_committed_after = dlq.is_already_committed(&committed_letter).await?;
    assert!(
        is_committed_after,
        "Committed TxId in store must return true for is_already_committed (Replay is No-Op)"
    );

    // Scenario (c): Consecutive calls for the same committed DLQ entry yield identical deterministic outcome
    let second_check = dlq.is_already_committed(&committed_letter).await?;
    assert!(
        second_check,
        "Consecutive idempotency checks must produce identical true result without side effects"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_allocate_tx_uniqueness() -> Result<()> {
    use memfuse_agent::DeadLetterQueue;
    use std::collections::HashSet;

    let temp_dir = tempfile::TempDir::new()?;
    let config = memfuse_db::MemFuseConfig::default();
    let db = Arc::new(memfuse_db::MemFuse::open_with_config(temp_dir.path(), config).await?);
    let lsm_storage = db.inner_storage();

    let dlq = Arc::new(DeadLetterQueue::new(lsm_storage));

    let mut handles = Vec::new();
    for _ in 0..50 {
        let dlq_clone = dlq.clone();
        handles.push(tokio::spawn(async move { dlq_clone.allocate_tx().await }));
    }

    let mut tx_ids = Vec::new();
    for handle in handles {
        let tx_id = handle
            .await
            .map_err(|e| MemFuseError::Internal(e.to_string()))??;
        tx_ids.push(tx_id);
    }

    assert_eq!(tx_ids.len(), 50);

    let unique_tx_ids: HashSet<_> = tx_ids.iter().cloned().collect();
    assert_eq!(
        unique_tx_ids.len(),
        50,
        "All 50 allocated TxIds under concurrent calls must be distinct"
    );

    Ok(())
}

#[tokio::test]
async fn test_tool_retry_succeeds_on_second_attempt() -> Result<()> {
    struct FlakeyTool {
        attempt: AtomicU32,
    }
    impl AgentTool for FlakeyTool {
        fn name(&self) -> &str {
            "flakey"
        }
        fn is_retriable(&self) -> bool {
            true
        }
        fn execute<'a>(
            &'a self,
            _: &'a AgentContext,
            _: serde_json::Value,
        ) -> BoxFuture<'a, Result<StepResult>> {
            Box::pin(async move {
                let n = self.attempt.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    Err(MemFuseError::Internal("transient failure".into()))
                } else {
                    Ok(StepResult {
                        node_id: "task_a".to_string(),
                        output: serde_json::json!({"status": "ok"}),
                        tokens_consumed: 10,
                        next_edge: None,
                    })
                }
            })
        }
    }

    let temp_dir = tempfile::TempDir::new()?;
    let config = memfuse_db::MemFuseConfig::default();
    let db = Arc::new(memfuse_db::MemFuse::open_with_config(temp_dir.path(), config).await?);
    let state_coll = db.collection("dlq_retry_col").await?;

    let mut engine = OrchestratorEngine::try_from_db(&db)?;
    let flakey_tool = FlakeyTool {
        attempt: AtomicU32::new(0),
    };
    engine.try_register_tool(Box::new(flakey_tool))?;

    let mut graph = StateGraph::new();
    graph.try_add_node("start", "Start Node", NodeType::Start, None)?;
    graph.try_add_node("task_a", "Flakey Task", NodeType::Task, Some("flakey"))?;
    graph.try_add_node("end", "End Node", NodeType::End, None)?;
    graph.try_add_edge("start", "task_a", None, 1)?;
    graph.try_add_edge("task_a", "end", None, 1)?;

    let mut ctx = AgentContext::try_new(
        "retry_session",
        "start",
        db,
        state_coll,
        TokenBudget::new(1000, 0),
    )?;

    let res = engine.run(&mut ctx, &graph).await;
    assert!(res.is_ok(), "Run should succeed on second attempt");

    let dlq = engine.dead_letter_queue.as_ref().unwrap();
    let letters = dlq.list().await?;
    assert!(
        letters.is_empty(),
        "DLQ should be empty when workflow succeeds after retry"
    );

    Ok(())
}
