#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_router::{DecisionId, RouterEngine, RoutingOutcome, SlmProfile};
use contextra_types::{
    ContextChunk, ContextWindow, ContextraError, DocId, EntityId, Result, TokenBudget,
};
use std::sync::Arc;

#[cfg(feature = "bandit-routing")]
use contextra_adapt::bandit::BanditProfileState;
#[cfg(feature = "bandit-routing")]
use contextra_router::RoutingStrategy;

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
async fn test_with_initial_decision_id_wiring() {
    let dim = 4;
    let p1 = SlmProfile::new(
        "p1",
        "http://loc1",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );

    let start_id = 5000u64;
    let router = create_mock_router(vec![p1], dim).with_initial_decision_id(start_id);

    let embedding = vec![0.5f32; dim];
    let decision = router.route(&embedding, "query 1").await.unwrap();

    assert_eq!(decision.decision_id, DecisionId::from_raw(start_id));

    let decision2 = router.route(&embedding, "query 2").await.unwrap();
    assert_eq!(decision2.decision_id, DecisionId::from_raw(start_id + 1));
}

#[cfg(feature = "bandit-routing")]
#[tokio::test]
async fn test_with_routing_strategy_and_bandit_propensity_wiring() {
    let dim = 4;
    let mut p1 = SlmProfile::new(
        "p1",
        "http://loc1",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );
    p1.bandit_state = Some(BanditProfileState::cold_start(dim, 0.5));

    let mut p2 = SlmProfile::new(
        "p2",
        "http://loc2",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );
    p2.bandit_state = Some(BanditProfileState::cold_start(dim, 0.5));

    let router = create_mock_router(vec![p1, p2], dim).with_routing_strategy(
        RoutingStrategy::ContextualBandit,
        0.1,
        12345,
    );

    let embedding = vec![0.5f32; dim];
    let decision = router.route(&embedding, "query").await.unwrap();

    let propensity = router.bandit_decision_propensity(decision.decision_id);
    assert!(propensity.is_some());
    assert!(propensity.unwrap() > 0.0);

    let unknown_propensity = router.bandit_decision_propensity(DecisionId::from_raw(999_999));
    assert!(unknown_propensity.is_none());
}

#[tokio::test]
async fn test_try_update_profiles_wiring() {
    let dim = 4;
    let p1 = SlmProfile::new(
        "p1",
        "http://loc1",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );

    let router = create_mock_router(vec![p1], dim);

    // Valid profile update
    let p1_updated = SlmProfile::new(
        "p1",
        "http://loc1_updated",
        vec![1],
        TokenBudget::new(2000, 200),
        0.6,
    );
    let update_res = router.try_update_profiles(vec![p1_updated]);
    assert!(update_res.is_ok());
    assert_eq!(router.profiles()[0].min_relevance_score, 0.6);

    // Invalid profile update (e.g. NaN min_relevance_score)
    let p_invalid = SlmProfile::new(
        "p_invalid",
        "http://loc_invalid",
        vec![1],
        TokenBudget::new(1000, 100),
        f32::NAN,
    );
    let invalid_res = router.try_update_profiles(vec![p_invalid]);
    assert!(invalid_res.is_err());
    // Verification: previous valid profile remains unchanged
    assert_eq!(router.profiles()[0].min_relevance_score, 0.6);
}

#[tokio::test]
async fn test_pending_decision_count_wiring() {
    let dim = 4;
    let p1 = SlmProfile::new(
        "p1",
        "http://loc1",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );

    let router = create_mock_router(vec![p1], dim);

    assert_eq!(router.pending_decision_count(), 0);

    let embedding = vec![0.5f32; dim];
    let decision = router.route(&embedding, "query").await.unwrap();

    assert_eq!(router.pending_decision_count(), 1);

    let recorded = router.record_outcome(decision.decision_id, RoutingOutcome::Success);
    assert!(recorded);

    assert_eq!(router.pending_decision_count(), 0);
}

#[tokio::test]
async fn test_reset_all_calibration_wiring() {
    let dim = 4;
    let p1 = SlmProfile::new(
        "p1",
        "http://loc1",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );
    let p2 = SlmProfile::new(
        "p2",
        "http://loc2",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );

    let router = create_mock_router(vec![p1, p2], dim);

    let embedding = vec![0.5f32; dim];
    let decision = router.route(&embedding, "query").await.unwrap();
    router.record_outcome(decision.decision_id, RoutingOutcome::Success);

    // Initial stats checked
    let initial_stats = router.calibration_stats();
    assert!(initial_stats.contains_key("p1"));

    // Reset all calibration stats
    router.reset_all_calibration();

    let reset_stats = router.calibration_stats();
    for (_name, state) in reset_stats {
        assert_eq!(state.conformal.window_total, 0);
    }
}

#[tokio::test]
async fn test_set_lyapunov_baseline_wiring() {
    let dim = 4;
    let p1 = SlmProfile::new(
        "p1",
        "http://loc1",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );

    let router = create_mock_router(vec![p1], dim);

    let baseline: Vec<f32> = (0..50).map(|i| (i as f32) / 50.0).collect();

    // Set baseline for existing profile
    let set_ok = router.set_lyapunov_baseline("p1", &baseline);
    assert!(set_ok);

    // Set baseline for non-existent profile returns false
    let set_fail = router.set_lyapunov_baseline("nonexistent", &baseline);
    assert!(!set_fail);

    // Verify routing works and updates Lyapunov drift status when baseline set
    let embedding = vec![0.5f32; dim];
    let decision = router.route(&embedding, "query").await.unwrap();
    assert_eq!(decision.profile.name, "p1");
}
