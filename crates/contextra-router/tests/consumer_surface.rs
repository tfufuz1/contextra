#![allow(clippy::unwrap_used, clippy::expect_used)]
//! # Konsumentenoberflächen-Test für contextra-mcp (WP-RT-1.6)
//!
//! Spiegelt exakt die von `contextra-mcp` verwendeten Typen, Konstruktoren und Signaturen wider:
//! - `contextra_router::DefaultRouterEngine`
//! - `contextra_router::RouterEngine::new(...)` (wie in `mcp/src/routing.rs:96`)
//! - `contextra_router::SlmProfile::new(...)` (wie in `mcp/tests/mcp_test.rs:979`)
//! - Serde-Roundtrip von `Vec<SlmProfile>` (wie in `mcp/src/config.rs`)
//! - Drift-Status-Abruf (`overall_drift_status`)

use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, DriftStatusProvider, HybridSearchProvider};
use contextra_router::{DefaultRouterEngine, RouterEngine, SlmProfile};
use contextra_types::{ContextChunk, ContextWindow, EntityId, Result, TokenBudget};
use std::sync::Arc;

struct ConsumerSearchMock;
impl HybridSearchProvider for ConsumerSearchMock {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        _query_embedding: &'a [f32],
        _top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        Box::pin(async move { Ok(vec![]) })
    }
}

struct ConsumerCommunityMock;
impl CommunityResolver for ConsumerCommunityMock {
    fn get_community<'a>(&'a self, _entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { Ok(None) })
    }
}

struct ConsumerPreparerMock;
impl ContextPreparer for ConsumerPreparerMock {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        _budget: &TokenBudget,
        _relevance_threshold: f32,
    ) -> Result<ContextWindow> {
        Ok(ContextWindow {
            chunks,
            total_tokens: 0,
            truncated: false,
        })
    }
}

struct RouterDriftAdapter(Arc<DefaultRouterEngine>);

impl DriftStatusProvider for RouterDriftAdapter {
    fn overall_drift_status(&self) -> String {
        self.0.overall_drift_status()
    }
}

// ANCHOR[contextra-router:TST-CNS-001]
#[tokio::test]
async fn test_mcp_consumer_surface_contracts() {
    // 1. SlmProfile::new wie in mcp/tests/mcp_test.rs:979
    let profile = SlmProfile::new(
        "test-profile",
        "http://localhost:11434",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );

    // 2. Serde-Roundtrip für Vec<SlmProfile> wie in mcp/src/config.rs
    let profiles = vec![profile];
    let serialized = serde_json::to_vec(&profiles).expect("serialize profiles");
    let deserialized: Vec<SlmProfile> =
        serde_json::from_slice(&serialized).expect("deserialize profiles");
    assert_eq!(profiles, deserialized);

    // 3. RouterEngine::new wie in mcp/src/routing.rs:96
    let search_provider = Arc::new(ConsumerSearchMock);
    let community_resolver = Arc::new(ConsumerCommunityMock);
    let context_preparer = Arc::new(ConsumerPreparerMock);
    let calibration_store_path: Option<std::path::PathBuf> = None;

    let router: Arc<DefaultRouterEngine> = Arc::new(RouterEngine::new(
        search_provider,
        community_resolver,
        context_preparer,
        deserialized,
        calibration_store_path,
    ));

    // 4. Drift-Adapter wie in mcp/src/routing.rs:77
    let drift_adapter: Arc<dyn DriftStatusProvider> =
        Arc::new(RouterDriftAdapter(router.clone()));

    let status = drift_adapter.overall_drift_status();
    assert_eq!(status, "unbekannt");
}
