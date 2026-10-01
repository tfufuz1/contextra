use contextra_agent::{
    AgentContext, AgentTool, DeadLetterReason, NodeType, OrchestratorEngine, StateGraph, StepResult,
};
use contextra_ports::{BoxFuture, Clock};
use contextra_types::{ContextraError, Result, TokenBudget};
use std::sync::Arc;

struct TestClock {
    nanos: u64,
}

impl Clock for TestClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos
    }
}

struct AlwaysFailingTool;

impl AgentTool for AlwaysFailingTool {
    fn name(&self) -> &str {
        "always_failing"
    }

    fn is_retriable(&self) -> bool {
        true
    }

    fn max_retries(&self) -> u32 {
        2
    }

    fn execute<'a>(
        &'a self,
        _: &'a AgentContext,
        _: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        Box::pin(async move { Err(ContextraError::Internal("always fails".to_string())) })
    }
}

#[tokio::test]
async fn test_dlq_failed_at_secs_uses_injected_clock() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let config = contextra_db::ContextraConfig::default();
    let db = Arc::new(contextra_db::Contextra::open_with_config(temp_dir.path(), config).await?);
    let state_coll = db.collection("dlq_clock_test_col").await?;

    // Set clock to 1,700_000_000 seconds in nanoseconds
    let expected_secs: u64 = 1_700_000_000;
    let clock_nanos: u64 = expected_secs * 1_000_000_000;
    let test_clock = Arc::new(TestClock { nanos: clock_nanos });

    let mut engine = OrchestratorEngine::try_from_db(&db)?.with_clock(test_clock);
    engine.try_register_tool(Box::new(AlwaysFailingTool))?;

    let mut graph = StateGraph::new();
    graph.try_add_node("start", "Start Node", NodeType::Start, None)?;
    graph.try_add_node("task_fail", "Failing Task", NodeType::Task, Some("always_failing"))?;
    graph.try_add_node("end", "End Node", NodeType::End, None)?;
    graph.try_add_edge("start", "task_fail", None, 1)?;
    graph.try_add_edge("task_fail", "end", None, 1)?;

    let mut ctx = AgentContext::try_new(
        "dlq_clock_session",
        "start",
        db,
        state_coll,
        TokenBudget::new(1000, 0),
    )?;

    let res = engine.run(&mut ctx, &graph).await;
    assert!(res.is_err(), "Workflow execution should fail");

    let dlq = engine
        .dead_letter_queue
        .as_ref()
        .expect("Dead letter queue should be initialized");
    let letters = dlq.list().await?;
    assert_eq!(letters.len(), 1, "DLQ should contain exactly one letter");

    let letter = &letters[0];
    assert_eq!(letter.session_id, "dlq_clock_session");
    assert_eq!(letter.node_id, "task_fail");
    match &letter.failure_reason {
        DeadLetterReason::MaxRetriesExceeded { attempts } => {
            assert_eq!(*attempts, 3);
        }
        _ => panic!(
            "Expected DeadLetterReason::MaxRetriesExceeded, got {:?}",
            letter.failure_reason
        ),
    }

    assert_eq!(
        letter.failed_at_secs, expected_secs,
        "failed_at_secs must match injected clock timestamp, got {}, expected {}",
        letter.failed_at_secs, expected_secs
    );

    Ok(())
}
