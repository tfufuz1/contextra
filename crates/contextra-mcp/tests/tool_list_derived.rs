use contextra::Contextra;
use contextra_mcp::{protocol::JsonRpcRequest, McpServer};
use contextra_ports::BoxFuture;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Debug)]
struct MockEmbedder;

impl contextra_ports::EmbeddingProvider for MockEmbedder {
    fn provider_name(&self) -> &str {
        "mock"
    }

    fn embed<'a>(
        &'a self,
        _text: &'a str,
    ) -> BoxFuture<'a, std::result::Result<Vec<f32>, contextra_ports::EmbeddingError>> {
        Box::pin(async move { Ok(vec![0.1f32; 128]) })
    }

    fn embedding_dim(&self) -> usize {
        128
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, std::result::Result<Vec<Vec<f32>>, contextra_ports::EmbeddingError>> {
        Box::pin(async move { Ok(vec![vec![0.1f32; 128]; texts.len()]) })
    }
}

#[tokio::test]
async fn tool_list_derived_from_registry() {
    let tmp = TempDir::new().unwrap();
    let db = Arc::new(Contextra::open(tmp.path()).await.unwrap());
    let embedder = Arc::new(MockEmbedder);

    // Server without plugin registry
    let server_no_plugin = McpServer::with_write_permission(db.clone(), embedder.clone(), true).unwrap();
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/list".to_string(),
        params: json!({}),
    };

    let resp = server_no_plugin.handle(req.clone()).await;
    let tools = resp.result.as_ref().unwrap()["tools"].as_array().unwrap();
    let names_no_plugin: Vec<String> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();

    assert!(!names_no_plugin.contains(&"contextra_plugin_status".to_string()));

    // Server with plugin registry
    let gate: Arc<dyn contextra_ports::license::LicenseGate> =
        Arc::new(contextra_ports::license::OpenFastGate);
    let plugin_reg = Arc::new(contextra_ports::plugin::PluginRegistry::new(gate));
    let server_with_plugin = McpServer::with_write_permission(db.clone(), embedder.clone(), true)
        .unwrap()
        .with_plugin_registry(Some(plugin_reg));

    let resp_with = server_with_plugin.handle(req.clone()).await;
    let tools_with = resp_with.result.as_ref().unwrap()["tools"].as_array().unwrap();
    let names_with_plugin: Vec<String> = tools_with
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();

    assert!(names_with_plugin.contains(&"contextra_plugin_status".to_string()));

    // Server with write permission disabled
    let server_no_write = McpServer::with_write_permission(db, embedder, false).unwrap();
    let resp_no_write = server_no_write.handle(req).await;
    let tools_no_write = resp_no_write.result.as_ref().unwrap()["tools"].as_array().unwrap();
    let names_no_write: Vec<String> = tools_no_write
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();

    assert!(!names_no_write.contains(&"contextra_relate".to_string()));
    assert!(!names_no_write.contains(&"contextra_relate_n_ary".to_string()));
}
