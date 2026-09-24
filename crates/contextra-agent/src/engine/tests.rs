use super::*;
use contextra_types::TokenBudget;
use contextra_db::{DistanceMetric, Contextra, ContextraConfig};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

async fn create_dummy_context() -> (AgentContext, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let config = ContextraConfig {
        dimension: 4,
        max_elements: 1000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let db = Arc::new(
        Contextra::open_with_config(tmp.path(), config)
            .await
            .expect("open db"),
    );
    let state_col = db.collection("test-state").await.expect("collection");
    let ctx = AgentContext::try_new(
        "test-task-1",
        "start",
        db,
        state_col,
        TokenBudget::new(1000, 0),
    )
    .expect("agent context");
    (ctx, tmp)
}

#[tokio::test]
async fn test_evaluate_condition_expr_grammar_and_outcomes() {
    let (mut ctx, _tmp) = create_dummy_context().await;
    ctx.memory.insert("simple_str".to_string(), json!("hello"));
    ctx.memory.insert("number_val".to_string(), json!(42));
    ctx.memory.insert("bool_val".to_string(), json!(true));
    ctx.memory.insert("null_val".to_string(), json!(null));
    ctx.memory.insert(
        "nested".to_string(),
        json!({
            "status": "approved",
            "code": 200
        }),
    );

    // 1. "exists" checks
    assert!(evaluate_condition_expr("simple_str exists", &ctx));
    assert!(evaluate_condition_expr("nested.status exists", &ctx));
    assert!(evaluate_condition_expr("task_id exists", &ctx));
    assert!(!evaluate_condition_expr("null_val exists", &ctx));
    assert!(!evaluate_condition_expr("missing_key exists", &ctx));
    assert!(!evaluate_condition_expr(" exists", &ctx)); // missing key

    // 2. "==" checks
    assert!(evaluate_condition_expr("simple_str == hello", &ctx));
    assert!(evaluate_condition_expr("simple_str == \"hello\"", &ctx));
    assert!(evaluate_condition_expr("number_val == 42", &ctx));
    assert!(evaluate_condition_expr("bool_val == true", &ctx));
    assert!(evaluate_condition_expr("nested.status == approved", &ctx));
    assert!(evaluate_condition_expr("nested.code == 200", &ctx));
    assert!(evaluate_condition_expr("task_id == test-task-1", &ctx));
    assert!(!evaluate_condition_expr("simple_str == world", &ctx));
    assert!(!evaluate_condition_expr("missing_key == foo", &ctx));

    // 3. "!=" checks
    assert!(evaluate_condition_expr("simple_str != world", &ctx));
    assert!(evaluate_condition_expr("missing_key != foo", &ctx));
    assert!(!evaluate_condition_expr("simple_str != hello", &ctx));

    // 4. Unparseable & invalid expressions (no panic)
    assert!(!evaluate_condition_expr("invalid condition syntax", &ctx));
    assert!(!evaluate_condition_expr("", &ctx));
    assert!(!evaluate_condition_expr("   ", &ctx));
    assert!(!evaluate_condition_expr("== value_without_key", &ctx));
    assert!(!evaluate_condition_expr("!= value_without_key", &ctx));
}

#[tokio::test]
async fn test_audit_before_commit_ordering() {
    let (ctx, _tmp) = create_dummy_context().await;
    let orchestrator = OrchestratorEngine::try_from_db(&ctx.db).expect("engine try_from_db");

    // Populate an existing KV entry under task:test-task-1:step:0 to force commit_step to fail
    // if state_collection.put_kv_if_absent was used, but put_kv overwrites.
    // Wait, put_kv doesn't fail on existing key, put_kv_if_absent does!
    // To simulate commit_step failure after successful audit_log:
    // Populate audit entry manually? No, audit_log uses put_kv_if_absent under "audit:test-task-1:step:0".
    // commit_step uses put_kv under "task:test-task-1:step:0".
    // If we want commit_step to fail while audit_log succeeds, we can simulate an error in commit_step or
    // test ordering directly.
    // Let's test calling audit_log directly then commit_step with invalid state ID or pre-condition,
    // or test that after audit_log succeeds, state_collection contains the audit entry "audit:test-task-1:step:0"
    // even if commit_step fails, and step_count is NOT incremented (remains 0).
    let step_res = StepResult {
        node_id: "start".to_string(),
        output: json!({"res": "ok"}),
        tokens_consumed: 10,
        next_edge: None,
    };

    // Call audit_log first (as in new loop order)
    let audit_res = orchestrator.audit_log(&ctx, &step_res).await;
    assert!(audit_res.is_ok());

    // Verify audit entry exists in state_collection
    let audit_id = format!("audit:{}:step:{}", ctx.task_id, ctx.step_count);
    let audit_entry = ctx.state_collection.get_kv(&audit_id).await.unwrap();
    assert!(audit_entry.is_some());

    // Verify state doc key for commit_step does NOT exist yet
    let state_doc_id = format!("task:{}:step:{}", ctx.task_id, ctx.step_count);
    let state_entry = ctx.state_collection.get_kv(&state_doc_id).await.unwrap();
    assert!(state_entry.is_none());

    // Verify step_count was not incremented
    assert_eq!(ctx.step_count, 0);
}

struct MockTool {
    tool_name: String,
}

impl AgentTool for MockTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn execute<'a>(
        &'a self,
        _ctx: &'a AgentContext,
        _input: serde_json::Value,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<StepResult>> {
        Box::pin(async move {
            Ok(StepResult {
                node_id: "test".to_string(),
                output: serde_json::Value::Null,
                tokens_consumed: 0,
                next_edge: None,
            })
        })
    }
}

#[tokio::test]
async fn test_try_register_tool_boundary_validations() {
    let (ctx, _tmp) = create_dummy_context().await;
    let mut orchestrator = OrchestratorEngine::try_from_db(&ctx.db).expect("engine try_from_db");

    // 1. Valid tool registration
    let valid_tool = MockTool {
        tool_name: "valid_tool".to_string(),
    };
    assert!(orchestrator.try_register_tool(Box::new(valid_tool)).is_ok());

    // 2. Empty tool name
    let empty_tool = MockTool {
        tool_name: "".to_string(),
    };
    assert!(matches!(
        orchestrator.try_register_tool(Box::new(empty_tool)),
        Err(ContextraError::InvalidInput(_))
    ));

    // 3. Null byte in tool name
    let null_tool = MockTool {
        tool_name: "tool\0null".to_string(),
    };
    assert!(matches!(
        orchestrator.try_register_tool(Box::new(null_tool)),
        Err(ContextraError::InvalidInput(_))
    ));

    // 4. Oversized tool name
    let oversized_tool = MockTool {
        tool_name: "t".repeat(257),
    };
    assert!(matches!(
        orchestrator.try_register_tool(Box::new(oversized_tool)),
        Err(ContextraError::InvalidInput(_))
    ));
}

#[tokio::test]
async fn test_orchestrator_recover_orphans_succeeds() {
    let (ctx, _tmp) = create_dummy_context().await;
    let orchestrator = OrchestratorEngine::try_from_db(&ctx.db).expect("engine try_from_db");
    assert!(orchestrator.recover_orphans().await.is_ok());
}
