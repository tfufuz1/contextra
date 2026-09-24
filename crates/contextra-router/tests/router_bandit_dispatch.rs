#![cfg(feature = "bandit-routing")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_adapt::bandit::BanditProfileState;
use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_types::{
    ContextChunk, ContextWindow, ContextraError, DocId, EntityId, Result, TokenBudget,
};
use contextra_router::{RouterEngine, RoutingOutcome, RoutingStrategy, SlmProfile};
use std::sync::Arc;

struct MockSearchProvider {
    dimension: usize,
}

impl HybridSearchProvider for MockSearchProvider {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        query_embedding: &'a [f32],
        k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        let dim = self.dimension;
        let embedding_len = query_embedding.len();
        Box::pin(async move {
            if embedding_len != dim {
                return Err(ContextraError::InvalidInput("Dimension mismatch".into()));
            }
            let chunk = ContextChunk {
                doc_id: DocId::new(101),
                content: "Mock search content chunk".to_string(),
                relevance: 0.85,
                token_count: 5,
                metadata: None,
                contextual_prefix: None,
                links: vec![],
            };
            Ok(vec![chunk; k.min(3)])
        })
    }
}

struct MockCommunityResolver {
    comm_id: Option<u64>,
}

impl CommunityResolver for MockCommunityResolver {
    fn get_community<'a>(&'a self, _entity: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        let comm_id = self.comm_id;
        Box::pin(async move { Ok(comm_id) })
    }
}

struct MockContextPreparer;

impl ContextPreparer for MockContextPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        _budget: &TokenBudget,
        _min_score: f32,
    ) -> Result<ContextWindow> {
        let total_tokens = chunks.iter().map(|c| c.token_count).sum();
        Ok(ContextWindow {
            chunks,
            total_tokens,
            truncated: false,
        })
    }
}

fn create_mock_router(profiles: Vec<SlmProfile>, dim: usize) -> RouterEngine {
    let search = Arc::new(MockSearchProvider { dimension: dim });
    let comm = Arc::new(MockCommunityResolver { comm_id: Some(1) });
    let prep = Arc::new(MockContextPreparer);
    RouterEngine::new(search, comm, prep, profiles, None)
}

#[tokio::test]
async fn test_default_strategy_is_cascade() {
    let dim = 4;
    let mut p1 = SlmProfile::new("p1", "http://loc1", vec![1], TokenBudget::new(1000, 100), 0.5);
    p1.bandit_state = Some(BanditProfileState::cold_start(dim, 0.5));

    let mut p2 = SlmProfile::new("p2", "http://loc2", vec![1], TokenBudget::new(1000, 100), 0.8);
    p2.bandit_state = Some(BanditProfileState::cold_start(dim, 0.5));

    let router = create_mock_router(vec![p1, p2], dim);
    let embedding = vec![0.5f32; dim];

    let decision = router.route(&embedding, "test query").await.unwrap();
    // Cascade selects p2 first because min_relevance_score is 0.8 (higher precision)
    assert_eq!(decision.profile.name, "p2");
}

#[tokio::test]
async fn test_bandit_deterministic_selection_with_same_seed() {
    let dim = 4;
    let mut p1 = SlmProfile::new("p1", "http://loc1", vec![1], TokenBudget::new(1000, 100), 0.5);
    p1.bandit_state = Some(BanditProfileState::cold_start(dim, 0.5));

    let mut p2 = SlmProfile::new("p2", "http://loc2", vec![1], TokenBudget::new(1000, 100), 0.5);
    p2.bandit_state = Some(BanditProfileState::cold_start(dim, 0.5));

    let router1 = create_mock_router(vec![p1.clone(), p2.clone()], dim)
        .with_routing_strategy(RoutingStrategy::ContextualBandit, 0.1, 42);

    let router2 = create_mock_router(vec![p1.clone(), p2.clone()], dim)
        .with_routing_strategy(RoutingStrategy::ContextualBandit, 0.1, 42);

    let embedding = vec![0.5f32; dim];

    let dec1 = router1.route(&embedding, "query 1").await.unwrap();
    let dec2 = router2.route(&embedding, "query 1").await.unwrap();

    assert_eq!(dec1.profile.name, dec2.profile.name);
}

