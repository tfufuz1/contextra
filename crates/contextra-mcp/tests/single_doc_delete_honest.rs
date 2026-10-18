#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mcp::sandbox::{McpSandbox, SandboxPolicy};
use contextra_mcp::McpServer;
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
        Box::pin(async move { Ok(vec![0.1f32; 128]) })
    }

    fn embedding_dim(&self) -> usize {
        128
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> contextra_ports::BoxFuture<
        'a,
        std::result::Result<Vec<Vec<f32>>, contextra_ports::EmbeddingError>,
    > {
        Box::pin(async move { Ok(vec![vec![0.1f32; 128]; texts.len()]) })
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
async fn test_unconfirmed_drop_collection_returns_error() {
    let (server, _tmp) = create_mock_server().await;

    // Drop collection without confirm: true must return error
    let req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_drop_collection",
            "arguments": {
                "collection": "test_col",
                "confirm": false
            }
        }),
    };
    let resp = server.handle(req).await;
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_eq!(res_val["result"]["isError"], true);
    let err_msg = res_val["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        err_msg.contains("confirm parameter must be explicitly set to true"),
        "Unexpected error message: {err_msg}"
    );
}

#[tokio::test]
async fn test_single_doc_delete_does_not_contain_proof_field() {
    let (server, _tmp) = create_mock_server().await;

    // First create or insert document
    let col = server.db.collection("default").await.expect("collection");
    col.delete("doc_123").await.expect("delete");

    let req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_delete",
            "arguments": {
                "collection": "default",
                "id": "doc_123"
            }
        }),
    };
    let resp = server.handle(req).await;
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_ne!(res_val["result"]["isError"], true);
    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    let delete_res: serde_json::Value = serde_json::from_str(text).unwrap();

    assert_eq!(
        delete_res.get("ok").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        delete_res.get("id").and_then(|v| v.as_str()),
        Some("doc_123")
    );
    assert_eq!(
        delete_res.get("proof_status").and_then(|v| v.as_str()),
        Some("logical_only")
    );
}

#[test]
fn test_default_test_deletion_proof_key_not_in_src() {
    let src_path = if std::path::Path::new("src").exists() {
        std::path::PathBuf::from("src")
    } else {
        std::path::PathBuf::from("crates/contextra-mcp/src")
    };
    assert!(src_path.exists(), "src directory must exist");

    fn check_dir(dir: &std::path::Path) {
        for entry in std::fs::read_dir(dir).expect("read_dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                check_dir(&path);
            } else if path.is_file() {
                let content = std::fs::read_to_string(&path).expect("read file");
                assert!(
                    !content.contains("default_test_deletion_proof_key"),
                    "File {:?} contains forbidden string 'default_test_deletion_proof_key'!",
                    path
                );
            }
        }
    }

    check_dir(&src_path);
}
