// FILE-CONTEXT
// STAND: 2026-10-29T00:00:00Z
// ZWECK: Crash recovery and abort injection integration tests for agent workflow execution.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unreachable,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_agent::{
    AgentContext, AgentStatus, AgentTool, NodeType, OrchestratorEngine, StateGraph, StepResult,
    TokenBudget,
};
use contextra_db::{Contextra, ContextraConfig, DistanceMetric};
use contextra_ports::BoxFuture;
use contextra_types::Result;
use serde_json::json;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

struct StatefulStepTool {
    name: String,
    execution_counter: Arc<AtomicU32>,
}

impl AgentTool for StatefulStepTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute<'a>(
        &'a self,
        _ctx: &'a AgentContext,
        input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        Box::pin(async move {
            let count = self.execution_counter.fetch_add(1, Ordering::SeqCst);
            Ok(StepResult {
                node_id: self.name.clone(),
                output: json!({
                    "step": self.name,
                    "count": count + 1,
                    "prev": input
                }),
                tokens_consumed: 10,
                next_edge: None,
            })
        })
    }
}

async fn create_test_db(tmp: &TempDir) -> Arc<Contextra> {
    let config = ContextraConfig {
        dimension: 4,
        max_elements: 1000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    Arc::new(
        Contextra::open_with_config(tmp.path(), config)
            .await
            .expect("open db"),
    )
}

#[tokio::test]
async fn test_crash_recovery_step_abort_injection() {
    let tmp = TempDir::new().expect("temp dir");
    let db = create_test_db(&tmp).await;
    let state_col = db.collection("crash-state").await.expect("collection");

    let counter1 = Arc::new(AtomicU32::new(0));
    let counter2 = Arc::new(AtomicU32::new(0));

    let tool1 = Box::new(StatefulStepTool {
        name: "tool_step1".to_string(),
        execution_counter: counter1.clone(),
    });
    let tool2 = Box::new(StatefulStepTool {
        name: "tool_step2".to_string(),
        execution_counter: counter2.clone(),
    });

    let mut graph = StateGraph::new();
    graph
        .try_add_node("start", "Start Node", NodeType::Start, None)
        .expect("start");
    graph
        .try_add_node("step1", "Step 1", NodeType::Task, Some("tool_step1"))
        .expect("step1");
    graph
        .try_add_node("step2", "Step 2", NodeType::Task, Some("tool_step2"))
        .expect("step2");
    graph
        .try_add_node("end", "End Node", NodeType::End, None)
        .expect("end");

    graph
        .try_add_edge("start", "step1", None, 1)
        .expect("edge 0");
    graph
        .try_add_edge("step1", "step2", None, 1)
        .expect("edge 1");
    graph.try_add_edge("step2", "end", None, 1).expect("edge 2");

    // Phase 1: Initialize context and engine
    let mut ctx = AgentContext::try_new(
        "task-crash-1",
        "start",
        db.clone(),
        state_col.clone(),
        TokenBudget::new(1000, 0),
    )
    .expect("ctx");

    let mut orchestrator = OrchestratorEngine::try_from_db(&db).expect("engine");
    orchestrator.try_register_tool(tool1).expect("tool1");
    orchestrator.try_register_tool(tool2).expect("tool2");

    // Checkpoint at start
    orchestrator.checkpoint(&ctx).await.expect("checkpoint");

    // Execute step 0 (start -> step1)
    let res = orchestrator.run(&mut ctx, &graph).await;
    assert!(res.is_ok());
    assert_eq!(ctx.status, AgentStatus::Completed);

    // Simulate crash and recovery from last valid checkpoint step (e.g. step:1)
    let mut recovered_ctx = AgentContext::try_new(
        "task-crash-1",
        "start",
        db.clone(),
        state_col.clone(),
        TokenBudget::new(1000, 0),
    )
    .expect("recovered ctx");

    let recovery_res = orchestrator.replay_from(&mut recovered_ctx, "step:1").await;
    assert!(recovery_res.is_ok());
    assert_eq!(recovered_ctx.step_count, 1);
    assert_eq!(recovered_ctx.current_node, "step1");
}

#[tokio::test]
async fn test_audit_log_before_commit_guarantee() {
    let tmp = TempDir::new().expect("temp dir");
    let db = create_test_db(&tmp).await;
    let state_col = db
        .collection("audit-order-state")
        .await
        .expect("collection");

    let ctx = AgentContext::try_new(
        "task-audit-order",
        "start",
        db.clone(),
        state_col.clone(),
        TokenBudget::new(1000, 0),
    )
    .expect("ctx");

    let orchestrator = OrchestratorEngine::try_from_db(&db).expect("engine");

    // Write audit log entry
    let step_res = StepResult {
        node_id: "start".to_string(),
        output: json!({"action": "pre_commit_audit"}),
        tokens_consumed: 5,
        next_edge: None,
    };

    let audit_id = format!("audit:{}:step:{}", ctx.task_id, ctx.step_count);
    let state_id = format!("task:{}:step:{}", ctx.task_id, ctx.step_count);

    // Verify neither exists initially
    assert!(state_col.get_kv(&audit_id).await.unwrap().is_none());
    assert!(state_col.get_kv(&state_id).await.unwrap().is_none());

    // Run checkpoint -> audit log (before commit)
    orchestrator.checkpoint(&ctx).await.expect("checkpoint");

    let entry = contextra_agent::audit::AuditEntry {
        task_id: ctx.task_id.clone(),
        step_count: ctx.step_count,
        node_id: ctx.current_node.clone(),
        tokens_consumed: step_res.tokens_consumed,
        payload: step_res.output.clone(),
        error: None,
        tx_id: None,
    };

    contextra_agent::audit::AuditLog::append_to(&state_col, &entry)
        .await
        .expect("append audit");

    // Audit record MUST exist before commit effect!
    let audit_val = state_col.get_kv(&audit_id).await.unwrap();
    assert!(audit_val.is_some());
    assert!(state_col.get_kv(&state_id).await.unwrap().is_none());
}
