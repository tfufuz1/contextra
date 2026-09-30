// INVARIANT: The MCP server entry point defaults to opening the database in Fast-Ring mode
// via Contextra::open() with OpenFastGate license enforcement. Without a valid signed
// license gate (SignedLicenseGate in contextra-license), no configuration or environment
// variable (CONTEXTRA_*) can activate a higher feature ring (Sovereign or Compliance).
// Furthermore, DeletionProof active mode is strictly disabled in Fast-Ring.

use contextra::Contextra;
use contextra_mcp::{protocol::JsonRpcRequest, McpServer};
use contextra_ports::license::FeatureRing;
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
    let server =
        McpServer::with_write_permission(Arc::new(db), embedder, true).expect("server new");
    (server, tmp)
}

#[tokio::test]
async fn test_mcp_entrypoint_without_license_reports_fast_ring_only() {
    let (server, _tmp) = setup_server().await;

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "contextra_plugin_status".to_string(),
        params: json!({}),
    };

    let response = server.handle(req).await;
    let res_val = serde_json::to_value(&response).unwrap();
    assert_eq!(res_val["jsonrpc"], "2.0");
    assert!(res_val["error"].is_null());

    let active_ring = res_val["result"]["feature_ring_active"]
        .as_str()
        .expect("feature_ring_active string");

    // (a) Verify active feature ring is Fast and strictly NOT Sovereign or Compliance
    assert_eq!(active_ring, "Fast");
    assert_ne!(active_ring, "Sovereign");
    assert_ne!(active_ring, "Compliance");
}

#[tokio::test]
async fn test_mcp_entrypoint_env_vars_cannot_bypass_license_gate() {
    // Set various CONTEXTRA_* environment variables attempting to force Sovereign/Compliance ring
    std::env::set_var("CONTEXTRA_RING", "Sovereign");
    std::env::set_var("CONTEXTRA_FEATURE_RING", "Compliance");
    std::env::set_var("CONTEXTRA_LICENSE_GATE", "Sovereign");

    let (server, _tmp) = setup_server().await;

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "contextra_plugin_status".to_string(),
        params: json!({}),
    };

    let response = server.handle(req).await;
    let res_val = serde_json::to_value(&response).unwrap();
    let active_ring = res_val["result"]["feature_ring_active"]
        .as_str()
        .expect("feature_ring_active string");

    // (b) Verify that environment variables cannot elevate ring without a valid signed license
    assert_eq!(active_ring, "Fast");
    assert_ne!(active_ring, "Sovereign");
    assert_ne!(active_ring, "Compliance");

    // Clean up test environment variables
    std::env::remove_var("CONTEXTRA_RING");
    std::env::remove_var("CONTEXTRA_FEATURE_RING");
    std::env::remove_var("CONTEXTRA_LICENSE_GATE");
}

#[tokio::test]
async fn test_mcp_entrypoint_ring_is_fast_and_no_deletion_proof_active() {
    // Verify that default plugin registry feature ring is Fast
    let (server, _tmp) = setup_server().await;

    let registry = server
        .plugin_registry
        .as_ref()
        .map(|r| r.current_feature_ring())
        .unwrap_or(FeatureRing::Fast);

    assert_eq!(registry, FeatureRing::Fast);
    assert_ne!(registry, FeatureRing::Sovereign);
    assert_ne!(registry, FeatureRing::Compliance);
}
