use contextra_agent::{
    AgentContext, AgentTool, NodeType, OrchestratorEngine, StateGraph, StepResult,
};
use contextra_ports::{BoxFuture, Clock};
use contextra_types::{ContextraError, Result, TokenBudget};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

struct TestClock {
    unix_nanos: AtomicU64,
}

impl TestClock {
    fn new(secs: u64) -> Self {
        Self {
            unix_nanos: AtomicU64::new(secs * 1_000_000_000),
        }
    }
}

impl Clock for TestClock {
    fn now_unix_nanos(&self) -> u64 {
        self.unix_nanos.load(Ordering::SeqCst)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.unix_nanos.load(Ordering::SeqCst)
    }
}

struct FailingTool;

impl AgentTool for FailingTool {
    fn name(&self) -> &str {
        "failing"
    }

    fn execute<'a>(
        &'a self,
        _: &'a AgentContext,
        _: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        Box::pin(async move {
            Err(ContextraError::Internal("intentional failure".to_string()))
        })
    }
}

#[tokio::test]
async fn test_orchestrator_deterministic_clock_failed_at_secs() -> Result<()> {
    let fixed_secs = 1_700_000_000u64;
    let clock = Arc::new(TestClock::new(fixed_secs));

    let tmp = TempDir::new()?;
    let config = contextra_db::ContextraConfig::default();
    let db = Arc::new(contextra_db::Contextra::open_with_config(tmp.path(), config).await?);
    let collection = db.collection("state").await?;

    let mut engine = OrchestratorEngine::try_from_db(&db)?.with_clock(clock);
    engine.try_register_tool(Box::new(FailingTool))?;

    let mut graph = StateGraph::new();
    graph.try_add_node("start", "Start", NodeType::Start, None)?;
    graph.try_add_node("task_fail", "Fail Task", NodeType::Task, Some("failing"))?;
    graph.try_add_edge("start", "task_fail", None, 1)?;

    let mut ctx = AgentContext::try_new(
        "clock_task_1",
        "start",
        db.clone(),
        collection.clone(),
        TokenBudget::new(100, 0),
    )?;

    let res = engine.run(&mut ctx, &graph).await;
    assert!(res.is_err());

    let dlq = engine.dead_letter_queue.as_ref().unwrap();
    let letters = dlq.list().await?;
    assert_eq!(letters.len(), 1);
    assert_eq!(letters[0].failed_at_secs, fixed_secs);

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_deterministic_clock_checkpoint_created_at() -> Result<()> {
    let fixed_secs = 1_800_000_000u64;
    let clock = Arc::new(TestClock::new(fixed_secs));

    let tmp = TempDir::new()?;
    let config = contextra_db::ContextraConfig::default();
    let db = Arc::new(contextra_db::Contextra::open_with_config(tmp.path(), config).await?);
    let collection = db.collection("state").await?;

    let engine = OrchestratorEngine::try_from_db(&db)?.with_clock(clock);

    let mut graph = StateGraph::new();
    graph.try_add_node("start", "Start", NodeType::Start, None)?;
    graph.try_add_node("end", "End", NodeType::End, None)?;
    graph.try_add_edge("start", "end", None, 1)?;

    let mut ctx = AgentContext::try_new(
        "clock_task_2",
        "start",
        db.clone(),
        collection.clone(),
        TokenBudget::new(100, 0),
    )?;

    engine.run(&mut ctx, &graph).await?;

    let checkpoints = engine.checkpoint_store.list_checkpoints().await?;
    assert!(!checkpoints.is_empty());
    for cp in &checkpoints {
        assert_eq!(cp.created_at, fixed_secs * 1000);
    }

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_deterministic_clock_identical_runs() -> Result<()> {
    let fixed_secs = 1_900_000_000u64;

    // Run 1
    let tmp1 = TempDir::new()?;
    let db1 = Arc::new(
        contextra_db::Contextra::open_with_config(tmp1.path(), Default::default()).await?,
    );
    let col1 = db1.collection("state").await?;
    let mut engine1 = OrchestratorEngine::try_from_db(&db1)?
        .with_clock(Arc::new(TestClock::new(fixed_secs)));
    engine1.try_register_tool(Box::new(FailingTool))?;

    let mut graph1 = StateGraph::new();
    graph1.try_add_node("start", "Start", NodeType::Start, None)?;
    graph1.try_add_node("task_fail", "Fail Task", NodeType::Task, Some("failing"))?;
    graph1.try_add_edge("start", "task_fail", None, 1)?;

    let mut ctx1 =
        AgentContext::try_new("task_same", "start", db1, col1, TokenBudget::new(100, 0))?;
    let _ = engine1.run(&mut ctx1, &graph1).await;

    let letters1 = engine1.dead_letter_queue.as_ref().unwrap().list().await?;
    let cps1 = engine1.checkpoint_store.list_checkpoints().await?;

    // Run 2
    let tmp2 = TempDir::new()?;
    let db2 = Arc::new(
        contextra_db::Contextra::open_with_config(tmp2.path(), Default::default()).await?,
    );
    let col2 = db2.collection("state").await?;
    let mut engine2 = OrchestratorEngine::try_from_db(&db2)?
        .with_clock(Arc::new(TestClock::new(fixed_secs)));
    engine2.try_register_tool(Box::new(FailingTool))?;

    let mut graph2 = StateGraph::new();
    graph2.try_add_node("start", "Start", NodeType::Start, None)?;
    graph2.try_add_node("task_fail", "Fail Task", NodeType::Task, Some("failing"))?;
    graph2.try_add_edge("start", "task_fail", None, 1)?;

    let mut ctx2 =
        AgentContext::try_new("task_same", "start", db2, col2, TokenBudget::new(100, 0))?;
    let _ = engine2.run(&mut ctx2, &graph2).await;

    let letters2 = engine2.dead_letter_queue.as_ref().unwrap().list().await?;
    let cps2 = engine2.checkpoint_store.list_checkpoints().await?;

    assert_eq!(letters1.len(), letters2.len());
    assert_eq!(letters1[0].failed_at_secs, letters2[0].failed_at_secs);
    assert_eq!(letters1[0].failed_at_secs, fixed_secs);

    assert_eq!(cps1.len(), cps2.len());
    for (cp1, cp2) in cps1.iter().zip(cps2.iter()) {
        assert_eq!(cp1.created_at, cp2.created_at);
        assert_eq!(cp1.created_at, fixed_secs * 1000);
    }

    Ok(())
}
