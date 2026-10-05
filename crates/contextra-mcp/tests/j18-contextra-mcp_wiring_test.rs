// FILE-CONTEXT
// STAND:       2026-10-04
// ZWECK:       Integration wiring test suite for J18 symbols in contextra-mcp
// INVARIANTEN: Direct invocation through enclosing production paths and builder facades

use contextra::Contextra;
use contextra_mcp::sandbox::{McpSandbox, SandboxPolicy};
use contextra_mcp::{
    deletion_proof_key_from_env, EgressClassification, EgressClassifier, LlmConfig, McpServer,
    PromptInjectionGuard,
};
use contextra_ports::MockEmbedder;
use serde_json::json;
use std::sync::Arc;

struct CustomEgressClassifier;

impl EgressClassifier for CustomEgressClassifier {
    fn classify<'a>(
        &'a self,
        _query: &'a str,
    ) -> contextra_ports::BoxFuture<'a, EgressClassification> {
        Box::pin(async move { EgressClassification::Allow })
    }
}

#[tokio::test]
async fn test_j18_llm_config_build_generator_integration() {
    let config = LlmConfig::default();
    let generator = config.build_generator().expect("build generator default");
    let response = generator
        .generate("hello from integration test")
        .await
        .expect("mock generate response");
    assert!(response.contains("[Mock LLM response for: hello from integration test]"));

    let env_config = LlmConfig::from_env();
    let env_generator = env_config.build_generator().expect("build generator from_env");
    let env_response = env_generator
        .generate("hello from env generator")
        .await
        .expect("mock generate env response");
    assert!(env_response.contains("[Mock LLM response for: hello from env generator]"));
}

#[test]
fn test_j18_deletion_proof_key_from_env_integration() {
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", "integration_test_key_32bytes!!");
    let resolved = deletion_proof_key_from_env().expect("deletion proof key resolved");
    assert_eq!(resolved.as_str(), "integration_test_key_32bytes!!");
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[test]
fn test_j18_sandbox_get_volatile_integration() {
    let sandbox = McpSandbox::new(SandboxPolicy::default()).expect("sandbox created");
    let data = b"sensitive temporary token output";
    sandbox
        .store_volatile("volatile_token_key", data)
        .expect("store volatile success");

    let retrieved = sandbox
        .get_volatile("volatile_token_key")
        .expect("get volatile call succeeded")
        .expect("entry exists");
    assert_eq!(retrieved.as_slice(), data);

    let nonexistent = sandbox
        .get_volatile("nonexistent_key")
        .expect("call succeeded");
    assert!(nonexistent.is_none());
}

#[tokio::test]
async fn test_j18_server_builder_wiring_integration() {
    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let db = Arc::new(
        Contextra::open(tmp_dir.path().to_str().expect("valid path"))
            .await
            .expect("open db"),
    );
    let embedder: Arc<dyn contextra_ports::EmbeddingProvider> = Arc::new(MockEmbedder::new(768));

    // Construct custom builder options
    let policy = SandboxPolicy {
        allow_db_reads: true,
        allow_db_writes: false,
        allow_code_execution: false,
        allow_cloud_egress: true,
        max_execution_ms: 5_000,
    };
    let sandbox = Arc::new(McpSandbox::new(policy).expect("sandbox"));

    let injection_guard = Arc::new(PromptInjectionGuard::from_env());
    let egress_classifier: Arc<dyn EgressClassifier> = Arc::new(CustomEgressClassifier);
    let plugin_registry: Option<Arc<contextra_ports::plugin::PluginRegistry>> = None;

    // Connect custom builder dependencies
    let server = Arc::new(
        McpServer::with_sandbox(db, embedder, sandbox)
            .with_injection_guard(injection_guard)
            .with_egress_classifier(egress_classifier)
            .with_plugin_registry(plugin_registry),
    );

    // Exercise enclosing production path: handle_value for contextra_plugin_status
    let req_plugin_status = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "contextra_plugin_status",
            "arguments": {}
        }
    });
    let resp_plugin = server.handle_value(req_plugin_status).await;
    assert!(resp_plugin.is_some(), "Expected response for contextra_plugin_status");

    // Exercise enclosing production path: handle_value for contextra_cloud_query
    let req_cloud_query = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "contextra_cloud_query",
            "arguments": {
                "query": "search query"
            }
        }
    });
    let resp_cloud = server.handle_value(req_cloud_query).await;
    assert!(resp_cloud.is_some(), "Expected response for contextra_cloud_query");
}
