//! CI Gate Modul: Prüft das Latenzbudget des Bandit-Routers.
//! Definiertes Budget: 1.0ms (1000 µs) P95 Decision + Update Berechnung.

use memfuse_router::BanditProfileState;
use std::time::Instant;

/// Maximale zugelassene P95 Decision + Update Latenz in Microsekunden (1.0 ms = 1000 µs).
/// Großzügig bemessen, um Noisy-Neighbor-Flakiness auf Shared-CI-Runnern zu verhindern.
pub const BANDIT_LATENCY_BUDGET_P95_US: u64 = 1000;

/// Anzahl an Benchmark-Iterationen für die Perzentil-Berechnung.
const BENCHMARK_ITERATIONS: usize = 1000;

/// Vektor-Dimension für Kontext-Embeddings im Benchmark (Standard: nomic-embed-text d=768).
const FEATURE_DIM: usize = 768;

pub fn check_bandit_latency_budget() -> Result<(), String> {
    println!("=== Gate: Check Bandit Latency Budget ===");

    let mut profile_state = BanditProfileState::cold_start(FEATURE_DIM, 0.5);
    let x = vec![0.5f32; FEATURE_DIM];
    let cost = 0.2f32;
    let is_cloud = false;
    let reward = 0.8f32;

    let mut latencies_us: Vec<u64> = Vec::with_capacity(BENCHMARK_ITERATIONS);

    // Warmup
    for _ in 0..10 {
        let _ = profile_state.score(&x, cost, is_cloud);
        let _ = profile_state.update(&x, reward, cost, is_cloud);
    }

    // Measurement Loop
    for _ in 0..BENCHMARK_ITERATIONS {
        let start = Instant::now();
        let _score = profile_state.score(&x, cost, is_cloud);
        let _ = profile_state.update(&x, reward, cost, is_cloud);
        let elapsed_us = start.elapsed().as_micros() as u64;
        latencies_us.push(elapsed_us);
    }

    latencies_us.sort_unstable();

    // P95 Perzentil-Berechnung
    let p95_idx = (BENCHMARK_ITERATIONS as f64 * 0.95) as usize;
    let p95_latency_us = latencies_us[p95_idx.min(BENCHMARK_ITERATIONS - 1)];

    println!(
        "Bandit Decision + Update Latency (P95 über {} Iterationen): {} µs (Budget: {} µs)",
        BENCHMARK_ITERATIONS, p95_latency_us, BANDIT_LATENCY_BUDGET_P95_US
    );

    if p95_latency_us <= BANDIT_LATENCY_BUDGET_P95_US {
        println!("✅ Gate passed: Bandit latency is within budget.");
        Ok(())
    } else {
        let err_msg = format!(
            "❌ Gate failed: Bandit P95 decision latency ({} µs) exceeds budget ({} µs)",
            p95_latency_us, BANDIT_LATENCY_BUDGET_P95_US
        );
        eprintln!("{}", err_msg);
        Err(err_msg)
    }
}
