// FILE-CONTEXT
// STAND: 2026-10-29T00:00:00Z
// ZWECK: Unit tests for ToolRegistry, ToolManifest validation, capability permission checks, and RetrieveTool execution.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::context::AgentContext;
use crate::step::{AgentTool, StepResult};
use contextra_db::{Contextra, ContextraConfig, DistanceMetric};
use contextra_ports::BoxFuture;
use contextra_types::{ContextraError, Result, TokenBudget};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

struct DummyTool {
    name: String,
}

impl AgentTool for DummyTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute<'a>(
        &'a self,
        _ctx: &'a AgentContext,
        _input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        Box::pin(async move {
            Ok(StepResult {
                node_id: "test".to_string(),
                output: json!({"ok": true}),
                tokens_consumed: 5,
                next_edge: None,
            })
        })
    }
}

async fn create_test_context() -> (AgentContext, TempDir) {
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
    let state_col = db.collection("test-col").await.expect("collection");
    let ctx = AgentContext::try_new(
        "task-tools-1",
        "start",
        db,
        state_col,
        TokenBudget::new(1000, 0),
    )
    .expect("ctx");
    (ctx, tmp)
}

#[test]
fn test_tool_manifest_validation() {
    let valid = ToolManifest::new("my_tool", "Description").with_timeout_ms(5000);
    assert!(valid.validate().is_ok());

    let empty_name = ToolManifest::new("", "Desc");
    assert!(matches!(
        empty_name.validate(),
        Err(ContextraError::InvalidInput(_))
    ));

    let zero_timeout = ToolManifest::new("t", "Desc").with_timeout_ms(0);
    assert!(matches!(
        zero_timeout.validate(),
        Err(ContextraError::InvalidInput(_))
    ));

    let null_name = ToolManifest::new("tool\0null", "Desc");
    assert!(matches!(
        null_name.validate(),
        Err(ContextraError::InvalidInput(_))
    ));
}

#[tokio::test]
async fn test_tool_registry_capability_fail_closed() {
    let (mut ctx, _tmp) = create_test_context().await;
    let mut registry = ToolRegistry::new();

    let manifest =
        ToolManifest::new("secure_tool", "A secure tool").with_capability("tool:secure_access");
    let tool = Box::new(DummyTool {
        name: "secure_tool".to_string(),
    });

    registry.register(manifest, tool).expect("register");

    // 1. Without capability -> PolicyViolation (fail-closed)
    let auth_res = registry.authorize(&ctx, "secure_tool");
    assert!(
        matches!(auth_res, Err(ContextraError::PolicyViolation(_))),
        "Must reject tool execution when required capability is missing"
    );

    // 2. Grant capability via memory
    ctx.memory
        .insert("capabilities".to_string(), json!(["tool:secure_access"]));
    let auth_ok = registry.authorize(&ctx, "secure_tool");
    assert!(
        auth_ok.is_ok(),
        "Must allow tool execution when capability is granted"
    );
}

#[tokio::test]
async fn test_retrieve_tool_execution() {
    let (ctx, _tmp) = create_test_context().await;
    let tool = RetrieveTool::new();

    assert_eq!(tool.name(), "retrieve");
    assert_eq!(tool.estimated_cost(&json!({})), 10);

    // Insert dummy document into state collection
    ctx.state_collection
        .put_kv("doc1", &json!({"text": "Contextra search term"}))
        .await
        .expect("put_kv");

    let input = json!({
        "query": "Contextra",
        "limit": 5
    });

    let res = tool.execute(&ctx, input).await.expect("execute");
    assert_eq!(res.node_id, "start");
    assert!(res.tokens_consumed >= 10);
    assert_eq!(res.output.get("count").and_then(|v| v.as_u64()), Some(1));
}
