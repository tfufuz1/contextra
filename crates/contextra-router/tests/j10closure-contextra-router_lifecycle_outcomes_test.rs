use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_router::{
    RoutingOutcome, SlmProfile,
};
#[cfg(feature = "bandit-routing")]
use contextra_router::DecisionId;
#[cfg(feature = "bandit-routing")]
use contextra_router::RoutingStrategy;
use contextra_types::{ContextChunk, ContextWindow, DocId, EntityId, Result, TokenBudget};
use std::sync::Arc;

struct MockSearch;
impl HybridSearchProvider for MockSearch {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        _query_embedding: &'a [f32],
        _k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        Box::pin(async move {
            Ok(vec![ContextChunk {
                doc_id: DocId::new(1),
                content: "mock context".into(),
                relevance: 0.95,
                token_count: 5,
                metadata: None,
                contextual_prefix: None,
                links: vec![],
            }])
        })
    }
}

struct MockCommunity;
impl CommunityResolver for MockCommunity {
    fn get_community<'a>(&'a self, _entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { Ok(Some(10)) })
    }
}

struct MockPreparer;
impl ContextPreparer for MockPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        _budget: &TokenBudget,
        _relevance_threshold: f32,
    ) -> Result<ContextWindow> {
        let total = chunks.iter().map(|c| c.token_count).sum();
        Ok(ContextWindow {
            chunks,
            total_tokens: total,
            truncated: false,
        })
    }
}

fn create_mock_router(profiles: Vec<SlmProfile>) -> contextra_router::RouterEngine {
    contextra_router::RouterEngine::new(
        Arc::new(MockSearch),
        Arc::new(MockCommunity),
        Arc::new(MockPreparer),
        profiles,
        None,
    )
}

#[tokio::test]
async fn test_j10closure_router_lifecycle_and_outcomes() {
    let p1 = SlmProfile::new("p1", "mcp1", vec![10], TokenBudget::default(), 0.5);
    let p2 = SlmProfile::new("p2", "mcp2", vec![20], TokenBudget::default(), 0.5);

    // 1. with_initial_decision_id
    let router = create_mock_router(vec![p1.clone(), p2.clone()])
        .with_initial_decision_id(500);

    #[cfg(feature = "bandit-routing")]
    let router = router.with_routing_strategy(RoutingStrategy::ContextualBandit, 0.2, 9999);

    // Check routing produces decisions starting from set decision_id
    let decision = router
        .route(&[0.1, 0.2], "j10 closure test query")
        .await
        .expect("routing should succeed");

    assert!(
        decision.decision_id.inner() >= 500,
        "Expected decision_id >= 500, got {:?}",
        decision.decision_id
    );

    // 3. pending_decision_count
    assert_eq!(router.pending_decision_count(), 1);

    #[cfg(feature = "bandit-routing")]
    {
        // 4. bandit_decision_propensity
        let propensity = router.bandit_decision_propensity(decision.decision_id);
        assert!(propensity.is_some());

        let unknown_propensity =
            router.bandit_decision_propensity(DecisionId::from_raw(999_999));
        assert!(unknown_propensity.is_none());
    }

    // Record outcome to clear pending decision
    let recorded = router.record_outcome(
        decision.decision_id,
        RoutingOutcome::Success,
    );
    assert!(recorded);
    assert_eq!(router.pending_decision_count(), 0);

    // 5. set_lyapunov_baseline
    let baseline = vec![0.1; 128];
    assert!(router.set_lyapunov_baseline("p1", &baseline));
    assert!(!router.set_lyapunov_baseline("nonexistent_profile", &baseline));

    // 6. reset_all_calibration
    router.reset_all_calibration();
    let stats = router.calibration_stats();
    assert!(stats.contains_key("p1"));
    assert!(stats.contains_key("p2"));

    // 7. try_update_profiles
    let p1_updated = SlmProfile::new("p1", "mcp1_new", vec![10], TokenBudget::default(), 0.6);
    let update_res = router.try_update_profiles(vec![p1_updated]);
    assert!(update_res.is_ok());
    assert_eq!(router.profiles().len(), 1);
    assert_eq!(router.profiles()[0].min_relevance_score, 0.6);

    let invalid_profile = SlmProfile::new("", "", vec![], TokenBudget::default(), 0.5);
    let invalid_res = router.try_update_profiles(vec![invalid_profile]);
    assert!(invalid_res.is_err());
}
