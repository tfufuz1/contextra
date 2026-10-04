// FILE-CONTEXT
// ZWECK: Verifiziert ADR-N16 Bereinigung — Routing und Bandit-Algorithmus funktionieren
// einwandfrei ohne das entfernte No-Op Feature `egress-sherman-morrison`.

#![cfg(feature = "bandit-routing")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_adapt::bandit::BanditProfileState;
use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_router::{RouterEngine, RoutingOutcome, RoutingStrategy, SlmProfile};
use contextra_types::{
    ContextChunk, ContextWindow, ContextraError, DocId, EntityId, Result, TokenBudget,
};
use std::sync::Arc;

struct MockSearchProvider;

impl HybridSearchProvider for MockSearchProvider {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        query_embedding: &'a [f32],
        k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        let dim = query_embedding.len();
        Box::pin(async move {
            if dim == 0 {
                return Err(ContextraError::InvalidInput("Empty query embedding".into()));
            }
            let chunk = ContextChunk {
                doc_id: DocId::new(42),
                content: "ADR-N16 validation chunk".to_string(),
                relevance: 0.9,
                token_count: 10,
                metadata: None,
                contextual_prefix: None,
                links: vec![],
            };
            Ok(vec![chunk; k.min(1)])
        })
    }
}

struct MockCommunityResolver;

impl CommunityResolver for MockCommunityResolver {
    fn get_community<'a>(&'a self, _entity: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { Ok(Some(1)) })
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

#[tokio::test]
async fn test_adr_n16_router_bandit_execution_without_noop_flags() {
    let dim = 4;
    let mut profile1 = SlmProfile::new(
        "fast-slm",
        "http://localhost:8080/slm1",
        vec![1],
        TokenBudget::new(1000, 100),
        0.6,
    )
    .with_resource_cost_estimate(0.05);
    profile1.bandit_state = Some(BanditProfileState::cold_start(dim, 0.5));

    let mut profile2 = SlmProfile::new(
        "accurate-slm",
        "http://localhost:8080/slm2",
        vec![1],
        TokenBudget::new(2000, 200),
        0.85,
    )
    .with_resource_cost_estimate(0.20);
    profile2.bandit_state = Some(BanditProfileState::cold_start(dim, 0.5));

    let search = Arc::new(MockSearchProvider);
    let comm = Arc::new(MockCommunityResolver);
    let prep = Arc::new(MockContextPreparer);

    let router = RouterEngine::new(search, comm, prep, vec![profile1, profile2], None)
        .with_routing_strategy(RoutingStrategy::ContextualBandit, 0.1, 12345);

    let embedding = vec![0.25f32; dim];
    let query = "Verify ADR-N16 no-op feature cleanup";

    // 1. Perform routing through ContextualBandit strategy
    let decision = router
        .route(&embedding, query)
        .await
        .expect("Routing should succeed under bandit-routing");

    assert!(!decision.profile.name.is_empty());
    assert!(decision.confidence.is_some());

    // 2. Verify propensity score retrieval before consuming decision
    let propensity = router.bandit_decision_propensity(decision.decision_id);
    assert!(propensity.is_some(), "Propensity should be available for pending decision");
    assert!(propensity.unwrap() > 0.0);

    // 3. Record outcome to verify bandit update
    let recorded = router.record_outcome(decision.decision_id, RoutingOutcome::Success);
    assert!(recorded, "Outcome should be recorded successfully");
}
