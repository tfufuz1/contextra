use contextra_mcp::sandbox::{McpSandbox, SandboxPolicy};
use contextra_mcp::McpServer;
use serde_json::{json, Value};
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

async fn create_mock_server() -> (Arc<McpServer>, TempDir) {
    use contextra::Contextra;

    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let policy = SandboxPolicy {
        allow_db_reads: true,
        allow_db_writes: true,
        allow_code_execution: false,
        allow_cloud_egress: true,
        max_execution_ms: 5_000,
    };
    let sandbox = Arc::new(McpSandbox::new(policy).expect("sandbox new"));
    let server = Arc::new(McpServer::with_sandbox(
        Arc::new(db),
        Arc::new(MockEmbedder),
        sandbox,
    ));
    (server, tmp)
}

#[tokio::test]
async fn test_contextra_delete_no_unverified_proof() -> Result<(), Box<dyn std::error::Error>> {
    let (server, _tmp) = create_mock_server().await;

    // Ensure CONTEXTRA_DELETION_PROOF_KEY is unset initially
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");

    // 1. Insert a document into col1
    let insert_req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_insert",
            "arguments": {
                "collection": "col1",
                "id": "doc1",
                "text": "Hello world text content"
            }
        }),
    };
    let insert_resp = server.handle(insert_req).await;
    let insert_val = serde_json::to_value(&insert_resp)?;
    if insert_val["result"]["isError"] == true {
        panic!("insert failed: {:?}", insert_val);
    }

    // 2. Call contextra_delete without CONTEXTRA_DELETION_PROOF_KEY set.
    // It must succeed and return {"ok": true, "collection": "col1", "id": "doc1", "proof": null, "proof_scope": "collection_only"}.
    let delete_req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(2)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_delete",
            "arguments": {
                "collection": "col1",
                "id": "doc1"
            }
        }),
    };
    let delete_resp = server.handle(delete_req).await;
    let delete_val = serde_json::to_value(&delete_resp)?;
    assert_ne!(
        delete_val["result"]["isError"], true,
        "delete returned error: {:?}",
        delete_val
    );

    let content_text = delete_val["result"]["content"][0]["text"]
        .as_str()
        .ok_or("missing content text")?;
    let content_json: Value = serde_json::from_str(content_text)?;

    assert_eq!(content_json["ok"], true);
    assert_eq!(content_json["collection"], "col1");
    assert_eq!(content_json["id"], "doc1");
    assert_eq!(content_json["proof"], Value::Null);
    assert_eq!(content_json["proof_scope"], "collection_only");

    // 3. Test contextra_forget for single document
    // Insert doc2 into col1
    let insert_req2 = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(3)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_insert",
            "arguments": {
                "collection": "col1",
                "id": "doc2",
                "text": "Another document"
            }
        }),
    };
    let insert_resp2 = server.handle(insert_req2).await;
    let insert_val2 = serde_json::to_value(&insert_resp2)?;
    assert_ne!(insert_val2["result"]["isError"], true);

    let forget_req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(4)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_forget",
            "arguments": {
                "collection": "col1",
                "id": "doc2",
                "confirm": true
            }
        }),
    };
    let forget_resp = server.handle(forget_req).await;
    let forget_val = serde_json::to_value(&forget_resp)?;
    assert_ne!(forget_val["result"]["isError"], true);

    let forget_text = forget_val["result"]["content"][0]["text"]
        .as_str()
        .ok_or("missing forget content text")?;
    let forget_json: Value = serde_json::from_str(forget_text)?;

    assert_eq!(forget_json["ok"], true);
    assert_eq!(forget_json["collection"], "col1");
    assert_eq!(forget_json["id"], "doc2");
    assert_eq!(forget_json["proof"], Value::Null);
    assert_eq!(forget_json["proof_scope"], "collection_only");

    // 4. Verify tool descriptions in tools/list
    let list_req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(5)),
        method: "tools/list".into(),
        params: json!({}),
    };
    let list_resp = server.handle(list_req).await;
    let list_val = serde_json::to_value(&list_resp)?;
    let tools = list_val["result"]["tools"]
        .as_array()
        .ok_or("missing tools array")?;

    let delete_tool = tools
        .iter()
        .find(|t| t["name"] == "contextra_delete")
        .ok_or("missing contextra_delete tool")?;
    let forget_tool = tools
        .iter()
        .find(|t| t["name"] == "contextra_forget")
        .ok_or("missing contextra_forget tool")?;

    assert_eq!(
        delete_tool["description"],
        "Delete a single document (tombstone). No DeletionProof is issued for single documents; use contextra_drop_collection for a collection-scoped proof."
    );
    assert_eq!(
        forget_tool["description"],
        "Delete a single document (tombstone, no DeletionProof) or drop an entire collection (returns a collection-scoped DeletionProof; requires CONTEXTRA_DELETION_PROOF_KEY)."
    );

    // Also verify TOOL_REGISTRY descriptions in sandbox
    let registry = contextra_mcp::sandbox::TOOL_REGISTRY;
    let reg_delete = registry
        .iter()
        .find(|t| t.name == "contextra_delete")
        .ok_or("missing contextra_delete in registry")?;
    let reg_forget = registry
        .iter()
        .find(|t| t.name == "contextra_forget")
        .ok_or("missing contextra_forget in registry")?;

    assert_eq!(
        reg_delete.description,
        "Delete a single document (tombstone). No DeletionProof is issued for single documents; use contextra_drop_collection for a collection-scoped proof."
    );
    assert_eq!(
        reg_forget.description,
        "Delete a single document (tombstone, no DeletionProof) or drop an entire collection (returns a collection-scoped DeletionProof; requires CONTEXTRA_DELETION_PROOF_KEY)."
    );

    Ok(())
}
