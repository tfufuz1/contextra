#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::Contextra;
use contextra_mcp::sandbox::{McpSandbox, SandboxPolicy};
use contextra_mcp::McpServer;
use contextra_ports::BoxFuture;
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

async fn create_mock_server_with_write(allow_db_writes: bool) -> (Arc<McpServer>, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let collection = db.collection("default").await.expect("collection");
    let dim = collection.dimension();
    let embedder = Arc::new(MockEmbedder { dimension: dim });
    let policy = SandboxPolicy {
        allow_db_reads: true,
        allow_db_writes,
        allow_code_execution: false,
        allow_cloud_egress: false,
        max_execution_ms: 5_000,
    };
    let sandbox = Arc::new(McpSandbox::new(policy).expect("sandbox new"));
    let server = Arc::new(McpServer::with_sandbox(Arc::new(db), embedder, sandbox));
    (server, tmp)
}

#[tokio::test]
async fn test_forget_collection_removes_data_and_returns_proof() {
    let (server, _tmp) = create_mock_server_with_write(true).await;

    // Set deletion proof key environment variable
    std::env::set_var(
        "CONTEXTRA_DELETION_PROOF_KEY",
        "primary_deletion_proof_key_32bytes_valid!",
    );

    // 1. Insert documents into custom collection
    let col_name = "forget_test_col";
    let insert_req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_insert",
            "arguments": {
                "id": "doc1",
                "text": "Data in custom collection to forget",
                "collection": col_name
            }
        }),
    };
    let resp = server.handle(insert_req).await;
    assert!(resp.error.is_none());

    // 2. Forget collection with confirm=true
    let forget_req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(2)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_forget",
            "arguments": {
                "collection": col_name,
                "confirm": true
            }
        }),
    };
    let resp_forget = server.handle(forget_req).await;
    assert!(resp_forget.error.is_none());

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}
