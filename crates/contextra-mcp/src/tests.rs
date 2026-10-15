#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[cfg(test)]
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::McpServer;
use contextra::Contextra;
use contextra_ports::BoxFuture;
use serde_json::{json, Value};
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

async fn create_mock_server() -> (Arc<McpServer>, TempDir) {
    create_mock_server_with_write(true).await
}

async fn create_mock_server_with_write(allow_db_writes: bool) -> (Arc<McpServer>, TempDir) {
    use crate::sandbox::{McpSandbox, SandboxPolicy};
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let collection = db.collection("default").await.expect("collection");
    let dim = collection.dimension();
    let embedder = Arc::new(MockEmbedder { dimension: dim });
    let policy = SandboxPolicy {
        allow_db_reads: true,
        allow_db_writes,
        allow_code_execution: false,
        allow_cloud_egress: false,
        max_execution_ms: 5_000,
        sidecar_limits: crate::sandbox::SidecarLimits::default(),
    };
    let sandbox = Arc::new(McpSandbox::new(policy).expect("sandbox new"));
    let server = Arc::new(McpServer::with_sandbox(Arc::new(db), embedder, sandbox));
    (server, tmp)
}

fn make_request(method: &str, params: Value) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        method: method.into(),
        params,
    }
}

/// Exact list of core tools expected in MCP server
pub const EXPECTED_CORE_TOOLS: &[&str] = &[
    "contextra_search",
    "contextra_insert",
    "contextra_get",
    "contextra_forget",
    "contextra_collections",
    "contextra_plugin_status",
    "contextra_upsert",
    "contextra_delete",
    "contextra_create_collection",
    "contextra_drop_collection",
];

#[tokio::test]
async fn test_tools_list_returns_exact_core_tools() {
    let (server, _tmp) = create_mock_server().await;
    let req = make_request("tools/list", json!({}));
    let response = server.handle(req).await;
    assert_eq!(response.jsonrpc, "2.0");
    let tools = response.result.unwrap()["tools"]
        .as_array()
        .unwrap()
        .clone();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert_eq!(tools.len(), EXPECTED_CORE_TOOLS.len());
    for expected_name in EXPECTED_CORE_TOOLS {
        assert!(
            names.contains(expected_name),
            "Expected tool '{expected_name}' missing from tools/list"
        );
    }

    // Dynamic assertion: every tool returned in tools/list MUST NOT classify as CodeExecution (INV-MCP-CLASSIFY-1)
    for name in &names {
        let cat = crate::sandbox::McpSandbox::try_classify_method(name).expect("registered tool");
        assert_ne!(
            cat,
            crate::sandbox::ToolCategory::CodeExecution,
            "Tool '{name}' listed in tools/list classified as CodeExecution"
        );
    }
}

/// Verification Test: Confirms that an unregistered method classifies as CodeExecution fallback
/// and is rejected fail-closed under default SandboxPolicy.
#[tokio::test]
async fn test_unknown_method_rejected_via_code_execution_fail_closed_path() {
    use crate::sandbox::{McpSandbox, SandboxPolicy, ToolCategory};

    // 1. Direct classification of unlisted method yields CodeExecution fallback
    let unknown_method = "unlisted_arbitrary_eval";
    #[allow(deprecated)]
    let legacy_cat = McpSandbox::classify_method(unknown_method);
    assert_eq!(
        legacy_cat,
        ToolCategory::CodeExecution,
        "Unknown method must fall back to CodeExecution category"
    );

    // 2. Under default policy (allow_code_execution = false), validation rejects unknown method
    let default_sandbox = McpSandbox::new(SandboxPolicy::default()).unwrap();
    let val_res = default_sandbox.validate_tool_call(unknown_method, &json!({}));
    assert!(val_res.is_err());
    let err_msg = val_res.unwrap_err().to_string();
    assert!(
        err_msg.contains("unknown tool") || err_msg.contains("Code-Ausführung ist gesperrt"),
        "Expected unknown tool / code execution rejection, got: '{err_msg}'"
    );

    // 3. RPC server invocation of unknown method via tools/call returns isError in tool execution
    let (server, _tmp) = create_mock_server().await;
    let req = make_request(
        "tools/call",
        json!({
            "name": unknown_method,
            "arguments": {}
        }),
    );
    let resp = server.handle(req).await;
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_eq!(res_val["result"]["isError"], true);
    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("unknown tool") || text.contains("Code-Ausführung"),
        "Expected unknown tool error text in tools/call response, got: '{text}'"
    );
}

