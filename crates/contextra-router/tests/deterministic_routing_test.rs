#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_router::{
    ArmRegistry, ArmRegistryError, DecisionIdGenerator, RoutingOutcome, RouterEngine, SlmProfile,
};
use contextra_types::{
    ContextChunk, ContextWindow, DocId, EntityId, Result, RetrievalStrategy, TokenBudget,
};
use std::sync::Arc;

struct TestSearchProvider {
    dimension: usize,
}

impl HybridSearchProvider for TestSearchProvider {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        query_embedding: &'a [f32],
        k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        let dim = self.dimension;
        let len = query_embedding.len();
        Box::pin(async move {
            if len != dim {
                return Err(contextra_types::ContextraError::InvalidInput(
                    "Dimension mismatch".into(),
                ));
            }
            let chunk = ContextChunk {
                doc_id: DocId::new(1),
                content: "Deterministic search chunk".to_string(),
                relevance: 0.9,
                token_count: 10,
                metadata: None,
                contextual_prefix: None,
                links: vec![],
            };
            Ok(vec![chunk; k.min(2)])
        })
    }
}

struct TestCommunityResolver;

impl CommunityResolver for TestCommunityResolver {
    fn get_community<'a>(&'a self, _entity: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { Ok(None) })
    }
}

struct TestContextPreparer;

impl ContextPreparer for TestContextPreparer {
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

fn create_engine(profiles: Vec<SlmProfile>, dim: usize) -> RouterEngine {
    let search = Arc::new(TestSearchProvider { dimension: dim });
    let comm = Arc::new(TestCommunityResolver);
    let prep = Arc::new(TestContextPreparer);
    RouterEngine::new(search, comm, prep, profiles, None)
}

#[tokio::test]
async fn test_deterministic_routing_sequence() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let dim = 4;
    let p1 = SlmProfile::new(
        "p1",
        "http://localhost:1111",
        vec![],
        TokenBudget::new(1000, 100),
        0.1,
    );
    let p2 = SlmProfile::new(
        "p2",
        "http://localhost:2222",
        vec![],
        TokenBudget::new(1000, 100),
        0.5,
    );

    let router1 = create_engine(vec![p1.clone(), p2.clone()], dim);
    let router2 = create_engine(vec![p1, p2], dim);

    let embedding = vec![0.1f32, 0.2f32, 0.3f32, 0.4f32];
    let query_text = "test query for deterministic sequence";

    let (dec1, ticket1) = router1.route_tracked(&embedding, query_text).await?;
    let (dec2, ticket2) = router2.route_tracked(&embedding, query_text).await?;

    assert_eq!(dec1.profile.name, dec2.profile.name);
    assert_eq!(dec1.decision_id, dec2.decision_id);
    assert_eq!(ticket1.decision_id(), ticket2.decision_id());
    assert_eq!(dec1.confidence, dec2.confidence);

    assert!(router1.complete(ticket1, RoutingOutcome::Success));
    assert!(router2.complete(ticket2, RoutingOutcome::Success));

    assert_eq!(
        router1.admin().calibration_stats(),
        router2.admin().calibration_stats()
    );

    Ok(())
}

#[test]
fn test_arm_registry_and_candidate_functions() {
    let registry = ArmRegistry::default();

    assert_eq!(
        registry.strategy_for(0).unwrap(),
        RetrievalStrategy::Vector
    );
    assert_eq!(registry.arm_for(RetrievalStrategy::Vector), 0);

    assert_eq!(
        registry.strategy_for(99),
        Err(ArmRegistryError::OutOfRange(99))
    );

    let gen = DecisionIdGenerator::new(42);
    let id1 = gen.next();
    assert_eq!(id1.inner(), 42);
}
