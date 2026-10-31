use contextra::Contextra;
use contextra_mcp::{
    egress_gateway::DefaultEgressClassifier,
    prompt_injection::PromptInjectionGuard,
    sandbox::{McpSandbox, SandboxPolicy},
    McpServer,
};
use contextra_ports::BoxFuture;
use contextra_types::TenantId;
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

#[tokio::test]
async fn test_server_constructors_and_builder_chaining() {
    let tmp = match TempDir::new() {
        Ok(t) => t,
        Err(e) => panic!("temp dir failed: {e}"),
    };
    let db = match Contextra::open(tmp.path()).await {
        Ok(d) => d,
        Err(e) => panic!("open db failed: {e}"),
    };
    let embedder = Arc::new(MockEmbedder { dimension: 4 });

    // 1. Default constructor
    let server_default = match McpServer::new(db.clone(), embedder.clone()) {
        Ok(s) => s,
        Err(e) => panic!("McpServer::new failed: {e}"),
    };
    assert!(server_default.sandbox.policy().allow_db_reads);

    // 2. with_write_permission
    let server_readonly =
        match McpServer::with_write_permission(db.clone(), embedder.clone(), false) {
            Ok(s) => s,
            Err(e) => panic!("readonly failed: {e}"),
        };
    assert!(!server_readonly.sandbox.policy().allow_db_writes);

    let server_write = match McpServer::with_write_permission(db.clone(), embedder.clone(), true) {
        Ok(s) => s,
        Err(e) => panic!("writeable failed: {e}"),
    };
    assert!(server_write.sandbox.policy().allow_db_writes);

    // 3. with_sandbox
    let custom_policy = SandboxPolicy {
        allow_db_reads: true,
        allow_db_writes: false,
        allow_code_execution: false,
        allow_cloud_egress: true,
        max_execution_ms: 10_000,
    };
    let custom_sandbox = match McpSandbox::new(custom_policy) {
        Ok(sb) => Arc::new(sb),
        Err(e) => panic!("custom sandbox creation failed: {e}"),
    };
    let server_custom = McpServer::with_sandbox(db, embedder, custom_sandbox);
    assert_eq!(server_custom.sandbox.policy().max_execution_ms, 10_000);
    assert!(server_custom.sandbox.policy().allow_cloud_egress);

    // 4. Builder methods
    let tenant = match TenantId::try_new(42) {
        Ok(t) => t,
        Err(e) => panic!("valid tenant failed: {e}"),
    };
    let guard = Arc::new(PromptInjectionGuard::default());
    let classifier = Arc::new(DefaultEgressClassifier::default());

    let configured_server = server_custom
        .with_tenant_id(tenant)
        .with_injection_guard(guard)
        .with_egress_classifier(classifier);

    assert_eq!(configured_server.tenant_id, Some(tenant));
}

#[tokio::test]
async fn test_server_stdio_lifecycle_simulation_and_shutdown_idempotency() {
    let tmp = match TempDir::new() {
        Ok(t) => t,
        Err(e) => panic!("temp dir failed: {e}"),
    };
    let db = match Contextra::open(tmp.path()).await {
        Ok(d) => d,
        Err(e) => panic!("open db failed: {e}"),
    };
    let embedder = Arc::new(MockEmbedder { dimension: 4 });
    let server = Arc::new(match McpServer::new(db, embedder) {
        Ok(s) => s,
        Err(e) => panic!("server init failed: {e}"),
    });

    // 1. Simulating incoming valid request and verifying response
    let valid_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "ping"
    });
    let resp = match server.handle_value(valid_req).await {
        Some(v) => v,
        None => panic!("response expected"),
    };
    assert_eq!(resp["id"], 1);
    assert_eq!(resp["result"], json!({}));

    // 2. Simulating EOF on stream (clean shutdown on stream close)
    let empty_input: &[u8] = b"";
    let mut reader = tokio::io::BufReader::new(empty_input);
    let mut line = String::new();
    let bytes_read = match contextra_mcp::io::read_line_bounded(&mut reader, &mut line).await {
        Ok(b) => b,
        Err(e) => panic!("read EOF failed: {e}"),
    };
    assert_eq!(bytes_read, 0, "EOF must read 0 bytes indicating clean stop");

    // 3. Idempotent shutdown: subsequent read on closed stream remains EOF
    let bytes_read_again = match contextra_mcp::io::read_line_bounded(&mut reader, &mut line).await
    {
        Ok(b) => b,
        Err(e) => panic!("second read EOF failed: {e}"),
    };
    assert_eq!(
        bytes_read_again, 0,
        "Repeated EOF must be idempotent 0 bytes"
    );

    // 4. Invalid configuration failure: invalid tenant ID in configuration
    let invalid_tenant_res = TenantId::try_new(0);
    assert!(
        invalid_tenant_res.is_err(),
        "Tenant 0 must fail validation at startup"
    );
}
