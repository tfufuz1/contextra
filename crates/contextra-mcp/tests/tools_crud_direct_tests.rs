use contextra::Contextra;
use contextra_mcp::{protocol::JsonRpcRequest, McpServer};
use contextra_ports::BoxFuture;
use contextra_types::TenantId;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

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

async fn create_crud_server() -> (McpServer, TempDir) {
    let tmp = match TempDir::new() {
        Ok(t) => t,
        Err(e) => panic!("temp dir failed: {e}"),
    };
    let db = match Contextra::open(tmp.path()).await {
        Ok(d) => d,
        Err(e) => panic!("open db failed: {e}"),
    };
    let embedder = Arc::new(MockEmbedder { dimension: 4 });
    let tenant = match TenantId::try_new(100) {
        Ok(t) => t,
        Err(e) => panic!("valid tenant failed: {e}"),
    };
    let server = match McpServer::with_write_permission(db, embedder, true) {
        Ok(s) => s.with_tenant_id(tenant),
        Err(e) => panic!("server init failed: {e}"),
    };
    (server, tmp)
}

fn call_tool(name: &str, arguments: Value) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": name,
            "arguments": arguments
        })),
    }
}

#[tokio::test]
async fn test_crud_create_collection_all_tiers_and_validation() {
    let (server, _tmp) = create_crud_server().await;

    // 1. Missing name/collection parameter
    let resp_missing = server
        .handle(call_tool("contextra_create_collection", json!({})))
        .await;
    let res_missing = match resp_missing.result {
        Some(r) => r,
        None => panic!("result expected"),
    };
    assert_eq!(res_missing["isError"], true);

    // 2. Tiers: memory, standard, ha, regulated
    for tier in &["memory", "standard", "ha", "regulated"] {
        let col_name = format!("col_tier_{tier}");
        let resp = server
            .handle(call_tool(
                "contextra_create_collection",
                json!({
                    "name": col_name,
                    "tier": tier,
                    "auto_extraction": true
                }),
            ))
            .await;
        let res = match resp.result {
            Some(r) => r,
            None => panic!("result expected for tier {tier}"),
        };
        assert_ne!(res["isError"], true, "Tier {tier} should succeed: {res:?}");
        let content_text = res["content"][0]["text"].as_str().unwrap_or("");
        assert!(content_text.contains("erstellt") || content_text.contains("Collection"));
    }

    // 3. Unknown tier falls back or fails gracefully
    let resp_bad_tier = server
        .handle(call_tool(
            "contextra_create_collection",
            json!({
                "name": "col_unknown_tier",
                "tier": "non_existent_tier_xyz"
            }),
        ))
        .await;
    let res_bad = match resp_bad_tier.result {
        Some(r) => r,
        None => panic!("result expected for bad tier"),
    };
    assert_ne!(
        res_bad["isError"], true,
        "Unknown tier falls back to default safely"
    );
}

