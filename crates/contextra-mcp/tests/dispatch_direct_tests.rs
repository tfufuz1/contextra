use contextra::Contextra;
use contextra_mcp::{
    io::{read_line_bounded, MAX_RPC_BYTES},
    protocol::JsonRpcRequest,
    McpServer,
};
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

async fn create_test_server() -> (McpServer, TempDir) {
    let tmp = match TempDir::new() {
        Ok(t) => t,
        Err(e) => panic!("temp dir failed: {e}"),
    };
    let db = match Contextra::open(tmp.path()).await {
        Ok(d) => d,
        Err(e) => panic!("open db failed: {e}"),
    };
    let embedder = Arc::new(MockEmbedder { dimension: 4 });
    let server = match McpServer::with_write_permission(db, embedder, true) {
        Ok(s) => s,
        Err(e) => panic!("server init failed: {e}"),
    };
    (server, tmp)
}

#[tokio::test]
async fn test_dispatch_unknown_method_returns_32601() {
    let (server, _tmp) = create_test_server().await;

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "non_existent_method".to_string(),
        params: None,
    };

    let resp = server.handle(req).await;
    assert_eq!(resp.id, Some(json!(1)));
    let err = match resp.error {
        Some(e) => e,
        None => panic!("error expected for unknown method"),
    };
    assert_eq!(err.code, -32601);
    assert!(
        err.message.contains("Method not found"),
        "Unexpected error message: {}",
        err.message
    );
}

#[tokio::test]
async fn test_dispatch_invalid_protocol_version_returns_32600() {
    let (server, _tmp) = create_test_server().await;

    let raw = json!({
        "jsonrpc": "1.0",
        "id": 42,
        "method": "ping"
    });

    let resp_val = match server.handle_value(raw).await {
        Some(v) => v,
        None => panic!("response expected"),
    };
    assert_eq!(resp_val["id"], 42);
    assert_eq!(resp_val["error"]["code"], -32600);
    assert!(resp_val["error"]["message"]
        .as_str()
        .unwrap_or("")
        .contains("Invalid JSON-RPC"));
}

#[tokio::test]
async fn test_dispatch_notifications_return_none() {
    let (server, _tmp) = create_test_server().await;

    // Notification without id
    let notif_no_id = json!({
        "jsonrpc": "2.0",
        "method": "initialized"
    });
    let resp = server.handle_value(notif_no_id).await;
    assert!(
        resp.is_none(),
        "Notification without id must not produce response"
    );

    // Notification with null id
    let notif_null_id = json!({
        "jsonrpc": "2.0",
        "id": null,
        "method": "initialized"
    });
    let resp_null = server.handle_value(notif_null_id).await;
    assert!(
        resp_null.is_none(),
        "Notification with id=null must not produce response"
    );
}

#[tokio::test]
async fn test_dispatch_empty_batch_returns_32600() {
    let (server, _tmp) = create_test_server().await;

    let empty_batch = json!([]);
    let resp = match server.handle_value(empty_batch).await {
        Some(v) => v,
        None => panic!("response expected for empty batch"),
    };
    assert_eq!(resp["error"]["code"], -32600);
    assert!(resp["error"]["message"]
        .as_str()
        .unwrap_or("")
        .contains("empty batch"));
}

#[tokio::test]
async fn test_dispatch_non_object_request_returns_32600() {
    let (server, _tmp) = create_test_server().await;

    let invalid_json = json!("just a string");
    let resp = match server.handle_value(invalid_json).await {
        Some(v) => v,
        None => panic!("response expected"),
    };
    assert_eq!(resp["error"]["code"], -32600);
}

#[tokio::test]
async fn test_dispatch_batch_with_notifications_and_requests() {
    let (server, _tmp) = create_test_server().await;

    let batch = json!([
        // notification 1
        { "jsonrpc": "2.0", "method": "initialized" },
        // request 1
        { "jsonrpc": "2.0", "id": 10, "method": "ping" },
        // notification 2
        { "jsonrpc": "2.0", "id": null, "method": "initialized" },
        // request 2 (unknown)
        { "jsonrpc": "2.0", "id": 20, "method": "unknown_call" }
    ]);

    let resp = match server.handle_value(batch).await {
        Some(v) => v,
        None => panic!("response expected for batch"),
    };
    let arr = match resp.as_array() {
        Some(a) => a,
        None => panic!("batch response must be an array"),
    };
    assert_eq!(
        arr.len(),
        2,
        "Only the 2 requests should produce response elements"
    );

    let id10 = match arr.iter().find(|r| r["id"] == 10) {
        Some(r) => r,
        None => panic!("resp for id 10 expected"),
    };
    assert_eq!(id10["result"], json!({}));

    let id20 = match arr.iter().find(|r| r["id"] == 20) {
        Some(r) => r,
        None => panic!("resp for id 20 expected"),
    };
    assert_eq!(id20["error"]["code"], -32601);
}

