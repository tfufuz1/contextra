#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

//! Benchmark to quantify simulated cost savings of cost-aware SLM routing vs. a static single-model baseline.
//!
//! # Limitation Notice (Ehrlichkeit der Kennzahl)
//! This benchmark evaluates cost savings using a greedy selection proxy based on `SlmProfile::resource_cost_estimate`
//! and `SlmProfile::min_relevance_score`. Full LinUCB contextual bandit dispatch requires an active `RouterEngine`
//! runtime backed by hybrid search and community resolution components.
//! Therefore, this benchmark explicitly uses the greedy cost-aware selection proxy to quantify cost benefits
//! on a synthetic query distribution (70% simple / 25% medium / 5% complex queries across 1,000 queries)
//! without introducing heavy database test dependencies into benchmark execution.

use contextra_router::SlmProfile;
use contextra_types::TokenBudget;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Simple deterministic LCG PRNG for seed-based query generation (AGENTS.md §6).
struct SeededRng {
    state: u64,
}

impl SeededRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 32) as u32 as f32) / (u32::MAX as f32)
    }
}

/// Simulated query with a required context relevance score.
#[derive(Debug, Clone, Copy)]
struct SimulatedQuery {
    id: usize,
    context_relevance: f32,
}

/// Generates 1,000 synthetic queries with realistic frequency distribution:
/// - 70% simple queries (relevance requirement 0.50..0.74)
/// - 25% medium queries (relevance requirement 0.75..0.89)
/// - 5% complex queries (relevance requirement 0.90..0.99)
fn generate_synthetic_queries(count: usize) -> Vec<SimulatedQuery> {
    let mut rng = SeededRng::new(42);
    let mut queries = Vec::with_capacity(count);

    for id in 0..count {
        let roll = rng.next_f32();
        let context_relevance = if roll < 0.70 {
            0.50 + rng.next_f32() * 0.24
        } else if roll < 0.95 {
            0.75 + rng.next_f32() * 0.14
        } else {
            0.90 + rng.next_f32() * 0.09
        };

        queries.push(SimulatedQuery {
            id,
            context_relevance,
        });
    }

    queries
}

/// Group 2a: Baseline routing where every request is routed to the single most expensive profile.
fn run_static_single_model_routing(queries: &[SimulatedQuery], large_profile: &SlmProfile) -> f32 {
    let mut total_cost = 0.0f32;
    for _query in queries {
        total_cost += large_profile.estimated_cost();
    }
    total_cost
}

/// Group 2b: Cost-aware routing selecting the lowest-cost profile satisfying the relevance score requirement.
fn run_cost_aware_routing(queries: &[SimulatedQuery], profiles: &[SlmProfile]) -> f32 {
    let mut total_cost = 0.0f32;
    for query in queries {
        let selected = profiles
            .iter()
            .filter(|p| p.min_relevance_score <= query.context_relevance)
            .min_by(|a, b| {
                a.estimated_cost()
                    .total_cmp(&b.estimated_cost())
                    .then_with(|| a.min_relevance_score.total_cmp(&b.min_relevance_score))
            })
            .unwrap_or_else(|| {
                profiles
                    .iter()
                    .min_by(|a, b| a.estimated_cost().total_cmp(&b.estimated_cost()))
                    .expect("profiles list is non-empty")
            });

        total_cost += selected.estimated_cost();
    }
    total_cost
}

fn bench_cost_routing_savings(c: &mut Criterion) {
    let queries = generate_synthetic_queries(1000);

    let profile_small = SlmProfile::new(
        "small-slm",
        "http://127.0.0.1:9090/mcp/small",
        vec![1],
        TokenBudget::new(2048, 256),
        0.50,
    )
    .with_resource_cost_estimate(0.002);

    let profile_medium = SlmProfile::new(
        "medium-slm",
        "http://127.0.0.1:9090/mcp/medium",
        vec![1],
        TokenBudget::new(4096, 512),
        0.75,
    )
    .with_resource_cost_estimate(0.010);

    let profile_large = SlmProfile::new(
        "large-slm",
        "http://127.0.0.1:9090/mcp/large",
        vec![1],
        TokenBudget::new(8192, 1024),
        0.90,
    )
    .with_resource_cost_estimate(0.050);

    let profiles = vec![profile_small, profile_medium, profile_large.clone()];

    let static_cost = run_static_single_model_routing(&queries, &profile_large);
    let cost_aware_cost = run_cost_aware_routing(&queries, &profiles);
    let savings_pct = ((static_cost - cost_aware_cost) / static_cost) * 100.0;

    println!("\n=======================================================");
    println!("COST ROUTING SAVINGS EVALUATION REPORT");
    println!("-------------------------------------------------------");
    println!("Queries evaluated:             {}", queries.len());
    println!("Static Single Model Cost:      {:.4}", static_cost);
    println!("Cost-Aware Routing Cost:       {:.4}", cost_aware_cost);
    println!("Cost Savings:                  {:.2}%", savings_pct);
    println!("=======================================================\n");

    let mut group = c.benchmark_group("cost_routing_savings");

    group.bench_function("static_single_model_routing", |b| {
        b.iter(|| {
            let cost =
                run_static_single_model_routing(black_box(&queries), black_box(&profile_large));
            black_box(cost)
        });
    });

    group.bench_function("cost_aware_routing", |b| {
        b.iter(|| {
            let cost = run_cost_aware_routing(black_box(&queries), black_box(&profiles));
            black_box(cost)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_cost_routing_savings);
criterion_main!(benches);
