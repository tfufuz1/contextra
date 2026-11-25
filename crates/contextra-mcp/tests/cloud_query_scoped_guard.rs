#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::Contextra;
use contextra_mcp::{
    protocol::JsonRpcRequest,
    sandbox::{McpSandbox, SandboxPolicy},
    server::McpServer,
};
use contextra_ports::BoxFuture;
use contextra_privacy::egress_gateway::EgressGuardCheck;
use contextra_privacy::egress_vault::{BlockReason, EgressClassification};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Debug)]
struct MockEmbedder {
    dimension: usize,
}

impl contextra_ports::EmbeddingProvider for MockEmbedder {
    fn provider_name(&self) -> &str {
        "mock"
    }

    fn embed<'a>(
        &'a self,
        _text: &'a str,
    ) -> BoxFuture<'a, std::result::Result<Vec<f32>, contextra_ports::EmbeddingError>> {
        Box::pin(async move { Ok(vec![0.1f32; self.dimension]) })
    }

    fn embedding_dim(&self) -> usize {
        self.dimension
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, std::result::Result<Vec<Vec<f32>>, contextra_ports::EmbeddingError>> {
        Box::pin(async move { Ok(vec![vec![0.1f32; self.dimension]; texts.len()]) })
    }
}

struct TestGuard {
    block: bool,
    reason_msg: String,
}

impl EgressGuardCheck for TestGuard {
    fn check<'a>(&'a self, _payload: &'a str) -> BoxFuture<'a, EgressClassification> {
        let block = self.block;
        let msg = self.reason_msg.clone();
        Box::pin(async move {
            if block {
                EgressClassification::Block(BlockReason::PolicyDenied(msg))
            } else {
                EgressClassification::Allow
            }
        })
    }
}

async fn setup_app() -> (McpServer, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let collection = db.collection("my_docs").await.expect("collection");
    let dim = collection.dimension();
    let embedder = Arc::new(MockEmbedder { dimension: dim });

    let policy = SandboxPolicy {
        allow_db_reads: true,
        allow_db_writes: true,
        allow_code_execution: false,
        allow_cloud_egress: true,
        max_execution_ms: 5_000,
    };
    let sandbox = Arc::new(McpSandbox::new(policy).expect("sandbox new"));
    let server = McpServer::with_sandbox(Arc::new(db), embedder, sandbox);
    (server, tmp)
}

#[tokio::test]
async fn test_cloud_query_guard_blocking() {
    let (server, _tmp) = setup_app().await;

    let blocking_guard = Arc::new(TestGuard {
        block: true,
        reason_msg: "Guard blocked egress payload".to_string(),
    });

    let server_with_guard = server.with_egress_guard(blocking_guard);

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_cloud_query",
            "arguments": {
                "query": "Safe looking search text"
            }
        }),
    };

    let response = server_with_guard.handle(req).await;
    let res_val = serde_json::to_value(&response).unwrap();
    assert_eq!(res_val["result"]["isError"], true);
    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Guard blocked egress payload"));
}

#[tokio::test]
async fn test_cloud_query_guard_allowing() {
    let (server, _tmp) = setup_app().await;

    let allowing_guard = Arc::new(TestGuard {
        block: false,
        reason_msg: String::new(),
    });

    let server_with_guard = server.with_egress_guard(allowing_guard);

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_cloud_query",
            "arguments": {
                "query": "Harmless cloud query text"
            }
        }),
    };

    let response = server_with_guard.handle(req).await;
    let res_val = serde_json::to_value(&response).unwrap();
    assert!(res_val["result"]["isError"].is_null());
    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    let query_res: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(query_res["status"], "success");
    assert_eq!(query_res["query"], "Harmless cloud query text");
}

#[tokio::test]
async fn test_cloud_query_no_guard_classifier_still_enforced() {
    let (server, _tmp) = setup_app().await;

    // Default server has no egress guard set (None).
    // Test that egress classifier fail-closed checks still block sensitive pattern (e.g. email address / PII)
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_cloud_query",
            "arguments": {
                "query": "Send email to user@example.com with details"
            }
        }),
    };

    let response = server.handle(req).await;
    let res_val = serde_json::to_value(&response).unwrap();
    assert_eq!(res_val["result"]["isError"], true);
    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Egress policy violation"));
}

#[tokio::test]
async fn test_unknown_method_returns_method_not_found_code() {
    let (server, _tmp) = setup_app().await;

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(4)),
        method: "unknown_custom_rpc_method".to_string(),
        params: json!({}),
    };

    let response = server.handle(req).await;
    assert!(response.error.is_some());
    let err = response.error.unwrap();
    assert_eq!(err.code, -32601);
    assert!(err.message.contains("Method not found"));
}