#[tokio::test]
async fn test_record_outcome_updates_theta() {
    let dim = 4;
    let mut p1 = SlmProfile::new("p1", "http://loc1", vec![1], TokenBudget::new(1000, 100), 0.5)
        .with_resource_cost_estimate(0.1);
    p1.bandit_state = Some(BanditProfileState::cold_start(dim, 0.5));

    let router = create_mock_router(vec![p1], dim)
        .with_routing_strategy(RoutingStrategy::ContextualBandit, 0.0, 42);

    let embedding = vec![1.0f32, 0.0, 0.0, 0.0];
    let dec = router.route(&embedding, "test query").await.unwrap();

    let theta_before = router.profiles()[0]
        .bandit_state
        .as_ref()
        .unwrap()
        .theta
        .clone();

    let success = router.record_outcome(dec.decision_id, RoutingOutcome::Success);
    assert!(success);

    let theta_after = router.profiles()[0]
        .bandit_state
        .as_ref()
        .unwrap()
        .theta
        .clone();

    assert!(theta_after[0] > theta_before[0]);
}

#[tokio::test]
async fn test_fallback_to_cascade_when_bandit_state_missing() {
    let dim = 4;
    let p1 = SlmProfile::new("p1", "http://loc1", vec![1], TokenBudget::new(1000, 100), 0.5);
    // p1 has no bandit_state (None)

    let router = create_mock_router(vec![p1], dim)
        .with_routing_strategy(RoutingStrategy::ContextualBandit, 0.1, 42);

    let embedding = vec![0.5f32; dim];
    let decision = router.route(&embedding, "query").await;
    assert!(decision.is_ok());
    assert_eq!(decision.unwrap().profile.name, "p1");
}

#[tokio::test]
async fn test_fallback_to_cascade_on_dimension_mismatch() {
    let dim = 4;
    let mut p1 = SlmProfile::new("p1", "http://loc1", vec![1], TokenBudget::new(1000, 100), 0.5);
    // Bandit state expects dimension 8, but query embedding will be dimension 4
    p1.bandit_state = Some(BanditProfileState::cold_start(8, 0.5));

    let router = create_mock_router(vec![p1], dim)
        .with_routing_strategy(RoutingStrategy::ContextualBandit, 0.1, 42);

    let embedding = vec![0.5f32; dim]; // Dimension 4 != 8
    let decision = router.route(&embedding, "query").await;
    assert!(decision.is_ok());
    assert_eq!(decision.unwrap().profile.name, "p1");
}

#[tokio::test]
async fn test_bandit_decision_propensity_floor_clamp() {
    let dim = 4;
    let mut p1 = SlmProfile::new("p1", "http://loc1", vec![1], TokenBudget::new(1000, 100), 0.5);
    p1.bandit_state = Some(BanditProfileState::cold_start(dim, 0.5));

    let mut p2 = SlmProfile::new("p2", "http://loc2", vec![1], TokenBudget::new(1000, 100), 0.5);
    p2.bandit_state = Some(BanditProfileState::cold_start(dim, 0.5));

    // Test with epsilon = 0.0 -> clamped epsilon should be at least 0.01 * K = 0.02
    let router = create_mock_router(vec![p1, p2], dim)
        .with_routing_strategy(RoutingStrategy::ContextualBandit, 0.0, 42);

    let embedding = vec![0.5f32; dim];
    let decision = router.route(&embedding, "query").await.unwrap();

    let propensity = router.bandit_decision_propensity(decision.decision_id);
    assert!(propensity.is_some());
    let p = propensity.unwrap();
    assert!(p >= 0.01, "Propensity must be >= 0.01, got {}", p);
}
