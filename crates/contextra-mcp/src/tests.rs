#[cfg(test)]
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::McpServer;
use contextra::Contextra;
use contextra_core::BoxFuture;
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Debug)]
struct MockEmbedder {
    dimension: usize,
}

impl contextra_core::EmbeddingProvider for MockEmbedder {
    fn provider_name(&self) -> &str {
        "mock"
    }

    fn embed<'a>(
        &'a self,
        _text: &'a str,
    ) -> BoxFuture<'a, std::result::Result<Vec<f32>, contextra_core::EmbeddingError>> {
        Box::pin(async move { Ok(vec![0.1f32; self.dimension]) })
    }

    fn embedding_dim(&self) -> usize {
        self.dimension
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, std::result::Result<Vec<Vec<f32>>, contextra_core::EmbeddingError>> {
        Box::pin(async move { Ok(vec![vec![0.1f32; self.dimension]; texts.len()]) })
    }
}

async fn create_mock_server() -> (Arc<McpServer>, TempDir) {
    create_mock_server_with_write_and_egress(true, true).await
}

async fn create_mock_server_with_write(allow_db_writes: bool) -> (Arc<McpServer>, TempDir) {
    create_mock_server_with_write_and_egress(allow_db_writes, false).await
}

async fn create_mock_server_with_write_and_egress(
    allow_db_writes: bool,
    allow_cloud_egress: bool,
) -> (Arc<McpServer>, TempDir) {
    use crate::sandbox::{McpSandbox, SandboxPolicy};
    let tmp = TempDir::new().expect("temp dir"); // expect
    let db = Contextra::open(tmp.path()).await.expect("open db"); // expect
    let collection = db.collection("default").await.expect("collection"); // expect
    let dim = collection.dimension();
    let embedder = Arc::new(MockEmbedder { dimension: dim });
    let policy = SandboxPolicy {
        allow_db_reads: true,
        allow_db_writes,
        allow_code_execution: false,
        allow_cloud_egress,
        max_execution_ms: 5_000,
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

#[tokio::test]
async fn test_tools_list_returns_all_tools() {
    let (server, _tmp) = create_mock_server().await;
    let req = make_request("tools/list", json!({}));
    let response = server.handle(req).await;
    assert_eq!(response.jsonrpc, "2.0");
    let tools = response.result.unwrap()["tools"] // unwrap
        .as_array()
        .unwrap() // unwrap
        .clone();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert_eq!(tools.len(), 7);
    assert!(names.contains(&"contextra_search"));
    assert!(names.contains(&"contextra_insert"));
    assert!(names.contains(&"contextra_get"));
    assert!(names.contains(&"contextra_forget"));
    assert!(names.contains(&"contextra_collections"));
    assert!(names.contains(&"contextra_consolidate"));
    assert!(names.contains(&"contextra_cloud_query"));
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
    assert_eq!(json_res["ok"], true);
    assert_eq!(json_res["id"], "forget_doc_1");
    assert_eq!(json_res["proof"], serde_json::Value::Null);
    assert_eq!(json_res["proof_scope"], "collection_only");

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
async fn test_contextra_forget_collection_with_proof() {
    let (server, _tmp) = create_mock_server_with_write(true).await;

    // Create custom collection and insert a document
    let col_name = "drop_me_col";
    let insert_req = make_request(
        "contextra_insert",
        json!({
            "id": "doc_in_col",
            "text": "Data in custom collection",
            "collection": col_name
        }),
    );
    let resp = server.handle(insert_req).await;
    assert!(resp.error.is_none());

    // 1. Without proof key configured -> fails
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
    std::env::remove_var("CONTEXTRA_PROOF_KEY");
    let forget_req_nokey = make_request(
        "tools/call",
        json!({
            "name": "contextra_forget",
            "arguments": {
                "collection": col_name,
                "confirm": true
            }
        }),
    );
    let resp_nokey = server.handle(forget_req_nokey).await;
    let res_nokey = serde_json::to_value(&resp_nokey).unwrap();
    assert_eq!(res_nokey["result"]["isError"], true);
    let err_text = res_nokey["result"]["content"][0]["text"].as_str().unwrap();
    assert!(err_text.contains("deletion proof key not configured"));

    // 2. With proof key configured -> succeeds and generates DeletionProof
    let test_key = "test_deletion_proof_key_32_bytes!";
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", test_key);

    let forget_req = make_request(
        "tools/call",
        json!({
            "name": "contextra_forget",
            "arguments": {
                "collection": col_name,
                "confirm": true
            }
        }),
    );
    let resp_drop = server.handle(forget_req).await;
    assert!(resp_drop.error.is_none());
    let res_drop = serde_json::to_value(&resp_drop).unwrap();
    assert_ne!(res_drop["result"]["isError"], true);

    let text = res_drop["result"]["content"][0]["text"].as_str().unwrap();
    let json_res: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(json_res["ok"], true);
    assert_eq!(json_res["collection"], col_name);
    assert_eq!(json_res["proof_scope"], "collection");

    let proof_val = &json_res["proof"];
    let proof: contextra_crypto::deletion_proof::DeletionProof =
        serde_json::from_value(proof_val.clone()).unwrap();
    assert!(proof.verify(test_key.as_bytes()).unwrap());

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[tokio::test]
async fn test_malformed_json_returns_parse_error() {
    let parse_res = serde_json::from_str::<Value>("{ invalid }");
    assert!(parse_res.is_err());
    let err_resp = JsonRpcResponse::err(None, -32700, "Parse error");
    assert_eq!(err_resp.jsonrpc, "2.0");
    assert_eq!(err_resp.error.unwrap().code, -32700); // unwrap
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
    assert_eq!(err_resp.error.unwrap().code, -32600); // unwrap
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
    let err = response.error.expect("error object expected"); // expect
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
        .expect("error expected for missing required param"); // expect
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
    let err_obj = resp.error.expect("error expected"); // expect
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
async fn test_read_line_bounded_idle_timeout_soft_reset() {
    use crate::read_line_bounded;
    use tokio::io::{AsyncWriteExt, BufReader};
    use tokio::time::{timeout, Duration};

    let (mut client_tx, server_rx) = tokio::io::duplex(1024);
    let mut reader = BufReader::new(server_rx);
    let mut line_buf = String::new();

    // 1. Reader times out when client is idle (soft timeout simulation)
    let timed_out = timeout(
        Duration::from_millis(10),
        read_line_bounded(&mut reader, &mut line_buf, 1024),
    )
    .await;
    assert!(
        timed_out.is_err(),
        "Expected timeout error when reader receives no input"
    );

    // 2. Client sends valid line after idle period
    client_tx
        .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n")
        .await
        .unwrap();

    // 3. Reader resumes normally on next iteration without connection termination
    let res = timeout(
        Duration::from_millis(500),
        read_line_bounded(&mut reader, &mut line_buf, 1024),
    )
    .await;
    assert!(res.is_ok(), "Expected successful read after soft reset");
    let len = res.unwrap().unwrap();
    assert!(len > 0);
    assert_eq!(
        line_buf.trim(),
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}"
    );
}

#[tokio::test]
async fn test_stdout_not_polluted_by_logs() {
    let source = std::fs::read_to_string("src/lib.rs")
        .or_else(|_| std::fs::read_to_string("crates/contextra-mcp/src/lib.rs"))
        .expect("read lib.rs"); // expect
    let bin_source = std::fs::read_to_string("src/bin/contextra-mcp-server.rs")
        .or_else(|_| std::fs::read_to_string("crates/contextra-mcp/src/bin/contextra-mcp-server.rs"))
        .expect("read bin"); // expect

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
    let err = resp.error.expect("error expected for empty query"); // expect
    assert_eq!(err.code, -32602);
    assert!(err.message.contains("query cannot be empty"));

    // Oversized query
    let huge_query = "x".repeat(crate::MAX_SEARCH_QUERY_BYTES + 1);
    let req_huge = make_request("contextra_search", json!({"query": huge_query}));
    let resp_huge = server.handle(req_huge).await;
    let err_huge = resp_huge.error.expect("error expected for oversized query"); // expect
    assert_eq!(err_huge.code, -32602);
    assert!(err_huge.message.contains("query size exceeds limit"));
}

#[tokio::test]
async fn test_insert_validates_vector_nan_inf_and_empty() {
    let (server, _tmp) = create_mock_server().await;

    // Empty vector
    let req_empty_vec = make_request(
        "contextra_insert",
        json!({
            "id": "doc1",
            "vector": []
        }),
    );
    let resp = server.handle(req_empty_vec).await;
    let err = resp.error.expect("error expected for empty vector"); // expect
    assert_eq!(err.code, -32602);
    assert!(err.message.contains("vector cannot be empty"));

    // Vector with float value that overflows f32 (e.g. 1.0e39)
    let req_inf = make_request(
        "contextra_insert",
        json!({
            "id": "doc2",
            "vector": [0.1, 1.0e39]
        }),
    );
    let resp_inf = server.handle(req_inf).await;
    let err_inf = resp_inf.error.expect("error expected for Inf in vector"); // expect
    assert_eq!(err_inf.code, -32602);
    assert!(err_inf.message.contains("NaN or Inf"));
}

#[tokio::test]
async fn test_insert_validates_oversized_id() {
    let (server, _tmp) = create_mock_server().await;

    let long_id = "i".repeat(257);
    let req = make_request(
        "contextra_insert",
        json!({
            "id": long_id,
            "text": "hello world"
        }),
    );
    let resp = server.handle(req).await;
    let err = resp.error.expect("error expected for long ID"); // expect
    assert_eq!(err.code, -32602);
    assert!(err.message.contains("id length exceeds limit"));
}

#[tokio::test]
async fn test_get_validates_oversized_id() {
    let (server, _tmp) = create_mock_server().await;

    let long_id = "g".repeat(257);
    let req = make_request(
        "contextra_get",
        json!({
            "id": long_id,
            "collection": "default"
        }),
    );
    let resp = server.handle(req).await;
    let err = resp
        .error
        .expect("error expected for long ID in contextra_get"); // expect
    assert_eq!(err.code, -32602);
    assert!(err.message.contains("id length exceeds limit"));
}

#[tokio::test]
async fn test_whitespace_collection_name_fallback_or_rejection() {
    let (server, _tmp) = create_mock_server().await;

    // "   " as collection defaults to "default"
    let req = make_request(
        "contextra_search",
        json!({
            "query": "find me",
            "collection": "   "
        }),
    );
    let resp = server.handle(req).await;
    assert!(resp.result.is_some());
}

#[tokio::test]
async fn test_write_tool_rejected_when_read_only() {
    let (server, _tmp) = create_mock_server_with_write(false).await;

    let write_tools = [
        "contextra_insert",
        "contextra_delete",
        "contextra_upsert",
        "contextra_relate",
        "contextra_create_collection",
        "contextra_drop_collection",
        "contextra_consolidate",
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
        let res_val = serde_json::to_value(&resp).unwrap(); // unwrap
        assert_eq!(res_val["result"]["isError"], true);
        let text = res_val["result"]["content"][0]["text"].as_str().unwrap(); // unwrap
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
    let res_val = serde_json::to_value(&resp).unwrap(); // unwrap
    assert_ne!(res_val["result"]["isError"], true);
    let text = res_val["result"]["content"][0]["text"].as_str().unwrap(); // unwrap
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
    let res_ro = serde_json::to_value(&resp_ro).unwrap(); // unwrap
    assert_ne!(res_ro["result"]["isError"], true);

    let resp_rw = server_rw.handle(read_req).await;
    let res_rw = serde_json::to_value(&resp_rw).unwrap(); // unwrap
    assert_ne!(res_rw["result"]["isError"], true);
}

#[tokio::test]
async fn test_contextra_search_executes_query_builder_successfully() {
    let (server, _tmp) = create_mock_server().await;

    // First insert a test document
    let insert_req = make_request(
        "contextra_insert",
        json!({
            "id": "doc_search_test",
            "text": "hybrid query builder search content"
        }),
    );
    let insert_resp = server.handle(insert_req).await;
    assert!(insert_resp.error.is_none());

    // Search using contextra_search tool (triggers col.query().text().vector().k().execute())
    let search_req = make_request(
        "contextra_search",
        json!({
            "query": "hybrid query builder",
            "k": 5
        }),
    );
    let search_resp = server.handle(search_req).await;
    assert!(search_resp.error.is_none());
    if let Some(res_vec) = search_resp.result {
        if let Some(arr) = res_vec.as_array() {
            assert!(!arr.is_empty(), "expected search results");
            assert_eq!(arr[0]["content_provenance"], "retrieved_untrusted_data");
        } else {
            panic!("result must be array");
        }
    } else {
        panic!("search result expected");
    }
}

#[tokio::test]
async fn test_protocol_request_deserialization_no_panic() {
    let test_inputs = [
        "",
        "{}",
        "{\"jsonrpc\":\"2.0\"}",
        "{\"jsonrpc\":\"2.0\",\"method\":\"ping\"}",
        "{\"jsonrpc\":\"1.0\",\"method\":\"ping\"}",
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":123}",
        "{\"jsonrpc\":\"2.0\",\"id\":\"abc\",\"method\":\"test\",\"params\":null}",
        "{\"jsonrpc\":\"2.0\",\"id\":[1,2,3],\"method\":\"test\"}",
        "\0\r\n\t",
        "{\"method\":\"tool\",\"params\":{\"a\":\"\u{0000}\"}}",
    ];

    for input in test_inputs {
        let _ = serde_json::from_str::<JsonRpcRequest>(input);
    }
}

#[tokio::test]
async fn test_protocol_response_serialization_no_panic() {
    let test_cases = [
        (Some(json!(1)), -32600, "Invalid Request"),
        (Some(json!("str_id")), -32601, "Method not found"),
        (Some(json!(null)), -32700, "Parse error"),
        (None, -32603, "Internal error"),
    ];

    for (id, code, msg) in test_cases {
        let err_resp = JsonRpcResponse::err(id, code, msg);
        let ser = serde_json::to_string(&err_resp);
        assert!(ser.is_ok());
    }
}

#[tokio::test]
async fn test_batch_request_handling() {
    let (server, _tmp) = create_mock_server().await;

    // 1. Batch with multiple valid requests
    let batch_val = json!([
        { "jsonrpc": "2.0", "id": 1, "method": "ping", "params": {} },
        { "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} }
    ]);
    if let Some(resp) = server.handle_value(batch_val).await {
        if let Some(arr) = resp.as_array() {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0]["id"], 1);
            assert_eq!(arr[1]["id"], 2);
        } else {
            panic!("array expected");
        }
    } else {
        panic!("batch response expected");
    }

    // 2. Empty batch array -> Invalid Request (-32600)
    let empty_batch = json!([]);
    if let Some(empty_resp) = server.handle_value(empty_batch).await {
        assert_eq!(empty_resp["error"]["code"], -32600);
    } else {
        panic!("empty batch response expected");
    }

    // 3. Batch notifications only -> None (no response)
    let notif_batch = json!([
        { "jsonrpc": "2.0", "method": "initialized", "params": {} },
        { "jsonrpc": "2.0", "method": "initialized", "params": {} }
    ]);
    let notif_resp = server.handle_value(notif_batch).await;
    assert!(notif_resp.is_none());

    // 4. Mixed batch (requests + notifications) -> returns array containing only non-notification responses
    let mixed_batch = json!([
        { "jsonrpc": "2.0", "id": 10, "method": "ping", "params": {} },
        { "jsonrpc": "2.0", "method": "initialized", "params": {} },
        { "jsonrpc": "2.0", "id": 11, "method": "nonexistent_method", "params": {} }
    ]);
    if let Some(mixed_resp) = server.handle_value(mixed_batch).await {
        if let Some(mixed_arr) = mixed_resp.as_array() {
            assert_eq!(mixed_arr.len(), 2);
            assert_eq!(mixed_arr[0]["id"], 10);
            assert_eq!(mixed_arr[1]["id"], 11);
            assert_eq!(mixed_arr[1]["error"]["code"], -32601);
        } else {
            panic!("array expected");
        }
    } else {
        panic!("mixed response expected");
    }
}

#[tokio::test]
async fn test_contextra_consolidate_success() {
    let (server, _tmp) = create_mock_server_with_write(true).await;

    // Insert 2 documents into default collection
    let insert_req1 = make_request(
        "contextra_insert",
        json!({
            "id": "turn_doc_1",
            "text": "First turn text",
            "collection": "default"
        }),
    );
    let resp1 = server.handle(insert_req1).await;
    assert!(resp1.error.is_none());

    let insert_req2 = make_request(
        "contextra_insert",
        json!({
            "id": "turn_doc_2",
            "text": "Second turn text",
            "collection": "default"
        }),
    );
    let resp2 = server.handle(insert_req2).await;
    assert!(resp2.error.is_none());

    // Trigger manual consolidation via MCP
    let consolidate_req = make_request(
        "tools/call",
        json!({
            "name": "contextra_consolidate",
            "arguments": {
                "collection": "default"
            }
        }),
    );

    let resp = server.handle(consolidate_req).await;
    assert!(resp.error.is_none());

    let res_val = serde_json::to_value(&resp).unwrap();
    assert_ne!(res_val["result"]["isError"], true);

    let text_out = res_val["result"]["content"][0]["text"].as_str().unwrap();
    let json_out: serde_json::Value = serde_json::from_str(text_out).unwrap();

    assert_eq!(json_out["ok"], true);
    assert_eq!(json_out["collection"], "default");
    assert!(json_out["turns_scanned"].as_u64().unwrap() >= 2);
    assert!(json_out.get("segments_created").is_some());
    assert!(json_out.get("duplicates_tombstoned").is_some());
    assert!(json_out.get("synthesized_chunks").is_some());
}

#[tokio::test]
async fn test_contextra_consolidate_read_only_rejected() {
    let (server, _tmp) = create_mock_server_with_write(false).await;

    let req = make_request(
        "tools/call",
        json!({
            "name": "contextra_consolidate",
            "arguments": {
                "collection": "default"
            }
        }),
    );

    let resp = server.handle(req).await;
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_eq!(res_val["result"]["isError"], true);
    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Sandbox: DB-Schreibzugriff gesperrt"));
}

#[tokio::test]
async fn test_contextra_consolidate_unknown_collection() {
    let (server, _tmp) = create_mock_server_with_write(true).await;

    let req = make_request(
        "tools/call",
        json!({
            "name": "contextra_consolidate",
            "arguments": {
                "collection": "invalid/collection:name"
            }
        }),
    );

    let resp = server.handle(req).await;
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_eq!(res_val["result"]["isError"], true);
    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("forbidden characters"));
}

#[tokio::test]
async fn test_contextra_consolidate_fault_injection() {
    let (server, _tmp) = create_mock_server_with_write(true).await;

    // Send malformed arguments (collection as integer instead of string)
    let req_wrong_type = make_request(
        "tools/call",
        json!({
            "name": "contextra_consolidate",
            "arguments": {
                "collection": 12345
            }
        }),
    );

    let resp = server.handle(req_wrong_type).await;
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_eq!(res_val["result"]["isError"], true);
    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("collection' must be a string"));

    // Direct RPC invocation with invalid params
    let req_direct = make_request("contextra_consolidate", json!({ "collection": 9999 }));
    let direct_resp = server.handle(req_direct).await;
    let err = direct_resp
        .error
        .expect("error expected for direct invalid call");
    assert_eq!(err.code, -32602);
}

#[cfg(feature = "kv-bridge")]
#[tokio::test]
async fn test_kv_bridge_adapter_consulted_on_retrieve() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let collection = db.collection("default").await.expect("collection");
    let dim = collection.dimension();
    let embedder = Arc::new(MockEmbedder { dimension: dim });

    let store = Arc::new(contextra_crypto::TenantIsolatedKvStore::new());
    let master_km = contextra_crypto::CryptoKey::try_new("test-mcp-kv", b"test-salt-mcp-kv").unwrap();
    let cipher = Arc::new(contextra_crypto::KvSegmentCipher::new(master_km));
    let bridge_adapter = Arc::new(contextra_infer_candle::KvBridgeAdapter::new(store, cipher));

    let server = Arc::new(
        McpServer::with_write_permission(Arc::new(db), embedder, true)
            .expect("server new")
            .with_kv_bridge(Some(bridge_adapter)),
    );

    // Insert a document first so search returns a result
    let insert_req = make_request(
        "contextra_insert",
        json!({
            "id": "doc_kv_test",
            "text": "kv bridge consultation test content"
        }),
    );
    let insert_resp = server.handle(insert_req).await;
    assert!(insert_resp.error.is_none());

    // Execute contextra_search tool call
    let search_req = make_request(
        "contextra_search",
        json!({
            "query": "kv bridge consultation",
            "k": 5
        }),
    );
    let search_resp = server.handle(search_req).await;
    assert!(search_resp.error.is_none());

    let bridge = server
        .kv_bridge
        .as_ref()
        .expect("kv_bridge must be attached for this test");
    assert!(
        bridge.consultation_count() > 0,
        "KvBridgeAdapter must be consulted during contextra_search execution"
    );
}

#[tokio::test]
async fn test_cloud_query_sensitive_input_blocked_by_egress_classifier() {
    let (server, _tmp) = create_mock_server().await;

    // 1. Test simulated email address payload
    let req_email = make_request(
        "tools/call",
        json!({
            "name": "contextra_cloud_query",
            "arguments": {
                "query": "search user data for alice@example.com"
            }
        }),
    );
    let resp_email = server.handle(req_email).await;
    let res_val1 = serde_json::to_value(&resp_email).unwrap();
    assert_eq!(res_val1["result"]["isError"], true);
    let err_msg1 = res_val1["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        err_msg1.contains("Egress policy violation: query blocked by rule R-005"),
        "Expected opaque rule ID in block message for email input, got: '{err_msg1}'"
    );
    assert!(
        !err_msg1.contains("@"),
        "Error message must not leak raw regex pattern"
    );

    // 2. Test simulated API key payload (e.g. sk-...)
    let req_apikey = make_request(
        "tools/call",
        json!({
            "name": "contextra_cloud_query",
            "arguments": {
                "query": "query API using secret sk-abc123456789"
            }
        }),
    );
    let resp_apikey = server.handle(req_apikey).await;
    let res_val2 = serde_json::to_value(&resp_apikey).unwrap();
    assert_eq!(res_val2["result"]["isError"], true);
    let err_msg2 = res_val2["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        err_msg2.contains("Egress policy violation: query blocked by rule R-001"),
        "Expected opaque rule ID in block message for API key input, got: '{err_msg2}'"
    );

    // 3. Test sensitive keyword pattern (api_key, password)
    let req_password = make_request(
        "tools/call",
        json!({
            "name": "contextra_cloud_query",
            "arguments": {
                "query": "retrieve my password for admin account"
            }
        }),
    );
    let resp_password = server.handle(req_password).await;
    let res_val3 = serde_json::to_value(&resp_password).unwrap();
    assert_eq!(res_val3["result"]["isError"], true);
    let err_msg3 = res_val3["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        err_msg3.contains("Egress policy violation: query blocked by rule R-004"),
        "Expected opaque rule ID in block message for password input, got: '{err_msg3}'"
    );
}

#[tokio::test]
async fn test_cloud_query_allow_returns_success() {
    let (server, _tmp) = create_mock_server().await;

    let req = make_request(
        "tools/call",
        json!({
            "name": "contextra_cloud_query",
            "arguments": {
                "query": "safe public query for documentation"
            }
        }),
    );

    let resp = server.handle(req).await;
    assert!(resp.error.is_none());
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_ne!(res_val["result"]["isError"], true);

    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    let json_res: serde_json::Value = serde_json::from_str(text).unwrap();

    assert_eq!(json_res["status"], "success");
    assert_eq!(json_res["abstracted"], false);
    assert_eq!(json_res["query"], "safe public query for documentation");
}

#[tokio::test]
async fn test_removed_substring_early_branch_runs_through_regex_vault() {
    let (server, _tmp) = create_mock_server().await;

    // Previously, queries containing "abstract" or "PII" were intercepted by a substring early branch.
    // Now they must run through DefaultEgressClassifier's EgressVault patterns.
    let req = make_request(
        "tools/call",
        json!({
            "name": "contextra_cloud_query",
            "arguments": {
                "query": "safe query mentioning abstract concepts and PII terminology"
            }
        }),
    );

    let resp = server.handle(req).await;
    assert!(resp.error.is_none());
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_ne!(res_val["result"]["isError"], true);

    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    let json_res: serde_json::Value = serde_json::from_str(text).unwrap();

    // Since it contains no sensitive regex patterns (sk-, AKIA, password, email, etc.), it returns Allow (status: success, abstracted: false).
    assert_eq!(json_res["status"], "success");
    assert_eq!(json_res["abstracted"], false);
    assert_eq!(
        json_res["query"],
        "safe query mentioning abstract concepts and PII terminology"
    );
}

#[tokio::test]
async fn test_cloud_query_secret_with_abstract_word_is_blocked_by_vault() {
    let (server, _tmp) = create_mock_server().await;

    // Payload containing the word "abstract" PLUS a sensitive key pattern "sk-abc123456789"
    let req = make_request(
        "tools/call",
        json!({
            "name": "contextra_cloud_query",
            "arguments": {
                "query": "please abstract this secret key sk-abc123456789 for me"
            }
        }),
    );

    let resp = server.handle(req).await;
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_eq!(res_val["result"]["isError"], true);

    let err_msg = res_val["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        err_msg.contains("Egress policy violation: query blocked by rule R-001"),
        "Expected opaque rule ID R-001 in block message, got: '{err_msg}'"
    );
    assert!(
        !err_msg.contains("sk-"),
        "Error message must not leak raw pattern, got: '{err_msg}'"
    );
}

#[tokio::test]
async fn test_cloud_query_at_sign_without_email_is_allowed() {
    let (server, _tmp) = create_mock_server().await;

    // Text containing '@' but NOT matching email address regex
    let req = make_request(
        "tools/call",
        json!({
            "name": "contextra_cloud_query",
            "arguments": {
                "query": "meeting @ 5pm in office"
            }
        }),
    );

    let resp = server.handle(req).await;
    assert!(resp.error.is_none());
    let res_val = serde_json::to_value(&resp).unwrap();
    assert_ne!(res_val["result"]["isError"], true);

    let text = res_val["result"]["content"][0]["text"].as_str().unwrap();
    let json_res: serde_json::Value = serde_json::from_str(text).unwrap();

    assert_eq!(json_res["status"], "success");
    assert_eq!(json_res["query"], "meeting @ 5pm in office");
}

#[tokio::test]
async fn test_cloud_query_validates_empty_and_oversized_query() {
    let (server, _tmp) = create_mock_server().await;

    // 1. Whitespace / empty query
    let req_empty = make_request(
        "contextra_cloud_query",
        json!({
            "query": "   "
        }),
    );
    let resp_empty = server.handle(req_empty).await;
    let err_empty = resp_empty
        .error
        .expect("error expected for empty cloud query");
    assert_eq!(err_empty.code, -32602);
    assert!(err_empty.message.contains("query cannot be empty"));

    // 2. Oversized query (> 64KB)
    let huge_query = "q".repeat(crate::MAX_SEARCH_QUERY_BYTES + 1);
    let req_huge = make_request(
        "contextra_cloud_query",
        json!({
            "query": huge_query
        }),
    );
    let resp_huge = server.handle(req_huge).await;
    let err_huge = resp_huge
        .error
        .expect("error expected for oversized cloud query");
    assert_eq!(err_huge.code, -32602);
    assert!(err_huge.message.contains("query size exceeds limit"));
}
