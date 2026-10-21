use contextra::Contextra;
use contextra_mcp::{protocol::JsonRpcRequest, McpServer};
use contextra_ports::BoxFuture;
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
    ) -> BoxFuture<'a, std::result::Result<Vec<f32>, contextra_ports::EmbeddingError>> {
        Box::pin(async move { Ok(vec![0.1f32; 128]) })
    }

    fn embedding_dim(&self) -> usize {
        128
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, std::result::Result<Vec<Vec<f32>>, contextra_ports::EmbeddingError>> {
        Box::pin(async move { Ok(vec![vec![0.1f32; 128]; texts.len()]) })
    }
}

#[tokio::test]
async fn ep1_build_has_no_cloud_tool() {
    let tmp = TempDir::new().unwrap();
    let db = Contextra::open(tmp.path()).await.unwrap();
    let embedder = Arc::new(MockEmbedder);
    let server = McpServer::with_write_permission(Arc::new(db), embedder, true).unwrap();

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/list".to_string(),
        params: json!({}),
    };

    let resp = server.handle(req).await;
    assert!(resp.error.is_none(), "tools/list should succeed");

    let result = resp.result.expect("result present");
    let tools = result.get("tools").and_then(|v| v.as_array()).expect("tools array");

    let has_cloud_tool = tools.iter().any(|t| t.get("name").and_then(|n| n.as_str()) == Some("contextra_cloud_query"));
    assert!(
        !has_cloud_tool,
        "ep1 build without feature 'guard' MUST NOT register contextra_cloud_query in tools/list"
    );

    // Calling contextra_cloud_query directly must also be rejected
    let call_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_cloud_query",
            "arguments": { "query": "test" }
        }),
    };

    let call_resp = server.handle(call_req).await;
    let is_err = call_resp.error.is_some()
        || call_resp.result.as_ref().and_then(|r| r.get("isError")).and_then(|e| e.as_bool()).unwrap_or(false);
    assert!(
        is_err,
        "Calling contextra_cloud_query on ep1 build without feature 'guard' MUST return error"
    );
}
