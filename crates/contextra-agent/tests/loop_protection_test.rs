// FILE-CONTEXT
// STAND: 2026-10-29T00:00:00Z
// ZWECK: Loop protection (Schleifenschutz) tests ensuring guaranteed termination via max steps or budget limits.

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
    TokenBudget, MAX_WORKFLOW_STEPS,
};
use contextra_db::{Contextra, ContextraConfig, DistanceMetric};
use contextra_ports::BoxFuture;
use contextra_types::Result;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

struct NeverDoneTool;

impl AgentTool for NeverDoneTool {
    fn name(&self) -> &str {
        "never_done_tool"
    }

    fn estimated_cost(&self, _input: &serde_json::Value) -> usize {
        10
    }

    fn execute<'a>(
        &'a self,
        _ctx: &'a AgentContext,
        _input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        Box::pin(async move {
            Ok(StepResult {
                node_id: "never_done".to_string(),
                output: json!({
                    "status": {
                        "done": false
                    }
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
async fn test_loop_protection_max_workflow_steps_termination() {
    let tmp = TempDir::new().expect("temp dir");
    let db = create_test_db(&tmp).await;
    let state_col = db.collection("loop-max-steps").await.expect("col");

    // Infinite loop graph with never-satisfied goal condition on exit edge
    let mut graph = StateGraph::new();
    graph
        .try_add_node("start", "Start", NodeType::Start, None)
        .expect("start");
    graph
        .try_add_node(
            "loop_node",
            "Loop Node",
            NodeType::Task,
            Some("never_done_tool"),
        )
        .expect("loop_node");
    graph
        .try_add_node("end", "End", NodeType::End, None)
        .expect("end");

    // Loop edge back to loop_node with lower priority
    graph
        .try_add_edge("loop_node", "loop_node", None, 1)
        .expect("loop edge");
    // Exit edge requiring status.done == true with higher priority
    graph
        .try_add_edge("loop_node", "end", Some("status.done == true"), 2)
        .expect("exit edge");
    graph
        .try_add_edge("start", "loop_node", None, 1)
        .expect("start edge");

    let mut ctx = AgentContext::try_new(
        "task-infinite-loop",
        "start",
        db.clone(),
        state_col.clone(),
        // Generous budget so max workflow step limit is hit first
        TokenBudget::new(100_000_000, 0),
    )
    .expect("ctx");

    let mut orchestrator = OrchestratorEngine::try_from_db(&db).expect("engine");
    orchestrator
        .try_register_tool(Box::new(NeverDoneTool))
        .expect("register tool");

    let run_res = orchestrator.run(&mut ctx, &graph).await;
    assert!(run_res.is_err(), "Loop must terminate with an error");
    assert_eq!(
        ctx.status,
        AgentStatus::Failed,
        "Agent status must be Failed"
    );

    let err_msg = run_res.unwrap_err().to_string();
    assert!(
        err_msg.contains("Maximum workflow step limit"),
        "Error message must indicate step limit exhaustion, got: {err_msg}"
    );
    assert_eq!(
        ctx.step_count, MAX_WORKFLOW_STEPS,
        "Step count must reach MAX_WORKFLOW_STEPS limit"
    );
}

#[tokio::test]
async fn test_loop_protection_budget_exhaustion_termination() {
    let tmp = TempDir::new().expect("temp dir");
    let db = create_test_db(&tmp).await;
    let state_col = db.collection("loop-budget").await.expect("col");

    let mut graph = StateGraph::new();
    graph
        .try_add_node("start", "Start", NodeType::Start, None)
        .expect("start");
    graph
        .try_add_node(
            "loop_node",
            "Loop Node",
            NodeType::Task,
            Some("never_done_tool"),
        )
        .expect("loop_node");
    graph
        .try_add_node("end", "End", NodeType::End, None)
        .expect("end");

    graph
        .try_add_edge("loop_node", "loop_node", None, 1)
        .expect("loop edge");
    graph
        .try_add_edge("loop_node", "end", Some("status.done == true"), 2)
        .expect("exit edge");
    graph
        .try_add_edge("start", "loop_node", None, 1)
        .expect("start edge");

    // Constrained budget (allows only ~3 steps at cost 10 each)
    let mut ctx = AgentContext::try_new(
        "task-budget-loop",
        "start",
        db.clone(),
        state_col.clone(),
        TokenBudget::new(30, 0),
    )
    .expect("ctx");

    let mut orchestrator = OrchestratorEngine::try_from_db(&db).expect("engine");
    orchestrator
        .try_register_tool(Box::new(NeverDoneTool))
        .expect("register tool");

    let run_res = orchestrator.run(&mut ctx, &graph).await;
    assert!(run_res.is_err(), "Loop must terminate with budget error");
    assert_eq!(ctx.status, AgentStatus::Failed);

    let err_msg = run_res.unwrap_err().to_string();
    assert!(
        err_msg.to_lowercase().contains("budget"),
        "Expected budget exhaustion error message, got: {err_msg}"
    );
    assert!(
        ctx.step_count < MAX_WORKFLOW_STEPS,
        "Budget exhaustion must halt execution long before max step limit"
    );
}
