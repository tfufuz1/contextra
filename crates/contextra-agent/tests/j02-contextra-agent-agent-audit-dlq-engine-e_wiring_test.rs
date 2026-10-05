use contextra_agent::audit::AuditEntry;
use contextra_agent::context::AgentContext;
use contextra_agent::dlq::DeadLetterQueue;
use contextra_agent::event_source::{EventSource, PollingDocumentEventSource};
use contextra_agent::goal_condition::GoalCondition;
use contextra_agent::graph::{NodeType, StateGraph};
use contextra_agent::step::{DeadLetterReason, StepDeadLetter};
use contextra_agent::OrchestratorEngine;
use contextra_db::{Contextra, ContextraConfig};
use contextra_ports::{DocId, StorageEngine, TestMetricsSink, VectorIndex};
use contextra_types::{Result, TokenBudget, TxId};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_wiring_audit_migration_and_metrics_and_dlq_drain() -> Result<()> {
    let tmp = TempDir::new().expect("temp dir");
    let db = Arc::new(Contextra::open_with_config(tmp.path(), ContextraConfig::default()).await?);
    let col = db.collection("agent").await?;

    // 1. Inject legacy audit entry with embedding to verify migration during engine.run()
    let zero_vec = vec![0.0f32; 768];
    let key = "audit:task-wire:step:999";
    let doc_id = DocId::from_key(key)?;
    let tx = col.allocate_tx()?;

    let legacy_entry = AuditEntry {
        task_id: "task-wire".to_string(),
        step_count: 999,
        node_id: "legacy_step".to_string(),
        tokens_consumed: 5,
        payload: serde_json::json!({"action": "legacy"}),
        error: None,
        tx_id: None,
    };

    let stored = serde_json::json!({
        "id": key,
        "embedding": zero_vec,
        "metadata": serde_json::to_value(&legacy_entry)?
    });
    let meta_only = serde_json::json!({
        "id": key,
        "metadata": serde_json::to_value(&legacy_entry)?
    });

    let user_key = col.namespaced_key(key.as_bytes(), 0);
    let doc_key = col.namespaced_key(&doc_id.inner().to_le_bytes(), 1);

    col.storage()
        .put(tx, &user_key, &serde_json::to_vec(&stored)?)
        .await?;
    col.storage()
        .put(tx, &doc_key, &serde_json::to_vec(&meta_only)?)
        .await?;
    col.vector_index().insert(tx, doc_id, &zero_vec).await?;
    col.storage().commit(tx).await?;
    col.vector_index().commit(tx).await?;

    assert_eq!(col.len().await, 1, "Legacy zero vector present in index");

    // 2. Set up StateGraph with GoalCondition edge string to verify try_add_edge_with_goal wiring
    let mut graph = StateGraph::new();
    graph.try_add_node("start", "Start Node", NodeType::Start, None)?;
    graph.try_add_node("end", "End Node", NodeType::End, None)?;
    graph.try_add_edge("start", "end", Some("field == value"), 1)?;

    assert_eq!(graph.edges.len(), 1);
    assert_eq!(
        graph.edges[0].goal,
        Some(GoalCondition::OutputFieldEquals {
            field_path: "field".to_string(),
            expected: serde_json::json!("value")
        })
    );

    // 3. Set up OrchestratorEngine with metrics sink
    let metrics = Arc::new(TestMetricsSink::new());
    let engine = OrchestratorEngine::try_from_db(&db)?.with_metrics_sink(metrics.clone());

    // Also construct a standalone DLQ via DeadLetterQueue::new_with_metrics to verify the constructor
    let standalone_dlq = DeadLetterQueue::new_with_metrics(db.inner_storage(), metrics.clone());

    let mut ctx = AgentContext::try_new(
        "task-wire".to_string(),
        "start".to_string(),
        db.clone(),
        col.clone(),
        TokenBudget::new(1_000_000, 0),
    )?;

    // 4. Run workflow: triggers legacy audit migration & commits step 0
    engine.run(&mut ctx, &graph).await?;

    assert_eq!(
        col.len().await,
        0,
        "HNSW index count reduced to 0 after legacy audit migration in run()"
    );

    // Get actual committed tx_id from step 0 in state_collection
    let step0_val = col.get_kv("task:task-wire:step:0").await?.expect("step 0 committed");
    let committed_tx_id = step0_val.get("tx_id").and_then(|v| v.as_u64()).expect("tx_id exists");

    // 5. Set up DLQ entries (1 uncommitted, 1 already committed in storage)
    let dlq = engine.dead_letter_queue.as_ref().expect("DLQ exists");

    let committed_letter = StepDeadLetter {
        session_id: "task-wire".to_string(),
        node_id: "start".to_string(),
        step_index: 0,
        tx_id: Some(TxId::new(committed_tx_id)),
        failure_reason: DeadLetterReason::Timeout { timeout_ms: 1000 },
        input: serde_json::json!({}),
        attempt: 1,
        failed_at_secs: 100,
    };

    let uncommitted_letter = StepDeadLetter {
        session_id: "task-wire".to_string(),
        node_id: "start".to_string(),
        step_index: 888,
        tx_id: Some(TxId::new(99999)),
        failure_reason: DeadLetterReason::Timeout { timeout_ms: 1000 },
        input: serde_json::json!({}),
        attempt: 1,
        failed_at_secs: 100,
    };

    dlq.push(&committed_letter).await?;
    dlq.push(&uncommitted_letter).await?;

    // Push uncommitted letter to standalone DLQ as well
    standalone_dlq.push(&uncommitted_letter).await?;
    assert_eq!(standalone_dlq.list().await?.len(), 2); // committed_letter from storage scan + uncommitted_letter

    // 6. Test drain_uncommitted()
    let uncommitted = dlq.drain_uncommitted().await?;
    assert_eq!(
        uncommitted.len(),
        1,
        "Only uncommitted dead letter should remain after drain_uncommitted()"
    );
    assert_eq!(uncommitted[0].step_index, 888);

    // 7. Verify PollingDocumentEventSource::new_from_seq
    let mut poll_src = PollingDocumentEventSource::new_from_seq(col.clone(), Duration::from_millis(10), 100);
    assert_eq!(poll_src.next_event().await?, None);

    Ok(())
}
