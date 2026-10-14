use contextra_agent::wiring::AgentWiring;
use contextra_agent::{
    AgentContext, AgentStatus, AgentTool, NodeType, StateGraph, StepResult,
};
use contextra_ports::{BoxFuture, Clock, IdGenerator, SequentialIdGen, SystemClock};
use contextra_types::{ContextraError, Result, TokenBudget};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

struct IdempotentMockTool {
    call_count: Arc<AtomicU32>,
}

impl AgentTool for IdempotentMockTool {
    fn name(&self) -> &str {
        "idempotent_tool"
    }

    fn is_retriable(&self) -> bool {
        true
    }

    fn max_retries(&self) -> u32 {
        3
    }

    fn execute<'a>(
        &'a self,
        _ctx: &'a AgentContext,
        _input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        let call_count = self.call_count.clone();
        Box::pin(async move {
            let cnt = call_count.fetch_add(1, Ordering::SeqCst);
            if cnt == 0 {
                Err(ContextraError::Internal("Transient network error".to_string()))
            } else {
                Ok(StepResult {
                    node_id: "step1".to_string(),
                    output: serde_json::json!({ "status": "ok", "call_count": cnt + 1 }),
                    tokens_consumed: 10,
                    next_edge: None,
                })
            }
        })
    }
}

#[tokio::test]
async fn test_agent_wiring_build_and_tool_retry() -> Result<()> {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = contextra_db::ContextraConfig::default();
    let db = Arc::new(contextra_db::Contextra::open_with_config(temp_dir.path(), config).await?);
    let state_coll = db.collection("test_wiring").await?;

    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
    let id_gen: Arc<dyn IdGenerator> = Arc::new(SequentialIdGen::new(1));

    let mut wiring = AgentWiring::build(
        db.clone(),
        state_coll.clone(),
        TokenBudget::new(1000, 0),
        clock,
        id_gen,
    )?;

    assert_eq!(wiring.budget.limit, 1000);

    let call_count = Arc::new(AtomicU32::new(0));
    wiring
        .orchestrator
        .try_register_tool(Box::new(IdempotentMockTool {
            call_count: call_count.clone(),
        }))?;

    let mut graph = StateGraph::new();
    graph.try_add_node("start", "Start Node", NodeType::Start, None)?;
    graph.try_add_node(
        "step1",
        "Step 1 Task",
        NodeType::Task,
        Some("idempotent_tool"),
    )?;
    graph.try_add_node("end", "End Node", NodeType::End, None)?;
    graph.try_add_edge("start", "step1", None, 1)?;
    graph.try_add_edge("step1", "end", None, 1)?;

    let mut ctx = AgentContext::try_new(
        "wiring_task_1",
        "start",
        db.clone(),
        state_coll,
        wiring.budget.clone(),
    )?;

    let run_res = wiring.orchestrator.run(&mut ctx, &graph).await;
    assert!(run_res.is_ok());

    // Tool failed on attempt 0 (call_count 0 -> 1) and succeeded on attempt 1 (call_count 1 -> 2)
    assert_eq!(call_count.load(Ordering::SeqCst), 2);
    assert_eq!(ctx.status(), AgentStatus::Completed);
    assert_eq!(ctx.current_node, "end");

    Ok(())
}

#[tokio::test]
async fn test_agent_wiring_build_with_event_source() -> Result<()> {
    use contextra_agent::event_source::VecEventSource;

    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = contextra_db::ContextraConfig::default();
    let db = Arc::new(contextra_db::Contextra::open_with_config(temp_dir.path(), config).await?);
    let state_coll = db.collection("test_wiring_events").await?;

    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
    let id_gen: Arc<dyn IdGenerator> = Arc::new(SequentialIdGen::new(1));

    let custom_source = Box::new(VecEventSource::try_new(vec![])?);

    let wiring = AgentWiring::build_with_event_source(
        db,
        state_coll,
        TokenBudget::new(500, 0),
        clock,
        id_gen,
        custom_source,
    )?;

    assert_eq!(wiring.budget.limit, 500);
    Ok(())
}