#[tokio::test]
async fn test_crud_drop_collection_confirm_and_key_semantics() {
    let _guard = match ENV_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let (server, _tmp) = create_crud_server().await;

    // Ensure collection exists
    let _ = server
        .handle(call_tool(
            "contextra_create_collection",
            json!({ "name": "drop_test_col" }),
        ))
        .await;

    // 1. Drop without confirm: true must fail with confirmation requirement
    let resp_unconfirmed = server
        .handle(call_tool(
            "contextra_drop_collection",
            json!({ "collection": "drop_test_col" }),
        ))
        .await;
    let res_unconfirmed = match resp_unconfirmed.result {
        Some(r) => r,
        None => panic!("result expected"),
    };
    assert_eq!(
        res_unconfirmed["isError"], true,
        "Unconfirmed drop must return isError: true"
    );
    let text = res_unconfirmed["content"][0]["text"].as_str().unwrap_or("");
    assert!(
        text.contains("confirm: true") || text.contains("Destruktive Operation"),
        "Unexpected message: {text}"
    );

    // 2. Drop with confirm: true but MISSING proof key must fail preflight BEFORE deletion
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
    let resp_no_key = server
        .handle(call_tool(
            "contextra_drop_collection",
            json!({ "collection": "drop_test_col", "confirm": true }),
        ))
        .await;
    let res_no_key = match resp_no_key.result {
        Some(r) => r,
        None => panic!("result expected"),
    };
    assert_eq!(
        res_no_key["isError"], true,
        "Drop without proof key must fail"
    );
    let text_no_key = res_no_key["content"][0]["text"].as_str().unwrap_or("");
    assert!(
        text_no_key.contains("proof key")
            || text_no_key.contains("key")
            || text_no_key.contains("Löschbeweis"),
        "Unexpected message: {text_no_key}"
    );

    // 3. Drop with valid proof key and confirm: true must succeed and produce measured report
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", "test_key_crud_mcp_12345");
    let resp_success = server
        .handle(call_tool(
            "contextra_drop_collection",
            json!({ "collection": "drop_test_col", "confirm": true }),
        ))
        .await;
    let res_success = match resp_success.result {
        Some(r) => r,
        None => panic!("result expected"),
    };
    assert_ne!(
        res_success["isError"], true,
        "Drop with key and confirm must succeed: {res_success:?}"
    );
    let text_success = res_success["content"][0]["text"].as_str().unwrap_or("");
    let json_res: Value = match serde_json::from_str(text_success) {
        Ok(v) => v,
        Err(e) => panic!("json payload in content invalid: {e}"),
    };
    assert_eq!(json_res["ok"], true);
    assert_eq!(json_res["deleted"], true);
    assert!(json_res.get("proof").is_some());

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[tokio::test]
async fn test_crud_drop_collection_tenant_isolation_rule_7() {
    let _guard = match ENV_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", "test_key_rule_7");

    let (server, _tmp) = create_crud_server().await;

    // Create under server's tenant (100)
    let _ = server
        .handle(call_tool(
            "contextra_create_collection",
            json!({ "name": "tenant_col_test" }),
        ))
        .await;

    // Passing a different tenant in arguments ("tenant": 999) must NOT override the server context tenant
    // (Rule 7: Mandant kommt aus RequestContext / server.tenant_id, nie aus Nutzerparametern)
    let resp = server
        .handle(call_tool(
            "contextra_drop_collection",
            json!({
                "collection": "tenant_col_test",
                "confirm": true,
                "tenant": 999,
                "tenant_id": 999
            }),
        ))
        .await;
    let res = match resp.result {
        Some(r) => r,
        None => panic!("result expected"),
    };
    assert_ne!(
        res["isError"], true,
        "Drop must succeed using server-bound tenant"
    );

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[tokio::test]
async fn test_crud_insert_upsert_get_and_delete_document() {
    let _guard = match ENV_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", "test_key_doc_delete");

    let (server, _tmp) = create_crud_server().await;

    // 1. Insert via Markdown chunking
    let resp_insert = server
        .handle(call_tool(
            "contextra_insert",
            json!({
                "collection": "default",
                "text": "# Heading 1\nSome introductory text for chunking.\n## Heading 2\nMore detailed content.",
                "metadata": { "author": "Jules" }
            }),
        ))
        .await;
    let res_insert = match resp_insert.result {
        Some(r) => r,
        None => panic!("insert result expected"),
    };
    assert_ne!(res_insert["isError"], true);

    // 2. Upsert with explicit vector
    let resp_upsert = server
        .handle(call_tool(
            "contextra_upsert",
            json!({
                "collection": "default",
                "id": 1,
                "text": "Direct single document text",
                "vector": [0.1, 0.2, 0.3, 0.4]
            }),
        ))
        .await;
    let res_upsert = match resp_upsert.result {
        Some(r) => r,
        None => panic!("upsert result expected"),
    };
    assert_ne!(res_upsert["isError"], true);

    // 3. Get document
    let resp_get = server
        .handle(call_tool(
            "contextra_get",
            json!({ "collection": "default", "id": 1 }),
        ))
        .await;
    let res_get = match resp_get.result {
        Some(r) => r,
        None => panic!("get result expected"),
    };
    assert_ne!(res_get["isError"], true);

    // 4. Delete document (Rule 10: Schichteigene Messung für Einzel-Dokument-Löschung nicht angebunden)
    let resp_del = server
        .handle(call_tool(
            "contextra_delete",
            json!({ "collection": "default", "id": 1 }),
        ))
        .await;
    let res_del = match resp_del.result {
        Some(r) => r,
        None => panic!("delete result expected"),
    };
    assert_ne!(res_del["isError"], true);
    let text_del = res_del["content"][0]["text"].as_str().unwrap_or("");
    let json_del: Value = match serde_json::from_str(text_del) {
        Ok(v) => v,
        Err(e) => panic!("valid json in delete response expected: {e}"),
    };
    assert_eq!(json_del["ok"], true);
    assert_eq!(json_del["deleted"], true);
    assert_eq!(json_del["verified"], false);
    assert_eq!(json_del["proof"], Value::Null);
    assert!(json_del["reason"]
        .as_str()
        .unwrap_or("")
        .contains("Einzel-Dokument-Löschung nicht angebunden"));

    // 5. Delete with missing ID must fail
    let resp_del_missing = server
        .handle(call_tool(
            "contextra_delete",
            json!({ "collection": "default" }),
        ))
        .await;
    let res_missing = match resp_del_missing.result {
        Some(r) => r,
        None => panic!("missing id result expected"),
    };
    assert_eq!(res_missing["isError"], true);

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[tokio::test]
async fn test_crud_contextra_forget_tool() {
    let _guard = match ENV_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", "test_key_forget");

    let (server, _tmp) = create_crud_server().await;

    // Ensure collection exists
    let _ = server
        .handle(call_tool(
            "contextra_create_collection",
            json!({ "name": "forget_col" }),
        ))
        .await;

    // 1. contextra_forget without confirm: true must fail
    let resp_unconfirmed = server
        .handle(call_tool(
            "contextra_forget",
            json!({ "collection": "forget_col" }),
        ))
        .await;
    let res_unconfirmed = match resp_unconfirmed.result {
        Some(r) => r,
        None => panic!("result expected"),
    };
    assert_eq!(res_unconfirmed["isError"], true);

    // 2. contextra_forget single document with confirm: true
    let resp_doc = server
        .handle(call_tool(
            "contextra_forget",
            json!({ "collection": "forget_col", "id": 42, "confirm": true }),
        ))
        .await;
    let res_doc = match resp_doc.result {
        Some(r) => r,
        None => panic!("result expected"),
    };
    assert_ne!(res_doc["isError"], true);

    // 3. contextra_forget whole collection with confirm: true
    let resp_col = server
        .handle(call_tool(
            "contextra_forget",
            json!({ "collection": "forget_col", "confirm": true }),
        ))
        .await;
    let res_col = match resp_col.result {
        Some(r) => r,
        None => panic!("result expected"),
    };
    assert_ne!(res_col["isError"], true);

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[tokio::test]
async fn test_crud_search_and_explain() {
    let (server, _tmp) = create_crud_server().await;

    // Search with query and limit capping
    let resp_search = server
        .handle(call_tool(
            "contextra_search",
            json!({
                "collection": "default",
                "query": "test query",
                "k": 500 // should be clamped to MAX_SEARCH_K (100)
            }),
        ))
        .await;
    let res_search = match resp_search.result {
        Some(r) => r,
        None => panic!("search result expected"),
    };
    assert_ne!(res_search["isError"], true);

    // Explain with query and id
    let resp_explain = server
        .handle(call_tool(
            "contextra_explain",
            json!({
                "collection": "default",
                "query": "explain test",
                "id": 1
            }),
        ))
        .await;
    let res_explain = match resp_explain.result {
        Some(r) => r,
        None => panic!("explain result expected"),
    };
    assert_ne!(res_explain["isError"], true);
}
