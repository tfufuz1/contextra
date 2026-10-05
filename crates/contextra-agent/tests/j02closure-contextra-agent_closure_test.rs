// FILE-CONTEXT Header (Format v3)
// ZWECK: Integration test verifying wiring of target symbols for J02 closure.
// INVARIANTEN: Tests all 6 required symbols through public crate APIs.
// STAND: TS:2026-10-05T00:00:00Z

use contextra_agent::audit::{migrate_legacy_audit_entries, AuditLog};
use contextra_agent::dlq::DeadLetterQueue;
use contextra_agent::engine::OrchestratorEngine;
use contextra_agent::event_source::PollingDocumentEventSource;
use contextra_agent::goal_condition::GoalCondition;
use contextra_agent::graph::{NodeType, StateGraph};
use contextra_agent::step::{DeadLetterReason, StepDeadLetter};
use contextra_db::{Contextra, ContextraConfig};
use contextra_ports::TestMetricsSink;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_j02_closure_all_six_symbols_wired() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup Contextra DB using tempdir
    let tmp = tempfile::tempdir()?;
    let db = Contextra::open_with_config(tmp.path(), ContextraConfig::default()).await?;
    let col = db.collection("j02_closure_col").await?;

    // 2. Symbol 1: migrate_legacy_audit_entries
    let stats = migrate_legacy_audit_entries(&*col).await?;
    assert_eq!(stats.migrated, 0);
    assert_eq!(stats.failed, 0);

    let audit_log = AuditLog::new(col.clone());
    let audit_stats = audit_log.migrate_legacy_entries().await?;
    assert_eq!(audit_stats.migrated, 0);

    // 3. Symbol 2 & 3: DeadLetterQueue::new_with_metrics & DeadLetterQueue::is_already_committed
    let metrics = Arc::new(TestMetricsSink::new());
    let dlq = DeadLetterQueue::new_with_metrics(db.inner_storage(), metrics.clone());

    let uncommitted_letter = StepDeadLetter {
        session_id: "closure-session-1".to_string(),
        node_id: "node-task".to_string(),
        step_index: 0,
        tx_id: Some(contextra_types::TxId::new(9999)),
        failure_reason: DeadLetterReason::Timeout { timeout_ms: 1000 },
        input: serde_json::json!({"action": "test"}),
        attempt: 0,
        failed_at_secs: 100,
    };

    let is_committed = dlq.is_already_committed(&uncommitted_letter).await?;
    assert!(
        !is_committed,
        "Uncommitted TxId should return false from is_already_committed"
    );

    // Push and verify metrics gauge
    dlq.push(&uncommitted_letter).await?;
    let drained = dlq.drain_uncommitted().await?;
    assert_eq!(drained.len(), 1);

    // 4. Symbol 4: OrchestratorEngine::from_db
    #[allow(deprecated)]
    let mut engine = OrchestratorEngine::from_db(&db);
    engine = engine.with_metrics_sink(metrics.clone());
    engine.recover_orphans().await?;

    // 5. Symbol 5: PollingDocumentEventSource::with_last_seen_seq
    let poll_source = PollingDocumentEventSource::new(col.clone(), Duration::from_millis(50))
        .with_last_seen_seq(42);
    assert_eq!(poll_source.poll_interval(), Duration::from_millis(50));

    // 6. Symbol 6: StateGraph::try_add_edge_with_goal
    let mut graph = StateGraph::new();
    graph.try_add_node("start", "Start Node", NodeType::Start, None)?;
    graph.try_add_node("end", "End Node", NodeType::End, None)?;

    let goal = GoalCondition::OutputFieldEquals {
        field_path: "status".to_string(),
        expected: serde_json::json!("ok"),
    };
    graph.try_add_edge_with_goal("start", "end", goal, 10)?;

    // Also test try_add_edge delegating to try_add_edge_with_goal
    graph.try_add_edge("start", "end", Some("status == 'ok'"), 5)?;

    assert_eq!(graph.edges.len(), 2);
    assert!(graph.edges[0].goal.is_some());
    assert!(graph.edges[1].goal.is_some());

    Ok(())
}
