use contextra_agent::context::{AgentContext, AgentStatus};
use contextra_agent::engine::OrchestratorEngine;
use contextra_agent::graph::{NodeType, StateGraph};
use contextra_agent::step::{AgentTool, StepResult};
use contextra_db::{Contextra, ContextraConfig};
use contextra_ports::BoxFuture;
use contextra_types::TokenBudget;
use std::sync::Arc;
use tempfile::TempDir;

struct IncrementTool;

impl AgentTool for IncrementTool {
    fn name(&self) -> &str {
        "increment"
    }

    fn execute<'a>(
        &'a self,
        _ctx: &'a AgentContext,
        input: serde_json::Value,
    ) -> BoxFuture<'a, contextra_types::Result<StepResult>> {
        Box::pin(async move {
            let val = input.as_u64().unwrap_or(0);
            Ok(StepResult {
                node_id: "task_1".to_string(),
                output: serde_json::json!(val + 1),
                tokens_consumed: 10,
                next_edge: None,
            })
        })
    }
}

#[tokio::test]
async fn test_agent_persistence_and_recovery() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path();

    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };

    let db = Arc::new(Contextra::open_with_config(db_path, config).await.unwrap());
    let state_collection = db.collection("agent_state").await.unwrap();

    let mut graph = StateGraph::new();
    graph
        .try_add_node("start", "Start Node", NodeType::Start, None)
        .unwrap();
    graph
        .try_add_node(
            "task_1",
            "Increment Task",
            NodeType::Task,
            Some("increment"),
        )
        .unwrap();
    graph
        .try_add_node("end", "End Node", NodeType::End, None)
        .unwrap();

    graph.try_add_edge("start", "task_1", None, 1).unwrap();
    graph.try_add_edge("task_1", "end", None, 1).unwrap();

    let mut engine = OrchestratorEngine::try_new(db.inner_storage()).expect("engine try_new");
    engine.try_register_tool(Box::new(IncrementTool)).unwrap();

    let mut ctx = AgentContext::try_new(
        "test_task_123",
        "start",
        db.clone(),
        state_collection.clone(),
        TokenBudget::new(100, 0),
    )
    .unwrap();

    // Initial run
    engine.run(&mut ctx, &graph).await.expect("Run failed");

    assert_eq!(ctx.status, AgentStatus::Completed);
    assert_eq!(ctx.memory.get("last_output").unwrap().as_u64().unwrap(), 1);

    // Verify persistence in DB
    let final_doc = state_collection
        .get("task:test_task_123:final")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(final_doc.metadata.unwrap()["status"], "Completed");

    // Drop first DB handle and engine to simulate crash/restart boundary cleanly
    drop(state_collection);
    drop(ctx);
    drop(engine);
    drop(db);

    // Test Replay (Simulation of recovery)
    let db2 = Arc::new(
        Contextra::open_with_config(
            db_path,
            ContextraConfig {
                dimension: 4,
                ..Default::default()
            },
        )
        .await
        .unwrap(),
    );
    let state_collection2 = db2.collection("agent_state").await.unwrap();

    let mut ctx2 = AgentContext::try_new(
        "test_task_123",
        "task_1", // Start from task_1 for replay
        db2.clone(),
        state_collection2.clone(),
        TokenBudget::new(100, 0),
    )
    .unwrap();

    let mut engine2 = OrchestratorEngine::try_new(db2.inner_storage()).expect("engine try_new");
    engine2.try_register_tool(Box::new(IncrementTool)).unwrap();

    engine2
        .replay_from(&mut ctx2, "task_1")
        .await
        .expect("Replay failed");

    // Deep assertions post-recovery
    assert_eq!(ctx2.current_node, "task_1");
    assert_eq!(ctx2.step_count, 1);
    engine2
        .run(&mut ctx2, &graph)
        .await
        .expect("Resume run failed");

    assert_eq!(ctx2.status, AgentStatus::Completed);
    assert_eq!(ctx2.current_node, "end");
    assert_eq!(ctx2.memory.get("last_output").unwrap().as_u64().unwrap(), 1);

    // Audit trail verification on recovered DB
    let audit_log = contextra_agent::audit::AuditLog::new(state_collection2);
    let audit_entries = audit_log
        .replay_task("test_task_123")
        .await
        .expect("audit replay");
    assert!(!audit_entries.is_empty());
    for entry in &audit_entries {
        assert_eq!(entry.task_id, "test_task_123");
    }
}
