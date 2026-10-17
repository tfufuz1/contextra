#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mcp::proof_key_env::resolve_proof_key_from_env;
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
async fn test_absence_or_invalid_proof_key_returns_error() {
    // Clear key env vars
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
    std::env::remove_var("CONTEXTRA_PROOF_KEY");

    let res = resolve_proof_key_from_env();
    assert!(
        res.key.is_none(),
        "Key must be None when no key env vars are set"
    );

    let (server, _tmp) = create_mock_server().await;

    // Drop collection requires key and must return error when missing
    let drop_res = server
        .handle_drop_collection(&json!({
            "collection": "test_col",
            "confirm": true
        }))
        .await;

    assert!(
        drop_res.is_err(),
        "drop_collection must fail closed when key is missing"
    );
    let err_msg = drop_res.unwrap_err().message;
    assert!(
        err_msg.contains("deletion proof key not configured"),
        "Unexpected error message: {err_msg}"
    );
}

#[tokio::test]
async fn test_single_doc_delete_does_not_contain_proof_field() {
    let (server, _tmp) = create_mock_server().await;

    // First create or insert document
    let col = server.db().collection("default").await.expect("collection");
    col.delete("doc_123").await.expect("delete");

    let delete_res = server
        .handle_delete(&json!({
            "collection": "default",
            "id": "doc_123"
        }))
        .await
        .expect("handle_delete success");

    assert_eq!(
        delete_res.get("ok").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        delete_res.get("id").and_then(|v| v.as_str()),
        Some("doc_123")
    );
    assert!(
        delete_res.get("proof").is_none(),
        "Single document delete output MUST NOT contain proof field!"
    );
    assert_eq!(
        delete_res.get("proof_status").and_then(|v| v.as_str()),
        Some("logical_only")
    );
}

#[test]
fn test_default_test_deletion_proof_key_not_in_src() {
    let src_path = std::path::Path::new("crates/contextra-mcp/src");
    assert!(src_path.exists(), "crates/contextra-mcp/src must exist");

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

    check_dir(src_path);
}
