use contextra_ports::{
    BoxFuture, Clock, CommunityResolver, ContextPreparer, DriftStatusProvider,
    HybridSearchProvider, SystemClock,
};
use contextra_router::profile::SlmProfile;
use contextra_router::wiring::{RouterWiring, RouterWiringDeps};
use contextra_types::{ContextChunk, ContextWindow, DocId, EntityId, Result, TokenBudget};
use std::sync::Arc;

struct MockSearchProvider;
impl HybridSearchProvider for MockSearchProvider {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        _query_embedding: &'a [f32],
        _k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        Box::pin(async move {
            Ok(vec![ContextChunk {
                doc_id: DocId::new(42),
                content: "Production wiring test chunk content".into(),
                relevance: 0.95,
                token_count: 5,
                metadata: None,
                contextual_prefix: None,
                links: vec![],
            }])
        })
    }
}

struct MockCommunityResolver;
impl CommunityResolver for MockCommunityResolver {
    fn get_community<'a>(&'a self, _entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { Ok(Some(1)) })
    }
}

struct MockContextPreparer;
impl ContextPreparer for MockContextPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        _budget: &TokenBudget,
        _relevance_threshold: f32,
    ) -> Result<ContextWindow> {
        let total_tokens = chunks.iter().map(|c| c.token_count).sum();
        Ok(ContextWindow {
            chunks,
            total_tokens,
            truncated: false,
        })
    }
}

#[tokio::test]
async fn test_router_wiring_build_and_handle() {
    let search = Arc::new(MockSearchProvider);
    let comm = Arc::new(MockCommunityResolver);
    let prep = Arc::new(MockContextPreparer);
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());

    let profile = SlmProfile::new(
        "test-slm-profile",
        "sh -c 'cat > /dev/null; echo \"{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":1,\\\"result\\\":{\\\"answer\\\":\\\"test slm response\\\"}}\"'",
        vec![1],
        TokenBudget::default(),
        0.1,
    );

    let deps = RouterWiringDeps {
        search_provider: search,
        community_resolver: comm,
        context_preparer: prep,
        catalog: vec![profile.clone()],
        clock,
        arm_registry: None,
        calibration_store_path: None,
        transport: None,
        route_timeout: None,
    };

    let wiring = Arc::new(RouterWiring::build(deps).expect("build succeeds"));
    assert_eq!(wiring.catalog.len(), 1);
    assert_eq!(wiring.overall_drift_status(), "unbekannt");

    let handle = wiring.handle();
    assert_eq!(handle.engine().profiles().len(), 1);
}

#[tokio::test]
async fn test_router_wiring_production_flow_success_dispatch() {
    let search = Arc::new(MockSearchProvider);
    let comm = Arc::new(MockCommunityResolver);
    let prep = Arc::new(MockContextPreparer);
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());

    // Endpoint consuming stdin and echoing valid JSON-RPC 2.0 response from SLM
    let profile = SlmProfile::new(
        "echo-slm",
        "sh -c 'cat > /dev/null; echo \"{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":1,\\\"result\\\":{\\\"answer\\\":\\\"valid response\\\"}}\"'",
        vec![1],
        TokenBudget::default(),
        0.1,
    );

    let deps = RouterWiringDeps {
        search_provider: search,
        community_resolver: comm,
        context_preparer: prep,
        catalog: vec![profile],
        clock,
        arm_registry: None,
        calibration_store_path: None,
        transport: None,
        route_timeout: None,
    };

    let wiring = Arc::new(RouterWiring::build(deps).expect("build succeeds"));
    let handle = wiring.handle();

    // Execute route and dispatch
    let res = handle.route(&[0.1, 0.2, 0.3], "hello world").await;
    assert!(
        res.is_ok(),
        "Expected successful route & dispatch: {:?}",
        res
    );
    assert_eq!(res.unwrap(), "valid response");

    // Verify outcome was recorded and pending decisions cleared
    assert_eq!(wiring.engine.pending_decision_count(), 0);
}