/// Inv-MCP-CLASSIFY-1 Verification Test: Iterates over tools/list and asserts explicit classification.
#[tokio::test]
async fn test_inv_mcp_classify_1_tools_list_classification_completeness_and_mapping() {
    use crate::sandbox::ToolCategory;

    const EXPECTED_TOOLS: &[(&str, ToolCategory)] = &[
        ("contextra_search", ToolCategory::DatabaseRead),
        ("contextra_insert", ToolCategory::DatabaseWrite),
        ("contextra_get", ToolCategory::DatabaseRead),
        ("contextra_forget", ToolCategory::DatabaseWrite),
        ("contextra_collections", ToolCategory::DatabaseRead),
        ("contextra_plugin_status", ToolCategory::DatabaseRead),
        ("contextra_upsert", ToolCategory::DatabaseWrite),
        ("contextra_delete", ToolCategory::DatabaseWrite),
        ("contextra_create_collection", ToolCategory::DatabaseWrite),
        ("contextra_drop_collection", ToolCategory::DatabaseWrite),
    ];

    let (server, _tmp) = create_mock_server().await;
    let req = make_request("tools/list", json!({}));
    let response = server.handle(req).await;
    assert_eq!(response.jsonrpc, "2.0");

    let tools = response.result.expect("result expected")["tools"]
        .as_array()
        .expect("tools array expected")
        .clone();

    assert_eq!(
        tools.len(),
        EXPECTED_TOOLS.len(),
        "Mismatch between tools/list count ({}) and EXPECTED_TOOLS count ({}).",
        tools.len(),
        EXPECTED_TOOLS.len()
    );

    let expected_map: std::collections::HashMap<&str, ToolCategory> =
        EXPECTED_TOOLS.iter().cloned().collect();

    for tool in &tools {
        let name = tool["name"].as_str().expect("tool name must be a string");

        let try_cat = crate::sandbox::McpSandbox::try_classify_method(name).unwrap_or_else(|_| {
            panic!("Tool '{name}' from tools/list was not found in TOOL_REGISTRY")
        });

        assert_ne!(
            try_cat,
            ToolCategory::CodeExecution,
            "Tool '{name}' must not rely on CodeExecution fallback"
        );

        let expected_cat = expected_map
            .get(name)
            .unwrap_or_else(|| panic!("Tool '{name}' missing from EXPECTED_TOOLS list"));

        assert_eq!(
            &try_cat, expected_cat,
            "Tool '{name}' category mismatch: expected {:?}, got {:?}",
            expected_cat, try_cat
        );
    }
}

