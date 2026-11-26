#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::Contextra;
use contextra_mcp::io::{read_line_bounded, MAX_RPC_BYTES};
use contextra_mcp::protocol::JsonRpcRequest;
use contextra_mcp::sandbox::{McpSandbox, SandboxPolicy};
use contextra_mcp::McpServer;
use contextra_ports::BoxFuture;
use serde_json::json;
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
async fn test_dispatch_lifecycle_initialize_and_ping() {
    let (server, _tmp) = create_test_server().await;

    // 1. initialize
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(10)),
        method: "initialize".to_string(),
        params: json!({}),
    };
    let resp = server.handle(req).await;
    assert_eq!(resp.jsonrpc, "2.0");
    assert_eq!(resp.id, Some(json!(10)));
    assert!(resp.error.is_none());
    let res = resp.result.expect("result present");
    assert_eq!(res["protocolVersion"], "2024-11-05");
    assert_eq!(res["serverInfo"]["name"], "contextra");
    assert_eq!(res["serverInfo"]["version"], "0.1.0");

    // 2. initialized
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(11)),
        method: "initialized".to_string(),
        params: json!({}),
    };
    let resp = server.handle(req).await;
    assert_eq!(resp.id, Some(json!(11)));
    assert!(resp.error.is_none());

    // 3. ping
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(12)),
        method: "ping".to_string(),
        params: json!({}),
    };
    let resp = server.handle(req).await;
    assert_eq!(resp.id, Some(json!(12)));
    assert!(resp.error.is_none());
    assert_eq!(resp.result, Some(json!({})));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatch_tools_list() {
    let (server, _tmp) = create_test_server().await;

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(20)),
        method: "tools/list".to_string(),
        params: json!({}),
    };
    let resp = server.handle(req).await;
    assert_eq!(resp.id, Some(json!(20)));
    assert!(resp.error.is_none());

    let result = resp.result.expect("result present");
    let tools = result["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 10);

    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"contextra_search"));
    assert!(names.contains(&"contextra_insert"));
    assert!(names.contains(&"contextra_get"));
    assert!(names.contains(&"contextra_forget"));
    assert!(names.contains(&"contextra_collections"));
    assert!(names.contains(&"contextra_explain"));
    assert!(names.contains(&"contextra_upsert"));
    assert!(names.contains(&"contextra_delete"));
    assert!(names.contains(&"contextra_create_collection"));
    assert!(names.contains(&"contextra_drop_collection"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatch_invalid_protocol_version() {
    let (server, _tmp) = create_test_server().await;

    let req = JsonRpcRequest {
        jsonrpc: "1.0".to_string(),
        id: Some(json!(30)),
        method: "ping".to_string(),
        params: json!({}),
    };
    let resp = server.handle(req).await;
    assert_eq!(resp.id, Some(json!(30)));
    let err = resp.error.expect("error present");
    assert_eq!(err.code, -32600);
    assert!(err.message.contains("jsonrpc version must be '2.0'"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatch_unknown_method() {
    let (server, _tmp) = create_test_server().await;

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(40)),
        method: "non_existent_method".to_string(),
        params: json!({}),
    };
    let resp = server.handle(req).await;
    assert_eq!(resp.id, Some(json!(40)));
    let err = resp.error.expect("error present");
    assert_eq!(err.code, -32601);
    assert!(err.message.contains("Method not found"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatch_tools_call_missing_or_empty_name() {
    let (server, _tmp) = create_test_server().await;

    // Missing 'name'
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(50)),
        method: "tools/call".to_string(),
        params: json!({ "arguments": {} }),
    };
    let resp = server.handle(req).await;
    assert_eq!(resp.id, Some(json!(50)));
    let err = resp.error.expect("error present");
    assert_eq!(err.code, -32602);
    assert!(err.message.contains("missing or empty tool 'name'"));

    // Empty 'name'
    let req_empty = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(51)),
        method: "tools/call".to_string(),
        params: json!({ "name": "", "arguments": {} }),
    };
    let resp_empty = server.handle(req_empty).await;
    assert_eq!(resp_empty.id, Some(json!(51)));
    let err_empty = resp_empty.error.expect("error present");
    assert_eq!(err_empty.code, -32602);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatch_tools_call_unknown_tool() {
    let (server, _tmp) = create_test_server().await;

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(60)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "unknown_tool",
            "arguments": {}
        }),
    };
    let resp = server.handle(req).await;
    assert_eq!(resp.id, Some(json!(60)));
    assert!(resp.error.is_none());
    let res = resp.result.expect("result present");
    assert_eq!(res["isError"], true);
    let text = res["content"][0]["text"].as_str().expect("text string");
    assert!(text.contains("unknown tool") || text.contains("Unbekanntes Tool"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatch_handle_value_non_object_and_empty_batch() {
    let (server, _tmp) = create_test_server().await;

    // Primitive value (not array or object)
    let val_prim = json!("just a string");
    let resp_opt = server.handle_value(val_prim).await;
    let resp = resp_opt.expect("response present");
    assert_eq!(resp["error"]["code"], -32600);
    assert!(resp["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Invalid Request"));

    // Empty batch array
    let val_empty_batch = json!([]);
    let resp_batch_opt = server.handle_value(val_empty_batch).await;
    let resp_batch = resp_batch_opt.expect("response present");
    assert_eq!(resp_batch["error"]["code"], -32600);
    assert!(resp_batch["error"]["message"]
        .as_str()
        .unwrap()
        .contains("empty batch array"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatch_handle_value_notification_vs_request() {
    let (server, _tmp) = create_test_server().await;

    // Notification without id
    let notif = json!({
        "jsonrpc": "2.0",
        "method": "ping"
    });
    let resp = server.handle_value(notif).await;
    assert!(resp.is_none(), "Notifications must yield None");

    // Notification with id: null
    let notif_null = json!({
        "jsonrpc": "2.0",
        "id": null,
        "method": "ping"
    });
    let resp_null = server.handle_value(notif_null).await;
    assert!(
        resp_null.is_none(),
        "Notifications with null id must yield None"
    );

    // Normal request with id
    let req = json!({
        "jsonrpc": "2.0",
        "id": 100,
        "method": "ping"
    });
    let resp_req = server.handle_value(req).await.expect("Some response");
    assert_eq!(resp_req["id"], 100);
    assert!(resp_req["result"].is_object());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatch_handle_value_batch_mixed() {
    let (server, _tmp) = create_test_server().await;

    let batch = json!([
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ping"
        },
        {
            "jsonrpc": "2.0",
            "method": "ping" // notification (no id)
        },
        {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "non_existent_method"
        },
        {
            "invalid_jsonrpc_object": true
        }
    ]);

    let resp_opt = server.handle_value(batch).await;
    let resp = resp_opt.expect("response array present");
    let arr = resp.as_array().expect("array of responses");
    assert_eq!(
        arr.len(),
        3,
        "Notification is not returned in batch response"
    );

    assert_eq!(arr[0]["id"], 1);
    assert!(arr[0]["result"].is_object());

    assert_eq!(arr[1]["id"], 2);
    assert_eq!(arr[1]["error"]["code"], -32601);

    // The invalid request object
    assert_eq!(arr[2]["error"]["code"], -32600);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatch_direct_tool_methods() {
    let (server, _tmp) = create_test_server().await;

    // Direct invocation via method name: "contextra_collections"
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(70)),
        method: "contextra_collections".to_string(),
        params: json!({}),
    };
    let resp = server.handle(req).await;
    assert_eq!(resp.id, Some(json!(70)));
    assert!(resp.error.is_none());
    let res = resp.result.expect("result present");
    assert!(res["collections"].is_array());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_bounded_reader_exceeds_max_rpc_bytes() {
    let max = 100; // custom small bound
    let oversized_data = format!("{}\n", "A".repeat(150));
    let mut cursor = std::io::Cursor::new(oversized_data.into_bytes());
    let mut buf = String::new();

    let res = read_line_bounded(&mut cursor, &mut buf, max).await;
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("Message size limit exceeded"));

    // Standard MAX_RPC_BYTES limit check
    assert_eq!(MAX_RPC_BYTES, 4 * 1024 * 1024);
}
