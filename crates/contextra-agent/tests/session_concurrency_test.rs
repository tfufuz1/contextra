//! Concurrency- und Isolation-Tests für parallele Agent-Sessions & Dead-Letter-Queue.
//!
//! Verifiziert:
//! 1. Parallele Agent-Sessions interferieren nicht miteinander (State- und Speicher-Isolation).
//! 2. Fehlerhafte Tool-Ausführungen landen deterministisch in der Dead-Letter-Queue (DLQ).
//! 3. Parallele fehlgeschlagene Sessions führen zu isolierten und korrekten DLQ-Einträgen.

use contextra_agent::step::{AgentTool, DeadLetterReason, StepResult};
use contextra_agent::{AgentContext, NodeType, OrchestratorEngine, StateGraph};
use contextra_ports::BoxFuture;
use contextra_types::{ContextraError, Result, TokenBudget};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

struct ConcurrentWorkerTool {
    counter: Arc<AtomicU32>,
}

impl AgentTool for ConcurrentWorkerTool {
    fn name(&self) -> &str {
        "concurrent_worker"
    }

    fn execute<'a>(
        &'a self,
        ctx: &'a AgentContext,
        input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        Box::pin(async move {
            self.counter.fetch_add(1, Ordering::SeqCst);
            let worker_id = input.get("worker_id").and_then(|v| v.as_u64()).unwrap_or(0);

            // Verifiziere Session-Isolation über die task_id im Context
            let expected_prefix = "session_";
            if !ctx.task_id.starts_with(expected_prefix) {
                return Err(ContextraError::InvalidInput(format!(
                    "Unexpected task_id: {}",
                    ctx.task_id
                )));
            }

            Ok(StepResult {
                node_id: "work_node".to_string(),
                output: serde_json::json!({
                    "task_id": ctx.task_id,
                    "worker_id": worker_id,
                    "completed": true
                }),
                tokens_consumed: 15,
                next_edge: None,
            })
        })
    }
}

struct FailingWorkerTool;

impl AgentTool for FailingWorkerTool {
    fn name(&self) -> &str {
        "failing_worker"
    }

    fn execute<'a>(
        &'a self,
        ctx: &'a AgentContext,
        _input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        Box::pin(async move {
            Err(ContextraError::Internal(format!(
                "Intentional tool error in session {}",
                ctx.task_id
            )))
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_parallel_agent_sessions_isolation() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let config = contextra_db::ContextraConfig::default();
    let db = Arc::new(contextra_db::Contextra::open_with_config(temp_dir.path(), config).await?);
    let state_coll = db.collection("concurrent_agent_state").await?;

    let counter = Arc::new(AtomicU32::new(0));
    let mut engine = OrchestratorEngine::try_from_db(&db)?;
    engine.try_register_tool(Box::new(ConcurrentWorkerTool {
        counter: counter.clone(),
    }))?;
    let engine = Arc::new(engine);

    let mut graph = StateGraph::new();
    graph.try_add_node("start", "Start Node", NodeType::Start, None)?;
    graph.try_add_node(
        "work_node",
        "Worker Node",
        NodeType::Task,
        Some("concurrent_worker"),
    )?;
    graph.try_add_node("end", "End Node", NodeType::End, None)?;
    graph.try_add_edge("start", "work_node", None, 1)?;
    graph.try_add_edge("work_node", "end", None, 1)?;
    let graph = Arc::new(graph);

    let mut handles = Vec::new();
    const NUM_CONCURRENT_SESSIONS: usize = 10;

    for i in 0..NUM_CONCURRENT_SESSIONS {
        let db_clone = db.clone();
        let state_coll_clone = state_coll.clone();
        let engine_clone = engine.clone();
        let graph_clone = graph.clone();

        handles.push(tokio::spawn(async move {
            let session_id = format!("session_{i}");
            let mut ctx = AgentContext::try_new(
                session_id.clone(),
                "start",
                db_clone,
                state_coll_clone,
                TokenBudget::new(2000, 0),
            )?;

            ctx.memory
                .insert("session_key".to_string(), serde_json::json!(i));

            engine_clone.run(&mut ctx, &graph_clone).await?;

            assert_eq!(ctx.status, contextra_agent::context::AgentStatus::Completed);
            assert_eq!(
                ctx.memory.get("session_key"),
                Some(&serde_json::json!(i)),
                "Memory isolation must be preserved for session {i}"
            );

            Ok::<(), ContextraError>(())
        }));
    }

    for handle in handles {
        let res = handle
            .await
            .map_err(|e| ContextraError::Internal(e.to_string()))?;
        res?;
    }

    assert_eq!(
        counter.load(Ordering::SeqCst),
        NUM_CONCURRENT_SESSIONS as u32,
        "Every session must execute exactly once"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_parallel_tool_execution_failure_dlq_routing() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let config = contextra_db::ContextraConfig::default();
    let db = Arc::new(contextra_db::Contextra::open_with_config(temp_dir.path(), config).await?);
    let state_coll = db.collection("dlq_parallel_state").await?;

    let mut engine = OrchestratorEngine::try_from_db(&db)?;
    engine.try_register_tool(Box::new(FailingWorkerTool))?;

    let mut graph = StateGraph::new();
    graph.try_add_node("start", "Start Node", NodeType::Start, None)?;
    graph.try_add_node(
        "failing_node",
        "Failing Node",
        NodeType::Task,
        Some("failing_worker"),
    )?;
    graph.try_add_node("end", "End Node", NodeType::End, None)?;
    graph.try_add_edge("start", "failing_node", None, 1)?;
    graph.try_add_edge("failing_node", "end", None, 1)?;

    let engine = Arc::new(engine);
    let graph = Arc::new(graph);

    let mut handles = Vec::new();
    const NUM_FAILING_SESSIONS: usize = 5;

    for i in 0..NUM_FAILING_SESSIONS {
        let db_clone = db.clone();
        let state_coll_clone = state_coll.clone();
        let engine_clone = engine.clone();
        let graph_clone = graph.clone();

        handles.push(tokio::spawn(async move {
            let session_id = format!("fail_session_{i}");
            let mut ctx = AgentContext::try_new(
                session_id,
                "start",
                db_clone,
                state_coll_clone,
                TokenBudget::new(1000, 0),
            )?;

            let res = engine_clone.run(&mut ctx, &graph_clone).await;
            assert!(res.is_err(), "Run must fail due to failing tool");
            assert_eq!(ctx.status, contextra_agent::context::AgentStatus::Failed);

            Ok::<(), ContextraError>(())
        }));
    }

    for handle in handles {
        let res = handle
            .await
            .map_err(|e| ContextraError::Internal(e.to_string()))?;
        res?;
    }

    let dlq = engine.dead_letter_queue.as_ref().expect("DLQ exists");
    let letters = dlq.list().await?;

    assert_eq!(
        letters.len(),
        NUM_FAILING_SESSIONS,
        "DLQ must capture exactly one entry per failed session"
    );

    for letter in &letters {
        assert!(letter.session_id.starts_with("fail_session_"));
        assert_eq!(letter.node_id, "failing_node");
        match &letter.failure_reason {
            DeadLetterReason::ToolError { message } => {
                assert!(message.contains("Intentional tool error"));
            }
            other => panic!("Expected ToolError, got {:?}", other),
        }
    }

    // Drain DLQ and verify empty state
    let drained = dlq.drain().await?;
    assert_eq!(drained.len(), NUM_FAILING_SESSIONS);
    let remaining = dlq.list().await?;
    assert!(remaining.is_empty(), "DLQ must be empty after drain");

    Ok(())
}
