#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

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
async fn test_confirm_required_semantics_table_test() {
    let (server, _tmp) = create_mock_server().await;

    // Set deletion proof key environment variable for drop_collection tests requiring proof key
    std::env::set_var(
        "CONTEXTRA_DELETION_PROOF_KEY",
        "test_key_32_bytes_long_exact!!",
    );

    struct TestCase {
        name: &'static str,
        tool: &'static str,
        confirm_value: Option<Value>,
        should_succeed: bool,
    }

    let test_cases = vec![
        // ── contextra_forget ──
        TestCase {
            name: "forget with confirm: true (bool) -> success",
            tool: "contextra_forget",
            confirm_value: Some(json!(true)),
            should_succeed: true,
        },
        TestCase {
            name: "forget with confirm: false (bool) -> rejection",
            tool: "contextra_forget",
            confirm_value: Some(json!(false)),
            should_succeed: false,
        },
        TestCase {
            name: "forget with confirm: \"true\" (string) -> rejection",
            tool: "contextra_forget",
            confirm_value: Some(json!("true")),
            should_succeed: false,
        },
        TestCase {
            name: "forget with confirm: 1 (number) -> rejection",
            tool: "contextra_forget",
            confirm_value: Some(json!(1)),
            should_succeed: false,
        },
        TestCase {
            name: "forget with confirm: null -> rejection",
            tool: "contextra_forget",
            confirm_value: Some(Value::Null),
            should_succeed: false,
        },
        TestCase {
            name: "forget with confirm parameter missing -> rejection",
            tool: "contextra_forget",
            confirm_value: None,
            should_succeed: false,
        },
        // ── contextra_drop_collection ──
        TestCase {
            name: "drop_collection with confirm: true (bool) -> success",
            tool: "contextra_drop_collection",
            confirm_value: Some(json!(true)),
            should_succeed: true,
        },
        TestCase {
            name: "drop_collection with confirm: false (bool) -> rejection",
            tool: "contextra_drop_collection",
            confirm_value: Some(json!(false)),
            should_succeed: false,
        },
        TestCase {
            name: "drop_collection with confirm: \"true\" (string) -> rejection",
            tool: "contextra_drop_collection",
            confirm_value: Some(json!("true")),
            should_succeed: false,
        },
        TestCase {
            name: "drop_collection with confirm: 1 (number) -> rejection",
            tool: "contextra_drop_collection",
            confirm_value: Some(json!(1)),
            should_succeed: false,
        },
        TestCase {
            name: "drop_collection with confirm: null -> rejection",
            tool: "contextra_drop_collection",
            confirm_value: Some(Value::Null),
            should_succeed: false,
        },
        TestCase {
            name: "drop_collection with confirm parameter missing -> rejection",
            tool: "contextra_drop_collection",
            confirm_value: None,
            should_succeed: false,
        },
    ];

    for tc in test_cases {
        let mut args = json!({
            "collection": "test_col",
            "id": "doc1"
        });

        if let Some(val) = tc.confirm_value {
            args.as_object_mut()
                .unwrap()
                .insert("confirm".to_string(), val);
        }

        let req = contextra_mcp::protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "tools/call".into(),
            params: json!({
                "name": tc.tool,
                "arguments": args
            }),
        };

        let resp = server.handle(req).await;
        let res_val = serde_json::to_value(&resp).unwrap();

        if tc.should_succeed {
            assert_ne!(
                res_val["result"]["isError"], true,
                "Case '{}' expected success, got error: {:?}",
                tc.name, res_val
            );
        } else {
            assert_eq!(
                res_val["result"]["isError"], true,
                "Case '{}' expected rejection, got success: {:?}",
                tc.name, res_val
            );
            let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
            assert!(
                text.contains("confirm parameter must be explicitly set to true"),
                "Case '{}' expected clear confirmation error message, got: '{text}'",
                tc.name
            );
        }
    }

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}
