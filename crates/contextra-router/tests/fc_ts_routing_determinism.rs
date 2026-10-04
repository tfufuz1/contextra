#![allow(clippy::unwrap_used, clippy::expect_used)]

#[cfg(feature = "flow-corrected-thompson")]
use contextra_adapt::{FcTsArmSet, FcTsConfig, FlowCorrectedThompsonBandit};
use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
#[cfg(feature = "flow-corrected-thompson")]
use contextra_router::{FcTsDispatchError, deterministic_fc_ts_rng, select_profile_fc_ts};
use contextra_router::{RouterEngine, SlmProfile};
use contextra_types::{
    ContextChunk, ContextWindow, ContextraError, DocId, EntityId, Result, TokenBudget,
};
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
async fn test_default_cascade_strategy_unchanged() {
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
        0.8,
    );

    let router = create_mock_router(vec![p1, p2], dim);
    let embedding = vec![0.5f32; dim];

    let decision = router
        .route(&embedding, "test query")
        .await
        .expect("routing succeeds");
    // Cascade selects p2 first because min_relevance_score is 0.8 (higher precision)
    assert_eq!(decision.profile.name, "p2");
}

#[cfg(feature = "flow-corrected-thompson")]
#[test]
fn test_fc_ts_selection_determinism_same_seed() {
    let dim = 4;
    let cfg = FcTsConfig {
        dim,
        ..Default::default()
    };
    let arm1 = FlowCorrectedThompsonBandit::new(cfg.clone()).expect("valid arm");
    let arm2 = FlowCorrectedThompsonBandit::new(cfg).expect("valid arm");
    let arm_set = FcTsArmSet {
        arms: vec![arm1, arm2],
    };

    let profile_names = vec!["profile_1".to_string(), "profile_2".to_string()];
    let context = vec![0.5, -0.1, 0.8, 0.2];

    let seed = 987654321u64;

    let mut rng1 = deterministic_fc_ts_rng(seed);
    let (idx1, name1) = select_profile_fc_ts(&profile_names, &arm_set, &context, &mut rng1)
        .expect("fc_ts selection ok");

    let mut rng2 = deterministic_fc_ts_rng(seed);
    let (idx2, name2) = select_profile_fc_ts(&profile_names, &arm_set, &context, &mut rng2)
        .expect("fc_ts selection ok");

    assert_eq!(idx1, idx2);
    assert_eq!(name1, name2);
}

#[cfg(feature = "flow-corrected-thompson")]
#[test]
fn test_fc_ts_fallback_on_empty_arm_set_or_mismatch() {
    let dim = 4;
    let cfg = FcTsConfig {
        dim,
        ..Default::default()
    };
    let arm1 = FlowCorrectedThompsonBandit::new(cfg).expect("valid arm");

    // Empty arm set with empty profiles
    let empty_arm_set = FcTsArmSet { arms: vec![] };
    let empty_profile_names: Vec<String> = vec![];
    let context = vec![0.1; dim];
    let mut rng = deterministic_fc_ts_rng(42);

    let err_empty = select_profile_fc_ts(&empty_profile_names, &empty_arm_set, &context, &mut rng)
        .expect_err("should fail on empty arm set");
    assert!(matches!(err_empty, FcTsDispatchError::ArmSet(_)));

    // Length mismatch (2 profiles vs 1 arm)
    let mismatch_arm_set = FcTsArmSet { arms: vec![arm1] };
    let mismatch_profiles = vec!["p1".to_string(), "p2".to_string()];

    let err_mismatch =
        select_profile_fc_ts(&mismatch_profiles, &mismatch_arm_set, &context, &mut rng)
            .expect_err("should fail on profile/arm mismatch");
    assert!(matches!(
        err_mismatch,
        FcTsDispatchError::ProfileArmMismatch {
            profiles: 2,
            arms: 1
        }
    ));
}

#[cfg(all(feature = "flow-corrected-thompson", feature = "bandit-routing"))]
#[tokio::test]
async fn test_router_engine_fc_ts_dispatch_determinism_and_fallback() {
    use contextra_router::RoutingStrategy;

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
        0.8,
    );

    // Create two routers with FC-TS strategy and identical initial decision_id
    let router1 = create_mock_router(vec![p1.clone(), p2.clone()], dim)
        .with_initial_decision_id(100)
        .with_routing_strategy(RoutingStrategy::FlowCorrectedThompson, 0.1, 42);

    let router2 = create_mock_router(vec![p1.clone(), p2.clone()], dim)
        .with_initial_decision_id(100)
        .with_routing_strategy(RoutingStrategy::FlowCorrectedThompson, 0.1, 42);

    let embedding = vec![0.3f32; dim];

    let dec1 = router1.route(&embedding, "query 1").await.unwrap();
    let dec2 = router2.route(&embedding, "query 1").await.unwrap();

    assert_eq!(dec1.profile.name, dec2.profile.name);
    assert_eq!(dec1.decision_id.inner(), dec2.decision_id.inner());
}
