#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::Contextra;
use contextra_mcp::protocol::JsonRpcRequest;
use contextra_mcp::sandbox::{McpSandbox, SandboxPolicy};
use contextra_mcp::McpServer;
use contextra_ports::BoxFuture;
use contextra_types::MAX_SEARCH_K;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
async fn test_crud_create_collection_variants_and_boundaries() {
    let (server, _tmp) = create_test_server().await;

    // 1. Valid creation with default tier
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_create_collection",
            "arguments": {
                "collection": "test_default_tier"
            }
        }),
    };
    let resp = server.handle(req).await;
    let res = resp.result.expect("result present");
    assert_ne!(res["isError"], true);
    let text = res["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("\"ok\":true"));
    assert!(text.contains("test_default_tier"));
    assert!(text.contains("PowerUserLocal"));

    // 2. Valid creation with explicit tiers and auto_extraction
    let tiers = [
        "EdgeMinimal",
        "PowerUserLocal",
        "EnterpriseShared",
        "EnterpriseRegulated",
    ];
    for (i, tier) in tiers.iter().enumerate() {
        let name = format!("col_{}", tier.to_lowercase());
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(10 + i)),
            method: "tools/call".to_string(),
            params: json!({
                "name": "contextra_create_collection",
                "arguments": {
                    "collection": name,
                    "deployment_tier": tier,
                    "auto_extraction": "enabled"
                }
            }),
        };
        let resp = server.handle(req).await;
        let res = resp.result.expect("result present");
        assert_ne!(res["isError"], true);
        let text = res["content"][0]["text"].as_str().unwrap();
        assert!(text.contains(tier));
        assert!(text.contains("enabled"));
    }

    // 3. Error: missing collection name
    let req_missing = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(20)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_create_collection",
            "arguments": {}
        }),
    };
    let resp_missing = server.handle(req_missing).await;
    let res_missing = resp_missing.result.expect("result present");
    assert_eq!(res_missing["isError"], true);
    assert!(res_missing["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required field"));

    // 4. Error: empty collection name
    let req_empty = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(21)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_create_collection",
            "arguments": { "collection": "   " }
        }),
    };
    let resp_empty = server.handle(req_empty).await;
    let res_empty = resp_empty.result.expect("result present");
    assert_eq!(res_empty["isError"], true);
    assert!(res_empty["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("collection name cannot be empty"));

    // 5. Error: invalid tier
    let req_tier = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(22)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_create_collection",
            "arguments": {
                "collection": "invalid_tier_col",
                "deployment_tier": "SuperCloudTier"
            }
        }),
    };
    let resp_tier = server.handle(req_tier).await;
    let res_tier = resp_tier.result.expect("result present");
    assert_eq!(res_tier["isError"], true);
    assert!(res_tier["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("Invalid deployment_tier"));

    // 6. Error: invalid auto_extraction
    let req_ae = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(23)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_create_collection",
            "arguments": {
                "collection": "invalid_ae_col",
                "auto_extraction": "maybe"
            }
        }),
    };
    let resp_ae = server.handle(req_ae).await;
    let res_ae = resp_ae.result.expect("result present");
    assert_eq!(res_ae["isError"], true);
    assert!(res_ae["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("Invalid auto_extraction"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_crud_drop_collection_preflight_and_tenant_origin() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let (server, _tmp) = create_test_server().await;

    // 1. Missing confirm -> error
    let req_no_confirm = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(30)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_drop_collection",
            "arguments": { "collection": "default" }
        }),
    };
    let resp_no_confirm = server.handle(req_no_confirm).await;
    let res = resp_no_confirm.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("confirm parameter must be explicitly set to true"));

    // 2. Preflight key check: fails when CONTEXTRA_DELETION_PROOF_KEY is unset
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
    let req_no_key = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(31)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_drop_collection",
            "arguments": {
                "collection": "default",
                "confirm": true
            }
        }),
    };
    let resp_no_key = server.handle(req_no_key).await;
    let res = resp_no_key.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("deletion proof key not configured"));

    // 3. Setting proof key
    std::env::set_var(
        "CONTEXTRA_DELETION_PROOF_KEY",
        "test_key_32_bytes_long_exact!!",
    );

    // 4. Tenant origin check: User passing tenant_id parameter must not override server tenant_id
    // If server has tenant_id = None, it must reject with "tenant_id not configured on server"
    let server_no_tenant = McpServer::with_sandbox(
        server.db.clone(),
        server.embedder.clone(),
        server.sandbox.clone(),
    )
    .with_tenant_id(None);

    let req_tenant = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(32)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_drop_collection",
            "arguments": {
                "collection": "default",
                "confirm": true,
                "tenant_id": 999 // Attacker tries to inject tenant parameter
            }
        }),
    };
    let resp_tenant = server_no_tenant.handle(req_tenant).await;
    let res = resp_tenant.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("tenant_id not configured on server"));

    // Create non-default collection to test drop_collection
    let req_create = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(33)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_create_collection",
            "arguments": {
                "collection": "col_to_drop"
            }
        }),
    };
    let _ = server.handle(req_create).await;

    // 5. Successful drop collection with configured tenant and key
    let req_ok = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(34)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_drop_collection",
            "arguments": {
                "collection": "col_to_drop",
                "confirm": true,
                "tenant_id": 999 // Injected tenant is ignored; server.tenant_id is used
            }
        }),
    };
    let resp_ok = server.handle(req_ok).await;
    let res = resp_ok.result.unwrap();
    assert_ne!(res["isError"], true);
    let text = res["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("\"ok\":true"));
    assert!(text.contains("\"verified\":true"));

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_crud_handle_delete_boundaries_and_semantics() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let (server, _tmp) = create_test_server().await;

    // 1. Missing id
    let req_no_id = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(40)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_delete",
            "arguments": { "collection": "default" }
        }),
    };
    let resp_no_id = server.handle(req_no_id).await;
    let res = resp_no_id.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required field 'id'"));

    // 2. Empty id
    let req_empty_id = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(41)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_delete",
            "arguments": { "id": "  " }
        }),
    };
    let resp_empty_id = server.handle(req_empty_id).await;
    let res = resp_empty_id.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("id cannot be empty"));

    // 3. Id length exceeds 256 chars limit
    let long_id = "x".repeat(257);
    let req_long_id = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(42)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_delete",
            "arguments": { "id": long_id }
        }),
    };
    let resp_long_id = server.handle(req_long_id).await;
    let res = resp_long_id.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("id length exceeds limit"));

    // 4. Key preflight check: fails when proof key is missing
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
    let req_no_key = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(43)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_delete",
            "arguments": { "id": "doc1" }
        }),
    };
    let resp_no_key = server.handle(req_no_key).await;
    let res = resp_no_key.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("deletion proof key not configured"));

    // 5. Successful single delete with key set
    std::env::set_var(
        "CONTEXTRA_DELETION_PROOF_KEY",
        "test_key_32_bytes_long_exact!!",
    );
    let req_ok = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(44)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_delete",
            "arguments": { "id": "doc1" }
        }),
    };
    let resp_ok = server.handle(req_ok).await;
    let res = resp_ok.result.unwrap();
    assert_ne!(res["isError"], true);
    let text = res["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("\"ok\":true"));
    assert!(text.contains("\"deleted\":true"));
    assert!(text.contains("\"verified\":false"));
    assert!(text.contains("\"claim\":\"LogicalDeletionOnly\""));

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_crud_insert_upsert_and_get() {
    let (server, _tmp) = create_test_server().await;

    // 1. Insert with text (Markdown chunking)
    let insert_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_insert",
            "arguments": {
                "id": "doc_prose",
                "text": "First section of prose\n\nSecond section with more content.",
                "metadata": { "tag": "test" }
            }
        }),
    };
    let resp = server.handle(insert_req).await;
    assert_eq!(resp.id, Some(json!(1)));
    assert!(resp.error.is_none());
    let res = resp.result.expect("result present");
    assert_ne!(res["isError"], true);

    // 2. Get inserted document
    let get_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_get",
            "arguments": { "id": "doc_prose" }
        }),
    };
    let get_resp = server.handle(get_req).await;
    assert!(get_resp.error.is_none());

    // 3. Upsert with explicit vector
    let dim = server.embedder.embedding_dim();
    let upsert_vec = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_upsert",
            "arguments": {
                "id": "doc_vec",
                "vector": vec![0.5f32; dim],
                "metadata": { "type": "vector_doc" }
            }
        }),
    };
    let upsert_resp = server.handle(upsert_vec).await;
    assert!(upsert_resp.error.is_none());

    // 4. Missing both text and vector -> error
    let req_neither = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(4)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_insert",
            "arguments": { "id": "doc_empty" }
        }),
    };
    let resp_neither = server.handle(req_neither).await;
    let res = resp_neither.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required field 'vector' or 'text'"));

    // 5. Invalid vector (empty array) -> error
    let req_empty_vec = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(5)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_insert",
            "arguments": {
                "id": "doc_bad_vec",
                "vector": []
            }
        }),
    };
    let resp_empty_vec = server.handle(req_empty_vec).await;
    let res = resp_empty_vec.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("vector cannot be empty"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_crud_search_and_explain() {
    let (server, _tmp) = create_test_server().await;

    // Search query boundaries: empty query -> error
    let req_empty_q = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(10)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_search",
            "arguments": { "query": "  " }
        }),
    };
    let resp_empty_q = server.handle(req_empty_q).await;
    let res = resp_empty_q.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("query cannot be empty"));

    // Valid search with k > MAX_SEARCH_K (should be capped without error)
    let req_large_k = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(11)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_search",
            "arguments": {
                "query": "rust language",
                "k": MAX_SEARCH_K + 500
            }
        }),
    };
    let resp_large_k = server.handle(req_large_k).await;
    assert!(resp_large_k.error.is_none());

    // Explain tool call
    let req_explain = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(12)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_explain",
            "arguments": {
                "id": "doc_prose"
            }
        }),
    };
    let resp_explain = server.handle(req_explain).await;
    assert!(resp_explain.error.is_none());
}