#[tokio::test]
async fn test_contextra_forget_single_doc() {
    let (server, _tmp) = create_mock_server_with_write(true).await;

    // 1. Insert document
    let insert_req = make_request(
        "contextra_insert",
        json!({
            "id": "forget_doc_1",
            "text": "Secret data to forget",
            "collection": "default"
        }),
    );
    let resp = server.handle(insert_req).await;
    assert!(resp.error.is_none());

    // 2. Forget single doc without confirm=true should fail
    let forget_req_no_confirm = make_request(
        "tools/call",
        json!({
            "name": "contextra_forget",
            "arguments": {
                "collection": "default",
                "id": "forget_doc_1"
            }
        }),
    );
    let resp_nc = server.handle(forget_req_no_confirm).await;
    let res_nc = serde_json::to_value(&resp_nc).unwrap();
    assert_eq!(res_nc["result"]["isError"], true);

    // 3. Forget single doc with confirm=true
    let forget_req = make_request(
        "tools/call",
        json!({
            "name": "contextra_forget",
            "arguments": {
                "collection": "default",
                "id": "forget_doc_1",
                "confirm": true
            }
        }),
    );
    let resp_f = server.handle(forget_req).await;
    assert!(resp_f.error.is_none());
    let res_f = serde_json::to_value(&resp_f).unwrap();
    assert_ne!(res_f["result"]["isError"], true);
    let text = res_f["result"]["content"][0]["text"].as_str().unwrap();
    let json_res: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(json_res["status"], "logically_deleted");
    assert!(json_res.get("tx").is_some());
    assert!(json_res.get("proof").is_none());

    // 4. Verify contextra_get returns null
    let get_req = make_request(
        "contextra_get",
        json!({
            "id": "forget_doc_1",
            "collection": "default"
        }),
    );
    let resp_g = server.handle(get_req).await;
    assert_eq!(resp_g.result.unwrap(), serde_json::Value::Null);
}

#[tokio::test]
async fn test_contextra_forget_options_forwarding_and_unsupported_capability() {
    let (server, _tmp) = create_mock_server_with_write(true).await;

    // Insert document
    let insert_req = make_request(
        "contextra_insert",
        json!({
            "id": "forget_doc_opts",
            "text": "Data with options",
            "collection": "default"
        }),
    );
    let resp = server.handle(insert_req).await;
    assert!(resp.error.is_none());

    // 1. Passing an unsupported option key triggers CapabilityUnsupported
    let forget_unsupported_opt = make_request(
        "tools/call",
        json!({
            "name": "contextra_forget",
            "arguments": {
                "collection": "default",
                "id": "forget_doc_opts",
                "confirm": true,
                "options": {
                    "unsupported_custom_flag": true
                }
            }
        }),
    );
    let resp_unsup = server.handle(forget_unsupported_opt).await;
    let res_unsup = serde_json::to_value(&resp_unsup).unwrap();
    assert_eq!(res_unsup["result"]["isError"], true);

    // 2. Forget single doc with options requiring assurance S3 (triggers CapabilityUnsupported in erase_document)
    let forget_req_opts = make_request(
        "tools/call",
        json!({
            "name": "contextra_forget",
            "arguments": {
                "collection": "default",
                "id": "forget_doc_opts",
                "confirm": true,
                "options": {
                    "assurance": "S3"
                }
            }
        }),
    );
    let resp_f = server.handle(forget_req_opts).await;
    let res_f = serde_json::to_value(&resp_f).unwrap();
    assert_eq!(res_f["result"]["isError"], true);
    let text = res_f["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("real_scan_erasure") || text.contains("Assurance levels above S0 require real storage scans"),
        "Expected CapabilityUnsupported error when assurance option S3 is passed, got: '{text}'"
    );
}

#[tokio::test]
async fn test_malformed_json_returns_parse_error() {
    let parse_res = serde_json::from_str::<Value>("{ invalid }");
    assert!(parse_res.is_err());
    let err_resp = JsonRpcResponse::err(None, -32700, "Parse error");
    assert_eq!(err_resp.jsonrpc, "2.0");
    assert_eq!(err_resp.error.unwrap().code, -32700);
}

#[tokio::test]
async fn test_invalid_rpc_request_returns_invalid_request_code() {
    let invalid_rpc = json!({
        "id": 1,
        "method": "initialize"
        // "jsonrpc": "2.0" is missing
    });
    let req_res = serde_json::from_value::<JsonRpcRequest>(invalid_rpc);
    assert!(req_res.is_err());
    let err_resp = JsonRpcResponse::err(Some(json!(1)), -32600, "Invalid Request");
    assert_eq!(err_resp.jsonrpc, "2.0");
    assert_eq!(err_resp.error.unwrap().code, -32600);
}