#[tokio::test]
async fn test_router_wiring_transport_down_fallback_no_panic() {
    let search = Arc::new(MockSearchProvider);
    let comm = Arc::new(MockCommunityResolver);
    let prep = Arc::new(MockContextPreparer);
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());

    // Non-existent binary path simulates transport / endpoint down
    let profile = SlmProfile::new(
        "broken-endpoint-slm",
        "/path/to/nonexistent/slm/binary/9999",
        vec![1],
        TokenBudget::default(),
        0.1,
    );

    let deps = RouterWiringDeps {
        search_provider: search,
        community_resolver: comm,
        context_preparer: prep,
        catalog: vec![profile],
        clock,
        arm_registry: None,
        calibration_store_path: None,
        transport: None,
        route_timeout: None,
    };

    let wiring = Arc::new(RouterWiring::build(deps).expect("build succeeds"));
    let handle = wiring.handle();

    // Execute route and dispatch on broken endpoint
    let res = handle.route(&[0.1, 0.2, 0.3], "failing query").await;
    assert!(res.is_err(), "Expected error when transport is down");
    let err_str = res.unwrap_err().to_string();
    assert!(
        err_str.contains("Fehler bei MCP-Dispatch"),
        "Error message should contain MCP dispatch error detail: {}",
        err_str
    );

    // Verify negative outcome was recorded automatically and pending decisions map cleared without panic
    assert_eq!(wiring.engine.pending_decision_count(), 0);
}

#[tokio::test]
async fn test_router_wiring_determinism_same_seed() {
    let search_a = Arc::new(MockSearchProvider);
    let comm_a = Arc::new(MockCommunityResolver);
    let prep_a = Arc::new(MockContextPreparer);
    let clock_a: Arc<dyn Clock> = Arc::new(SystemClock::new());

    let profile1 = SlmProfile::new(
        "slm-1",
        "/invalid/path/1",
        vec![1],
        TokenBudget::default(),
        0.2,
    );
    let profile2 = SlmProfile::new(
        "slm-2",
        "/invalid/path/2",
        vec![1],
        TokenBudget::default(),
        0.1,
    );

    let deps_a = RouterWiringDeps {
        search_provider: search_a,
        community_resolver: comm_a,
        context_preparer: prep_a,
        catalog: vec![profile1.clone(), profile2.clone()],
        clock: clock_a,
        arm_registry: None,
        calibration_store_path: None,
        transport: None,
        route_timeout: None,
    };

    let wiring_a = Arc::new(RouterWiring::build(deps_a).expect("wiring A build ok"));

    let search_b = Arc::new(MockSearchProvider);
    let comm_b = Arc::new(MockCommunityResolver);
    let prep_b = Arc::new(MockContextPreparer);
    let clock_b: Arc<dyn Clock> = Arc::new(SystemClock::new());

    let deps_b = RouterWiringDeps {
        search_provider: search_b,
        community_resolver: comm_b,
        context_preparer: prep_b,
        catalog: vec![profile1, profile2],
        clock: clock_b,
        arm_registry: None,
        calibration_store_path: None,
        transport: None,
        route_timeout: None,
    };

    let wiring_b = Arc::new(RouterWiring::build(deps_b).expect("wiring B build ok"));

    // Route on both instances with identical inputs
    let decision_a = wiring_a
        .engine
        .route(&[0.1, 0.2, 0.3], "identical test query")
        .await
        .expect("route A ok");

    let decision_b = wiring_b
        .engine
        .route(&[0.1, 0.2, 0.3], "identical test query")
        .await
        .expect("route B ok");

    assert_eq!(decision_a.profile.name, decision_b.profile.name);
    assert_eq!(
        decision_a.decision_id.inner(),
        decision_b.decision_id.inner()
    );
}
