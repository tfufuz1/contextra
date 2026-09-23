#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use memfuse_agent::{
    AgentContext, AgentTool, BackgroundEvent, DeadLetterReason, NodeType, OrchestratorEngine,
    StateGraph, StepResult, MAX_WORKFLOW_STEPS,
};
use memfuse_core::{MemFuseError, TokenBudget};
use memfuse_db::{DistanceMetric, MemFuse, MemFuseConfig};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

async fn setup_env() -> (OrchestratorEngine, Arc<MemFuse>, AgentContext, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let config = MemFuseConfig {
        dimension: 4,
        max_elements: 1000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let db = Arc::new(
        MemFuse::open_with_config(tmp.path(), config)
            .await
            .expect("open db"),
    );
    let state_col = db.collection("test-state").await.expect("collection");
    let ctx = AgentContext::try_new(
        "test-task-regression",
        "start",
        db.clone(),
        state_col,
        TokenBudget::new(1000, 0),
    )
    .expect("agent context");
    let engine = OrchestratorEngine::try_from_db(&db).expect("engine try_from_db");
    (engine, db, ctx, tmp)
}

struct FailingRetriableTool {
    exec_count: Arc<AtomicUsize>,
}

impl AgentTool for FailingRetriableTool {
    fn name(&self) -> &str {
        "failing_retriable_tool"
    }

    fn is_retriable(&self) -> bool {
        true
    }

    fn max_retries(&self) -> u32 {
        5
    }

    fn execute<'a>(
        &'a self,
        _ctx: &'a AgentContext,
        _input: serde_json::Value,
    ) -> memfuse_core::BoxFuture<'a, memfuse_core::Result<StepResult>> {
        let count = self.exec_count.clone();
        Box::pin(async move {
            count.fetch_add(1, Ordering::SeqCst);
            Err(MemFuseError::Internal(
                "Simulated transient tool failure".to_string(),
            ))
        })
    }
}

#[tokio::test]
async fn test_regression_zero_panic_doctrine_no_panics_on_invalid_inputs() {
    let (mut engine, _db, _ctx, _tmp) = setup_env().await;

    // 1. Deprecated/legacy constructors and helpers must not panic
    let event = BackgroundEvent::try_new(json!({}), "", 1);
    assert!(event.is_err());

    let mut graph = StateGraph::new();
    // Invalid node addition via try_add_node should return an error, not panic
    assert!(graph
        .try_add_node("", "Invalid node", NodeType::Task, None)
        .is_err());
    assert!(graph.get_node("").is_none());

    // Invalid edge addition via try_add_edge should return an error, not panic
    assert!(graph.try_add_edge("", "end", None, 1).is_err());
    assert!(graph.edges.is_empty());

    // Registered tool with invalid name
    let mock = FailingRetriableTool {
        exec_count: Arc::new(AtomicUsize::new(0)),
    };
    assert!(engine.try_register_tool(Box::new(mock)).is_ok());
}

#[tokio::test]
async fn test_regression_unbounded_loop_termination() {
    let (engine, _db, mut ctx, _tmp) = setup_env().await;

    // Create a graph with an infinite loop A -> B -> A using Start nodes (which pass-through without handlers)
    let mut graph = StateGraph::new();
    graph
        .try_add_node("start", "Start Node", NodeType::Start, None)
        .unwrap();
    graph
        .try_add_node("node_a", "Node A", NodeType::Start, None)
        .unwrap();
    graph
        .try_add_node("node_b", "Node B", NodeType::Start, None)
        .unwrap();

    graph.try_add_edge("start", "node_a", None, 1).unwrap();
    graph.try_add_edge("node_a", "node_b", None, 1).unwrap();
    graph.try_add_edge("node_b", "node_a", None, 1).unwrap();

    ctx.step_count = MAX_WORKFLOW_STEPS - 1;
    ctx.current_node = "node_a".to_string();

    let res = engine.run(&mut ctx, &graph).await;
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("Maximum workflow step limit of 10000 exceeded"),
        "Expected step limit error message, got: {err_msg}"
    );
}

#[tokio::test]
async fn test_regression_budget_exhaustion_halts_retries() {
    let (mut engine, _db, mut ctx, _tmp) = setup_env().await;

    let exec_count = Arc::new(AtomicUsize::new(0));
    let tool = FailingRetriableTool {
        exec_count: exec_count.clone(),
    };
    engine.try_register_tool(Box::new(tool)).unwrap();

    let mut graph = StateGraph::new();
    graph
        .try_add_node("start", "Start Node", NodeType::Start, None)
        .unwrap();
    graph
        .try_add_node(
            "task",
            "Task Node",
            NodeType::Task,
            Some("failing_retriable_tool"),
        )
        .unwrap();
    graph
        .try_add_node("end", "End Node", NodeType::End, None)
        .unwrap();

    graph.try_add_edge("start", "task", None, 1).unwrap();
    graph.try_add_edge("task", "end", None, 1).unwrap();

    // Deplete budget completely before run
    ctx.budget.consume(1000);
    assert_eq!(ctx.budget.available(), 0);

    let res = engine.run(&mut ctx, &graph).await;
    assert!(res.is_err());

    // Tool should be executed at most once (for attempt 0 before budget check or pre-check),
    // and subsequent retries must be halted because available budget is 0.
    assert!(
        exec_count.load(Ordering::SeqCst) <= 1,
        "Tool execution count should be <= 1 when budget is depleted, got: {}",
        exec_count.load(Ordering::SeqCst)
    );

    // Verify DLQ letter logged budget exhaustion
    if let Some(ref dlq) = engine.dead_letter_queue {
        let letters = dlq.list().await.unwrap();
        assert!(!letters.is_empty());
        assert!(matches!(
            letters[0].failure_reason,
            DeadLetterReason::BudgetExhausted { .. }
        ));
    }
}
