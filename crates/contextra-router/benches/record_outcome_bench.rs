//! Micro-benchmark for RouterEngine record_outcome with Arc<BanditProfileState> COW at d=384 and d=768.

#![cfg(feature = "bandit-routing")]

use contextra_adapt::BanditProfileState;
use contextra_router::{
    outcome::RoutingOutcome, profile::SlmProfile, router::RouterEngine,
};
use contextra_types::TokenBudget;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;

struct MockSearchProvider;
#[async_trait::async_trait]
impl contextra_ports::HybridSearchProvider for MockSearchProvider {
    async fn search_hybrid(
        &self,
        _query_text: &str,
        _query_embedding: &[f32],
        _limit: usize,
    ) -> contextra_types::Result<Vec<contextra_types::ContextChunk>> {
        Ok(vec![])
    }
}

struct MockCommunityResolver;
#[async_trait::async_trait]
impl contextra_ports::CommunityResolver for MockCommunityResolver {
    async fn get_community(
        &self,
        _entity_id: contextra_types::EntityId,
    ) -> contextra_types::Result<Option<u64>> {
        Ok(None)
    }
}

struct MockContextPreparer;
impl contextra_ports::ContextPreparer for MockContextPreparer {
    fn prepare_context(
        &self,
        _chunks: Vec<contextra_types::ContextChunk>,
        _token_budget: &TokenBudget,
        _min_relevance_score: f32,
    ) -> contextra_types::Result<contextra_types::ContextWindow> {
        Ok(contextra_types::ContextWindow::default())
    }
}

pub fn bench_record_outcome(c: &mut Criterion) {
    let mut group = c.benchmark_group("record_outcome_cow");

    for d in [384, 768] {
        group.bench_with_input(BenchmarkId::new("dimension", d), &d, |b, &dim| {
            let mut p1 = SlmProfile::new(
                "p1",
                "http://localhost:8000/mcp",
                vec![],
                TokenBudget::new(1000, 100),
                0.1,
            );
            p1.bandit_state = Some(Arc::new(BanditProfileState::cold_start(dim, 0.5)));

            let mut p2 = SlmProfile::new(
                "p2",
                "http://localhost:8000/mcp",
                vec![],
                TokenBudget::new(1000, 100),
                0.1,
            );
            p2.bandit_state = Some(Arc::new(BanditProfileState::cold_start(dim, 0.5)));

            let router = RouterEngine::new(
                Arc::new(MockSearchProvider),
                Arc::new(MockCommunityResolver),
                Arc::new(MockContextPreparer),
                vec![p1, p2],
            )
            .expect("RouterEngine initialization must succeed");

            b.iter(|| {
                let decision_id = router.decision_ids.next();
                router.pending_decisions.write().insert(
                    decision_id,
                    ("p1".to_string(), std::time::Instant::now()),
                );
                router.pending_bandit.write().insert(
                    decision_id,
                    contextra_router::router::PendingBanditDecision {
                        context: vec![0.1f32; dim],
                        profile_name: "p1".to_string(),
                        action_idx: 0,
                        propensity: 0.5,
                        created: std::time::Instant::now(),
                    },
                );

                let outcome = RoutingOutcome::Success {
                    latency_ms: 50.0,
                    quality_score: 0.9,
                };
                let processed = router.record_outcome(decision_id, outcome);
                assert!(processed);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_record_outcome);
criterion_main!(benches);
