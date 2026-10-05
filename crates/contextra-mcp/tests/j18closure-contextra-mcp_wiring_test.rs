// FILE-CONTEXT
// STAND:       2026-10-05
// ZWECK:       Integration wiring test suite for J18 closure symbols in contextra-mcp
// INVARIANTEN: Direct invocation through production McpServer::from_env and get_volatile_output facades

use contextra::Contextra;
use contextra_mcp::McpServer;
use contextra_ports::MockEmbedder;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_j18closure_mcp_server_from_env_and_volatile_integration() {
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", "j18closure_proof_key_secret_32b!");
    std::env::set_var("CONTEXTRA_LLM_PROVIDER", "mock");

    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let db = Arc::new(
        Contextra::open(tmp_dir.path().to_str().expect("valid path"))
            .await
            .expect("open db"),
    );
    let embedder: Arc<dyn contextra_ports::EmbeddingProvider> = Arc::new(MockEmbedder::new(768));

    // Exercise McpServer::from_env which internally calls:
    // - LlmConfig::from_env().build_generator()
    // - deletion_proof_key_from_env()
    // - with_injection_guard(...)
    // - with_egress_classifier(...)
    // - with_plugin_registry(...)
    let server = Arc::new(
        McpServer::from_env(db, embedder).expect("McpServer::from_env initialization succeeded"),
    );

    // Verify volatile storage roundtrip using McpServer::get_volatile_output -> McpSandbox::get_volatile
    let key = "j18_closure_test_volatile_key";
    let data = b"sensitive volatile payload output";
    server
        .sandbox
        .store_volatile(key, data)
        .expect("store_volatile");

    let retrieved = server
        .get_volatile_output(key)
        .expect("get_volatile_output succeeded")
        .expect("volatile entry exists");
    assert_eq!(retrieved.as_slice(), data);

    let nonexistent = server
        .get_volatile_output("nonexistent_closure_key")
        .expect("get_volatile_output succeeded");
    assert!(nonexistent.is_none());

    // Exercise JSON-RPC handle_value through the server
    let req_plugin_status = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "contextra_plugin_status",
            "arguments": {}
        }
    });
    let resp = server.handle_value(req_plugin_status).await;
    assert!(resp.is_some(), "Expected JSON-RPC response for contextra_plugin_status");

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
    std::env::remove_var("CONTEXTRA_LLM_PROVIDER");
}
