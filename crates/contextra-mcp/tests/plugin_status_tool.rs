use contextra::Contextra;
use contextra_mcp::{
    protocol::JsonRpcRequest,
    McpServer,
};
use contextra_ports::BoxFuture;
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

async fn setup_server() -> (McpServer, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let col = db.collection("default").await.expect("collection");
    let dim = col.dimension();
    let embedder = Arc::new(MockEmbedder { dimension: dim });
    let server = McpServer::with_write_permission(Arc::new(db), embedder, true).expect("server new");
    (server, tmp)
}

#[tokio::test]
async fn test_tools_list_contains_contextra_plugin_status() {
    let (server, _tmp) = setup_server().await;

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/list".to_string(),
        params: json!({}),
    };

    let response = server.handle(req).await;
    let res_val = serde_json::to_value(&response).unwrap();
    let tools = res_val["result"]["tools"].as_array().expect("tools array");

    let plugin_status_tool = tools
        .iter()
        .find(|t| t["name"] == "contextra_plugin_status")
        .expect("contextra_plugin_status must be registered in tools/list");

    assert_eq!(plugin_status_tool["name"], "contextra_plugin_status");
    assert!(plugin_status_tool["description"].is_string());
}

#[tokio::test]
async fn test_contextra_plugin_status_tools_call() {
    let (server, _tmp) = setup_server().await;

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: json!({
            "name": "contextra_plugin_status",
            "arguments": {}
        }),
    };

    let response = server.handle(req).await;
    let res_val = serde_json::to_value(&response).unwrap();
    assert_eq!(res_val["jsonrpc"], "2.0");
    assert_eq!(res_val["id"], 2);
    assert!(res_val["error"].is_null());

    let content_text = res_val["result"]["content"][0]["text"]
        .as_str()
        .expect("content text");

    let status_val: serde_json::Value = serde_json::from_str(content_text)
        .expect("payload must deserialize from valid JSON");

    assert!(status_val.get("plugins").is_some());
    assert!(status_val.get("feature_ring_active").is_some());
}

#[tokio::test]
async fn test_contextra_plugin_status_direct_method_call() {
    let (server, _tmp) = setup_server().await;

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "contextra_plugin_status".to_string(),
        params: json!({}),
    };

    let response = server.handle(req).await;
    let res_val = serde_json::to_value(&response).unwrap();
    assert_eq!(res_val["jsonrpc"], "2.0");
    assert_eq!(res_val["id"], 3);
    assert!(res_val["error"].is_null());

    let result = &res_val["result"];
    assert!(result.get("plugins").is_some());
    assert!(result.get("feature_ring_active").is_some());
}

#[test]
fn test_plugin_status_response_serialization_security_boundary() {
    use contextra_mcp::{PluginStatusEntry, PluginStatusResponse};
    use contextra_ports::license::FeatureRing;

    let response = PluginStatusResponse {
        plugins: vec![
            PluginStatusEntry {
                name: "test_plugin".to_string(),
                version: "1.2.3".to_string(),
                ring: 1,
                feature_ring_required: FeatureRing::Fast,
            },
            PluginStatusEntry {
                name: "sovereign_plugin".to_string(),
                version: "2.0.0".to_string(),
                ring: 2,
                feature_ring_required: FeatureRing::Sovereign,
            },
        ],
        feature_ring_active: FeatureRing::Fast,
    };

    let serialized = serde_json::to_value(&response).expect("serialization");

    // Verify correct structure
    assert_eq!(serialized["feature_ring_active"], "Fast");
    let plugins = serialized["plugins"].as_array().expect("plugins array");
    assert_eq!(plugins.len(), 2);
    assert_eq!(plugins[0]["name"], "test_plugin");
    assert_eq!(plugins[0]["version"], "1.2.3");
    assert_eq!(plugins[0]["ring"], 1);
    assert_eq!(plugins[0]["feature_ring_required"], "Fast");

    assert_eq!(plugins[1]["name"], "sovereign_plugin");
    assert_eq!(plugins[1]["version"], "2.0.0");
    assert_eq!(plugins[1]["ring"], 2);
    assert_eq!(plugins[1]["feature_ring_required"], "Sovereign");

    // Security boundary verification: verify absence of key/ticket fields
    let serialized_str = serde_json::to_string(&response).expect("to string");
    assert!(!serialized_str.contains("key"));
    assert!(!serialized_str.contains("ticket"));
    assert!(!serialized_str.contains("signature"));
    assert!(!serialized_str.contains("secret"));
}
