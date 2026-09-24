use contextra_agent::context::{AgentContext, AgentStatus};
use contextra_agent::engine::OrchestratorEngine;
use contextra_agent::graph::{NodeType, StateGraph};
use contextra_types::TokenBudget;
use contextra_db::Contextra;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_atomic_final_state_checkpoint() -> contextra_types::Result<()> {
    let tmp = TempDir::new().unwrap();
    let config = contextra_db::ContextraConfig {
        dimension: 1,
        ..Default::default()
    };
    let db = Arc::new(Contextra::open_with_config(tmp.path(), config).await?);

    // Create or get a state collection
    let collection = db.collection("state").await?;

    let storage = db.inner_storage();
    let engine = OrchestratorEngine::try_new(storage).expect("engine try_new");

    let mut graph = StateGraph::new();
    graph
        .try_add_node("start", "Start", NodeType::Start, None)
        .unwrap();
    graph
        .try_add_node("end", "End", NodeType::End, None)
        .unwrap();
    graph.try_add_edge("start", "end", None, 1).unwrap();

    let mut ctx = AgentContext::try_new(
        "test-task",
        "start",
        db.clone(),
        collection.clone(),
        TokenBudget::new(100, 0),
    )?;

    engine.run(&mut ctx, &graph).await?;

    assert_eq!(ctx.status, AgentStatus::Completed);

    // Verify that a checkpoint exists for the 'end' node
    let checkpoints = engine.checkpoint_store.list_checkpoints().await?;
    let end_checkpoint = checkpoints.iter().find(|c| c.name.contains(":node:end"));

    assert!(
        end_checkpoint.is_some(),
        "Checkpoint at 'end' node must exist"
    );

    // Verify final state is persisted
    let final_state = collection.get("task:test-task:final").await?;
    assert!(final_state.is_some(), "Final state must be persisted");

    Ok(())
}