#[tokio::test]
async fn test_request_id_echo_roundtrip() {
    let (server, _tmp) = create_mock_server().await;

    // String ID
    let req1 = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!("abc-123")),
        method: "ping".into(),
        params: json!({}),
    };
    let resp1 = server.handle(req1).await;
    assert_eq!(resp1.id, Some(json!("abc-123")));
    assert_eq!(resp1.jsonrpc, "2.0");

    // Numeric ID
    let req2 = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(42)),
        method: "ping".into(),
        params: json!({}),
    };
    let resp2 = server.handle(req2).await;
    assert_eq!(resp2.id, Some(json!(42)));
    assert_eq!(resp2.jsonrpc, "2.0");
}

#[tokio::test]
async fn test_unknown_method_returns_method_not_found() {
    let (server, _tmp) = create_mock_server().await;
    let req = make_request("nonexistent/method", json!({}));
    let response = server.handle(req).await;
    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, Some(json!(1)));
    let err = response.error.expect("error object expected");
    assert_eq!(err.code, -32601);
}

#[tokio::test]
async fn test_missing_required_param_returns_invalid_params_32602() {
    let (server, _tmp) = create_mock_server().await;
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(101)),
        method: "contextra_insert".into(),
        params: json!({
            // missing required "id" field
            "collection": "default",
            "text": "some text"
        }),
    };
    let response = server.handle(req).await;
    assert_eq!(response.id, Some(json!(101)));
    let err = response
        .error
        .expect("error expected for missing required param");
    assert_eq!(err.code, -32602);
    assert!(err.message.contains("id"));
}

#[tokio::test]
async fn test_internal_error_returns_32603() {
    use crate::protocol::{response_from_error, McpError};
    let err = McpError::internal_error("storage layer failure");
    assert_eq!(err.code(), -32603);
    let resp = response_from_error(Some(json!(102)), err);
    assert_eq!(resp.id, Some(json!(102)));
    let err_obj = resp.error.expect("error expected");
    assert_eq!(err_obj.code, -32603);
    assert_eq!(err_obj.message, "storage layer failure");
}

#[tokio::test]
async fn test_notification_expects_no_response() {
    let (server, _tmp) = create_mock_server().await;
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: None,
        method: "initialized".into(),
        params: json!({}),
    };
    assert!(req.id.is_none());
    let response = server.handle(req).await;
    assert_eq!(response.id, None);
}

#[tokio::test]
async fn test_read_line_bounded_enforces_limit() {
    use crate::{read_line_bounded, MAX_RPC_BYTES};
    use std::io::Cursor;
    use tokio::io::BufReader;

    assert_eq!(MAX_RPC_BYTES, 4 * 1024 * 1024);

    // 1. Normal line within limit
    let data = "{\"jsonrpc\":\"2.0\",\"id\":1}\n";
    let mut reader = BufReader::new(Cursor::new(data));
    let mut buf = String::new();
    let res = read_line_bounded(&mut reader, &mut buf, MAX_RPC_BYTES).await;
    assert!(res.is_ok());
    assert_eq!(buf, data);

    // 2. Line exceeding limit (e.g. 100 bytes when limit is 50), followed by valid line
    let oversized = "A".repeat(100) + "\n" + "{\"jsonrpc\":\"2.0\",\"id\":2}\n";
    let mut oversized_reader = BufReader::new(Cursor::new(oversized));
    let mut buf2 = String::new();
    let res_err = read_line_bounded(&mut oversized_reader, &mut buf2, 50).await;
    assert!(res_err.is_err());
    let err = res_err.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("limit exceeded"));

    // 3. Verify line-draining: next call reads the second valid line
    let mut buf3 = String::new();
    let res_ok = read_line_bounded(&mut oversized_reader, &mut buf3, 50).await;
    assert!(res_ok.is_ok());
    assert_eq!(buf3, "{\"jsonrpc\":\"2.0\",\"id\":2}\n");
}

