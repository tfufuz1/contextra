#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::Contextra;
use contextra_mcp::egress_gateway::DefaultEgressClassifier;
use contextra_mcp::io::read_line_bounded;
use contextra_mcp::prompt_injection::PromptInjectionGuard;
use contextra_mcp::protocol::JsonRpcResponse;
use contextra_mcp::sandbox::{McpSandbox, SandboxPolicy};
use contextra_mcp::McpServer;
use contextra_ports::BoxFuture;
use contextra_types::TenantId;
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Debug)]
struct MockEmbedder {
    dim: usize,
}

impl MockEmbedder {
    fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl contextra_ports::EmbeddingProvider for MockEmbedder {
    fn provider_name(&self) -> &str {
        "mock"
    }

    fn embed<'a>(
        &'a self,
        _text: &'a str,
    ) -> BoxFuture<'a, std::result::Result<Vec<f32>, contextra_ports::EmbeddingError>> {
        let dim = self.dim;
        Box::pin(async move { Ok(vec![0.1f32; dim]) })
    }

    fn embedding_dim(&self) -> usize {
        self.dim
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, std::result::Result<Vec<Vec<f32>>, contextra_ports::EmbeddingError>> {
        let dim = self.dim;
        let count = texts.len();
        Box::pin(async move { Ok(vec![vec![0.1f32; dim]; count]) })
    }
}

async fn create_test_server() -> (Arc<McpServer>, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let col = db.collection("default").await.expect("collection");
    let dim = col.dimension();

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
        Arc::new(MockEmbedder::new(dim)),
        sandbox,
    ));
    (server, tmp)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_server_constructors_and_builder_options() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Arc::new(Contextra::open(tmp.path()).await.expect("open db"));
    let embedder = Arc::new(MockEmbedder::new(128));

    // 1. McpServer::new default constructor
    let s1 = McpServer::new(db.clone(), embedder.clone()).expect("McpServer::new");
    assert!(s1.tenant_id.is_some());

    // 2. McpServer::with_write_permission
    let s2 = McpServer::with_write_permission(db.clone(), embedder.clone(), false)
        .expect("with_write_permission false");
    assert!(!s2.sandbox.policy().allow_db_writes);

    // 3. Builder chaining
    let custom_tenant = TenantId::try_new(42).ok();
    let injection_guard = Arc::new(PromptInjectionGuard::from_env());
    let classifier = Arc::new(DefaultEgressClassifier::default());

    let server_custom = s1
        .with_tenant_id(custom_tenant)
        .with_injection_guard(injection_guard)
        .with_egress_classifier(classifier)
        .with_routing(None)
        .with_plugin_registry(None);

    assert_eq!(server_custom.tenant_id, custom_tenant);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_server_stdio_processing_loop_simulation() {
    let (server, _tmp) = create_test_server().await;

    // Simulate lines being passed to server.handle_value
    let line1 = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "ping"
    });
    let line2 = json!({
        "jsonrpc": "2.0",
        "method": "initialized" // notification -> None
    });
    let line3 = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "initialize"
    });

    let resp1 = server.handle_value(line1).await.expect("response for ping");
    assert_eq!(resp1["id"], 1);

    let resp2 = server.handle_value(line2).await;
    assert!(resp2.is_none());

    let resp3 = server
        .handle_value(line3)
        .await
        .expect("response for initialize");
    assert_eq!(resp3["id"], 2);
    assert_eq!(resp3["result"]["protocolVersion"], "2024-11-05");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_server_parse_error_simulation() {
    // Malformed JSON string passed to error formatter like in run_stdio
    let malformed = "not valid json {";
    let parse_res = serde_json::from_str::<Value>(malformed);
    assert!(parse_res.is_err());
    let e = parse_res.unwrap_err();

    let err_resp = JsonRpcResponse::err(None, -32700, format!("Parse error: {e}"));
    assert_eq!(err_resp.error.as_ref().unwrap().code, -32700);
    assert!(err_resp
        .error
        .as_ref()
        .unwrap()
        .message
        .contains("Parse error"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_server_read_line_bounded_eof_and_empty_lines() {
    // 1. Immediate EOF
    let mut empty_cursor = std::io::Cursor::new(Vec::new());
    let mut buf = String::new();
    let n = read_line_bounded(&mut empty_cursor, &mut buf, 1024)
        .await
        .expect("read EOF");
    assert_eq!(n, 0);

    // 2. Empty newline
    let mut newline_cursor = std::io::Cursor::new(b"\n\n".to_vec());
    let n1 = read_line_bounded(&mut newline_cursor, &mut buf, 1024)
        .await
        .expect("read newline 1");
    assert_eq!(n1, 1);
    assert_eq!(buf.trim(), "");

    let n2 = read_line_bounded(&mut newline_cursor, &mut buf, 1024)
        .await
        .expect("read newline 2");
    assert_eq!(n2, 1);
    assert_eq!(buf.trim(), "");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_server_idempotency_multiple_calls() {
    let (server, _tmp) = create_test_server().await;

    // Multiple initialize calls should be idempotent
    for i in 1..=3 {
        let req = json!({
            "jsonrpc": "2.0",
            "id": i,
            "method": "initialize"
        });
        let resp = server.handle_value(req).await.expect("response");
        assert_eq!(resp["id"], i);
        assert_eq!(resp["result"]["protocolVersion"], "2024-11-05");
    }
}
