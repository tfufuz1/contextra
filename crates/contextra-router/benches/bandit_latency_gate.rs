#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[cfg(feature = "bandit-routing")]
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(feature = "bandit-routing")]
use contextra_router::{BanditImplementation, BanditProfileState};

/// Standard-Dimension für Kontext-Embeddings (nomic-embed-text d=768).
#[cfg(feature = "bandit-routing")]
const FEATURE_DIM: usize = 768;

#[cfg(feature = "bandit-routing")]
fn bench_bandit_latency_gate(c: &mut Criterion) {
    let x = vec![0.5f32; FEATURE_DIM];
    let cost = 0.2f32;
    let is_cloud = false;
    let reward = 0.8f32;

    let mut group = c.benchmark_group("bandit_latency_gate");

    // 1. Benchmark Sherman-Morrison O(d²) — Produktions-Default
    group.bench_function("sherman_morrison_d768", |b| {
        let mut state = BanditProfileState::cold_start(FEATURE_DIM, 0.5);
        state.implementation = BanditImplementation::ShermanMorrison;

        b.iter(|| {
            let score = state.score(black_box(&x), black_box(cost), black_box(is_cloud));
            let _ = state.update(
                black_box(&x),
                black_box(reward),
                black_box(cost),
                black_box(is_cloud),
            );
            black_box(score)
        });
    });

    // 2. Benchmark Diagonal-Approximation O(d) — Opt-in
    group.bench_function("diagonal_approximation_d768", |b| {
        let mut state = BanditProfileState::cold_start(FEATURE_DIM, 0.5);
        state.implementation = BanditImplementation::DiagonalApproximation;

        b.iter(|| {
            let score = state.score(black_box(&x), black_box(cost), black_box(is_cloud));
            let _ = state.update(
                black_box(&x),
                black_box(reward),
                black_box(cost),
                black_box(is_cloud),
            );
            black_box(score)
        });
    });

    group.finish();
}

#[cfg(feature = "bandit-routing")]
criterion_group!(benches, bench_bandit_latency_gate);
#[cfg(feature = "bandit-routing")]
criterion_main!(benches);

#[cfg(not(feature = "bandit-routing"))]
fn main() {
    println!("Benchmark `bandit_latency_gate` requires feature `bandit-routing`.");
}