#[tokio::test]
async fn test_stdout_not_polluted_by_logs() {
    let source = std::fs::read_to_string("src/lib.rs")
        .or_else(|_| std::fs::read_to_string("crates/contextra-mcp/src/lib.rs"))
        .expect("read lib.rs");
    let bin_source = std::fs::read_to_string("src/bin/contextra-mcp-server.rs")
        .or_else(|_| {
            std::fs::read_to_string("crates/contextra-mcp/src/bin/contextra-mcp-server.rs")
        })
        .expect("read bin");

    let stdout_writes = source
        .lines()
        .chain(bin_source.lines())
        .filter(|line| !line.trim().starts_with("//"))
        .filter(|line| line.contains("println!") || line.contains("print!"))
        .count();

    assert_eq!(
        stdout_writes, 0,
        "No println! or print! allowed in contextra-mcp — use stderr/tracing"
    );
}

#[tokio::test]
async fn test_search_validates_empty_or_oversized_query() {
    let (server, _tmp) = create_mock_server().await;

    // Whitespace query
    let req_whitespace = make_request("contextra_search", json!({"query": "   "}));
    let resp = server.handle(req_whitespace).await;
    let err = resp.error.expect("error expected for empty query");
    assert_eq!(err.code, -32602);
    assert!(err.message.contains("query cannot be empty"));

    // Oversized query
    let huge_query = "x".repeat(crate::MAX_SEARCH_QUERY_BYTES + 1);
    let req_huge = make_request("contextra_search", json!({"query": huge_query}));
    let resp_huge = server.handle(req_huge).await;
    let err_huge = resp_huge.error.expect("error expected for oversized query");
    assert_eq!(err_huge.code, -32602);
    assert!(err_huge.message.contains("query size exceeds limit"));
}

#[tokio::test]
async fn test_write_tool_rejected_when_read_only() {
    let (server, _tmp) = create_mock_server_with_write(false).await;

    let write_tools = [
        "contextra_insert",
        "contextra_delete",
        "contextra_upsert",
        "contextra_create_collection",
        "contextra_drop_collection",
    ];

    for tool in write_tools {
        let req = make_request(
            "tools/call",
            json!({
                "name": tool,
                "arguments": {
                    "id": "test_id",
                    "text": "test_text"
                }
            }),
        );
        let resp = server.handle(req).await;
        let res_val = serde_json::to_value(&resp).unwrap();
        assert_eq!(res_val["result"]["isError"], true);
        let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
        assert!(
            text.contains("Sandbox: DB-Schreibzugriff gesperrt"),
            "Expected write rejection for '{tool}', got: '{text}'"
        );
    }
}

#[tokio::test]
async fn test_write_tool_allowed_when_explicitly_enabled() {
    let (server, _tmp) = create_mock_server_with_write(true).await;

    let req = make_request(
        "tools/call",
        json!({
            "name": "contextra_insert",
            "arguments": {
                "id": "write_enabled_doc",
                "text": "Write enabled content"
            }
        }),
    );
    let resp = server.handle(req).await;
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_ne!(res_val["result"]["isError"], true);
    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("write_enabled_doc"));
}

#[tokio::test]
async fn test_read_tools_always_allowed_regardless_of_flag() {
    let (server_ro, _tmp1) = create_mock_server_with_write(false).await;
    let (server_rw, _tmp2) = create_mock_server_with_write(true).await;

    let read_req = make_request(
        "tools/call",
        json!({
            "name": "contextra_collections",
            "arguments": {}
        }),
    );

    let resp_ro = server_ro.handle(read_req.clone()).await;
    let res_ro = serde_json::to_value(&resp_ro).unwrap();
    assert_ne!(res_ro["result"]["isError"], true);

    let resp_rw = server_rw.handle(read_req).await;
    let res_rw = serde_json::to_value(&resp_rw).unwrap();
    assert_ne!(res_rw["result"]["isError"], true);
}
