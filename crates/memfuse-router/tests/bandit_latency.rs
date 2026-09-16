#![cfg(feature = "bandit-routing")]

use memfuse_router::{BanditImplementation, BanditProfileState};
use std::time::Instant;

const FEATURE_DIM: usize = 768; // nomic-embed-text standard dimension
const BENCHMARK_ITERATIONS: usize = 1000;
const LATENCY_BUDGET_P95_US: u64 = 1000; // 1.0 ms

#[test]
fn test_bandit_diagonal_vs_linucb_latency_budget() {
    let cost = 0.2f32;
    let is_cloud = false;
    let reward = 0.8f32;
    let x = vec![0.5f32; FEATURE_DIM];

    // 1. Benchmark Diagonal Approximation
    let mut diag_state = BanditProfileState::cold_start(FEATURE_DIM, 0.5);
    diag_state.implementation = BanditImplementation::DiagonalApproximation;

    // Warmup
    for _ in 0..10 {
        let _ = diag_state.score(&x, cost, is_cloud);
        diag_state.update(&x, reward, cost, is_cloud);
    }

    let mut diag_latencies: Vec<u64> = Vec::with_capacity(BENCHMARK_ITERATIONS);
    for _ in 0..BENCHMARK_ITERATIONS {
        let start = Instant::now();
        let _s = diag_state.score(&x, cost, is_cloud);
        diag_state.update(&x, reward, cost, is_cloud);
        diag_latencies.push(start.elapsed().as_micros() as u64);
    }
    diag_latencies.sort_unstable();
    let diag_p95 = diag_latencies[(BENCHMARK_ITERATIONS as f64 * 0.95) as usize];

    println!(
        "[BENCHMARK] Diagonal Approximation P95 Latency (d={}): {} µs",
        FEATURE_DIM, diag_p95
    );
    assert!(
        diag_p95 <= LATENCY_BUDGET_P95_US,
        "Diagonal P95 latency ({} µs) exceeds budget ({} µs)",
        diag_p95,
        LATENCY_BUDGET_P95_US
    );

    // 2. Benchmark Sherman-Morrison (if feature egress-sherman-morrison active)
    #[cfg(feature = "egress-sherman-morrison")]
    {
        let mut sm_state = BanditProfileState::cold_start(FEATURE_DIM, 0.5);
        sm_state.implementation = BanditImplementation::ShermanMorrison;

        // Warmup
        for _ in 0..10 {
            let _ = sm_state.score(&x, cost, is_cloud);
            sm_state.update(&x, reward, cost, is_cloud);
        }

        let mut sm_latencies: Vec<u64> = Vec::with_capacity(BENCHMARK_ITERATIONS);
        for _ in 0..BENCHMARK_ITERATIONS {
            let start = Instant::now();
            let _s = sm_state.score(&x, cost, is_cloud);
            sm_state.update(&x, reward, cost, is_cloud);
            sm_latencies.push(start.elapsed().as_micros() as u64);
        }
        sm_latencies.sort_unstable();
        let sm_p95 = sm_latencies[(BENCHMARK_ITERATIONS as f64 * 0.95) as usize];

        println!(
            "[BENCHMARK] Sherman-Morrison P95 Latency (d={}): {} µs",
            FEATURE_DIM, sm_p95
        );
    }
}
