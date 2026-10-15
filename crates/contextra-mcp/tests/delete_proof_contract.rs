// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Vertragsprüfungen für contextra_delete und contextra_drop_collection Beweisstatus (Auftrag A04 / F4)
// INVARIANTEN: Zero-Panic in Mocks, Fail-Closed, kein "proof": null ohne Status, INV-MCP-CLASSIFY-1 beibehalten.

use contextra::Contextra;
use contextra_mcp::sandbox::{McpSandbox, ToolCategory};
use contextra_mcp::server::McpServer;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Debug)]
struct MockEmbedder;

impl contextra_ports::EmbeddingProvider for MockEmbedder {
    fn provider_name(&self) -> &str {
        "mock"
    }

    fn embed<'a>(
        &'a self,
        _text: &'a str,
    ) -> contextra_ports::BoxFuture<
        'a,
        std::result::Result<Vec<f32>, contextra_ports::EmbeddingError>,
    > {
        Box::pin(async move { Ok(vec![0.1f32; 768]) })
    }

    fn embedding_dim(&self) -> usize {
        768
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> contextra_ports::BoxFuture<
        'a,
        std::result::Result<Vec<Vec<f32>>, contextra_ports::EmbeddingError>,
    > {
        Box::pin(async move { Ok(vec![vec![0.1f32; 768]; texts.len()]) })
    }
}

async fn create_test_server() -> (Arc<McpServer>, TempDir) {
    let tmp = TempDir::new().expect("temp dir creation");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let server = McpServer::with_write_permission(Arc::new(db), Arc::new(MockEmbedder), true)
        .expect("mcp server initialization");
    (Arc::new(server), tmp)
}

fn make_request(
    method: &str,
    params: serde_json::Value,
) -> contextra_mcp::protocol::JsonRpcRequest {
    contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        method: method.into(),
        params,
    }
}

#[tokio::test]
async fn test_delete_returns_proof_status_and_no_null_proof() {
    let (server, _tmp) = create_test_server().await;

    let req = make_request(
        "tools/call",
        json!({
            "name": "contextra_delete",
            "arguments": {
                "collection": "default",
                "id": "doc_123"
            }
        }),
    );

    let resp = server.handle(req).await;
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_ne!(res_val["result"]["isError"], true);
    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    let res: serde_json::Value = serde_json::from_str(text).unwrap();

    // Invariante F4: Kein proof: null ohne Statusfeld!
    assert!(
        res.get("proof").is_none(),
        "Response MUST NOT contain explicit 'proof': null without status"
    );
    assert_eq!(
        res.get("proof_status").and_then(|v| v.as_str()),
        Some("logical_only"),
        "proof_status MUST be 'logical_only' for contextra_delete"
    );
    assert_eq!(
        res.get("proof_scope").and_then(|v| v.as_str()),
        Some("collection_only"),
        "proof_scope MUST be 'collection_only' for contextra_delete"
    );
    assert_eq!(res.get("ok").and_then(|v| v.as_bool()), Some(true));
}

#[tokio::test]
async fn test_delete_engine_error_propagation() {
    let (server, _tmp) = create_test_server().await;

    let req = make_request(
        "tools/call",
        json!({
            "name": "contextra_delete",
            "arguments": {
                "collection": "default",
                "id": ""
            }
        }),
    );

    let resp = server.handle(req).await;
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_eq!(res_val["result"]["isError"], true);

    let err_msg = res_val["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        err_msg.contains("cannot be empty"),
        "Error message should reflect invalid parameter: {err_msg}"
    );
}

#[tokio::test]
async fn test_mcp_classify_tool_category_maintained() {
    // INV-MCP-CLASSIFY-1
    let cat_delete = McpSandbox::try_classify_method("contextra_delete").unwrap();
    assert_eq!(
        cat_delete,
        ToolCategory::DatabaseWrite,
        "contextra_delete category MUST remain DatabaseWrite"
    );

    let cat_drop = McpSandbox::try_classify_method("contextra_drop_collection").unwrap();
    assert_eq!(
        cat_drop,
        ToolCategory::DatabaseWrite,
        "contextra_drop_collection category MUST remain DatabaseWrite"
    );
}
