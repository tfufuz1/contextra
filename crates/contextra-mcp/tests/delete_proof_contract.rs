// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Vertragsprüfungen für contextra_delete und contextra_drop_collection Beweisstatus (Auftrag A04 / F4)
// INVARIANTEN: Zero-Panic in Mocks, Fail-Closed, kein "proof": null ohne Status, INV-MCP-CLASSIFY-1 beibehalten.

use contextra::Contextra;
use contextra_mcp::sandbox::{McpSandbox, ToolCategory};
use contextra_mcp::server::McpServer;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

async fn create_test_server() -> (Arc<McpServer>, TempDir) {
    let tmp = TempDir::new().expect("temp dir creation");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let server = McpServer::with_write_permission(Arc::new(db), None, true)
        .expect("mcp server initialization");
    (Arc::new(server), tmp)
}

#[tokio::test]
async fn test_delete_returns_proof_status_and_no_null_proof() {
    let (server, _tmp) = create_test_server().await;

    let delete_args = json!({
        "collection": "default",
        "id": "doc_123"
    });

    let res = server
        .handle_delete(&delete_args)
        .await
        .expect("handle_delete should succeed");

    // Invariante F4: Kein proof: null ohne Statusfeld!
    assert!(
        res.get("proof").is_none(),
        "Response MUST NOT contain explicit 'proof': null without status"
    );
    assert_eq!(
        res.get("proof_status").and_then(|v| v.as_str()),
        Some("logical_only"),
        "proof_status MUST be 'logical_only' for contextra_delete"
    );
    assert_eq!(
        res.get("proof_scope").and_then(|v| v.as_str()),
        Some("collection_only"),
        "proof_scope MUST be 'collection_only' for contextra_delete"
    );
    assert_eq!(res.get("ok").and_then(|v| v.as_bool()), Some(true));
}

#[tokio::test]
async fn test_delete_engine_error_propagation() {
    let (server, _tmp) = create_test_server().await;

    // Ungültige ID (z.B. leer oder überschreitet Max-Länge) -> muss MCP-Fehler liefern
    let invalid_args = json!({
        "collection": "default",
        "id": ""
    });

    let err = server
        .handle_delete(&invalid_args)
        .await
        .expect_err("handle_delete with empty ID must fail");

    let err_msg = err.to_string();
    assert!(
        err_msg.contains("cannot be empty"),
        "Error message should reflect invalid parameter: {err_msg}"
    );
}

#[tokio::test]
async fn test_mcp_classify_tool_category_maintained() {
    // INV-MCP-CLASSIFY-1
    let cat_delete = McpSandbox::classify_method("contextra_delete");
    assert_eq!(
        cat_delete,
        ToolCategory::DatabaseWrite,
        "contextra_delete category MUST remain DatabaseWrite"
    );

    let cat_drop = McpSandbox::classify_method("contextra_drop_collection");
    assert_eq!(
        cat_drop,
        ToolCategory::DatabaseWrite,
        "contextra_drop_collection category MUST remain DatabaseWrite"
    );
}