#[tokio::test]
async fn test_dispatch_tools_call_missing_or_invalid_params() {
    let (server, _tmp) = create_test_server().await;

    // Missing params completely
    let req_missing = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: None,
    };
    let resp_missing = server.handle(req_missing).await;
    let err_missing = match resp_missing.error {
        Some(e) => e,
        None => panic!("error expected"),
    };
    assert_eq!(err_missing.code, -32602);

    // Params is not an object
    let req_invalid_type = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: Some(json!("not an object")),
    };
    let resp_invalid_type = server.handle(req_invalid_type).await;
    let err_invalid_type = match resp_invalid_type.error {
        Some(e) => e,
        None => panic!("error expected"),
    };
    assert_eq!(err_invalid_type.code, -32602);

    // Missing 'name' field in params
    let req_no_name = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "tools/call".to_string(),
        params: Some(json!({ "arguments": {} })),
    };
    let resp_no_name = server.handle(req_no_name).await;
    let err_no_name = match resp_no_name.error {
        Some(e) => e,
        None => panic!("error expected"),
    };
    assert_eq!(err_no_name.code, -32602);

    // 'name' is not a string
    let req_non_string_name = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(4)),
        method: "tools/call".to_string(),
        params: Some(json!({ "name": 12345 })),
    };
    let resp_non_string_name = server.handle(req_non_string_name).await;
    let err_non_string_name = match resp_non_string_name.error {
        Some(e) => e,
        None => panic!("error expected"),
    };
    assert_eq!(err_non_string_name.code, -32602);
}

#[tokio::test]
async fn test_dispatch_unknown_tool_call_returns_error_result() {
    let (server, _tmp) = create_test_server().await;

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "totally_unknown_tool_xyz",
            "arguments": {}
        })),
    };

    let resp = server.handle(req).await;
    let res = match resp.result {
        Some(r) => r,
        None => panic!("result envelope expected"),
    };
    assert_eq!(res["isError"], true);
    let text = res["content"][0]["text"].as_str().unwrap_or("");
    assert!(
        text.contains("unknown tool") || text.contains("Unbekanntes Tool"),
        "Unexpected content text: {text}"
    );
}

#[tokio::test]
async fn test_dispatch_direct_builtin_methods() {
    let (server, _tmp) = create_test_server().await;

    // initialize
    let req_init = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "initialize".to_string(),
        params: Some(json!({})),
    };
    let resp_init = server.handle(req_init).await;
    let init_res = match resp_init.result {
        Some(r) => r,
        None => panic!("initialize result expected"),
    };
    assert_eq!(init_res["protocolVersion"], "2024-11-05");
    assert_eq!(init_res["serverInfo"]["name"], "contextra");

    // ping
    let req_ping = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "ping".to_string(),
        params: None,
    };
    let resp_ping = server.handle(req_ping).await;
    assert_eq!(resp_ping.result, Some(json!({})));

    // tools/list
    let req_list = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "tools/list".to_string(),
        params: None,
    };
    let resp_list = server.handle(req_list).await;
    let list_res = match resp_list.result {
        Some(r) => r,
        None => panic!("tools/list result expected"),
    };
    let tools = match list_res["tools"].as_array() {
        Some(t) => t,
        None => panic!("tools array expected"),
    };
    assert!(!tools.is_empty(), "tools/list should not be empty");
}

#[tokio::test]
async fn test_bounded_line_reading_size_limit() {
    // Check that read_line_bounded rejects oversized lines (> MAX_RPC_BYTES = 4MB)
    let huge_line = vec![b'a'; MAX_RPC_BYTES + 10];
    let mut reader = tokio::io::BufReader::new(&huge_line[..]);
    let mut buf = String::new();

    let res = read_line_bounded(&mut reader, &mut buf).await;
    assert!(res.is_err(), "Must error on oversized line");
    let err = match res {
        Err(e) => e,
        Ok(_) => unreachable!(),
    };
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("exceeds 4MB limit"));

    // Check that a line within the limit passes
    let valid_line = b"{\"jsonrpc\": \"2.0\", \"method\": \"ping\"}\n";
    let mut reader_valid = tokio::io::BufReader::new(&valid_line[..]);
    let mut buf_valid = String::new();

    let res_valid = read_line_bounded(&mut reader_valid, &mut buf_valid).await;
    assert!(res_valid.is_ok(), "Must succeed on line within limit");
    let bytes_read = match res_valid {
        Ok(b) => b,
        Err(_) => unreachable!(),
    };
    assert_eq!(bytes_read, valid_line.len());
}
