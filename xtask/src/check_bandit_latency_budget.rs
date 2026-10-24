//! CI Gate Modul: Prüft das Latenzbudget des Bandit-Routers.
//! Definiertes Budget: 1.0ms (1000 µs) P95 Decision + Update Berechnung.

use contextra_router::BanditProfileState;
use std::time::Instant;

/// Maximale zugelassene P95 Decision + Update Latenz in Microsekunden (1.0 ms = 1000 µs).
/// Großzügig bemessen, um Noisy-Neighbor-Flakiness auf Shared-CI-Runnern zu verhindern.
pub const BANDIT_LATENCY_BUDGET_P95_US: u64 = 1000;

/// Standard-Anzahl an Benchmark-Iterationen pro Lauf.
const DEFAULT_BENCHMARK_ITERATIONS: usize = 1000;

/// Standard-Anzahl an unabhängigen Messdurchläufen.
const DEFAULT_BENCHMARK_RUNS: usize = 5;

/// Anzahl verworfener Warm-up-Iterationen vor den Messungen.
const WARMUP_ITERATIONS: usize = 200;

/// Vektor-Dimension für Kontext-Embeddings im Benchmark (Standard: nomic-embed-text d=768).
const FEATURE_DIM: usize = 768;

/// Liest die Anzahl der Läufe aus `CONTEXTRA_BANDIT_LATENCY_RUNS` oder nutzt den Standardwert.
fn get_benchmark_runs() -> usize {
    std::env::var("CONTEXTRA_BANDIT_LATENCY_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&v| v > 0)
        .unwrap_or(DEFAULT_BENCHMARK_RUNS)
}

/// Liest die Anzahl der Iterationen pro Lauf aus `CONTEXTRA_BANDIT_LATENCY_ITERATIONS` oder nutzt den Standardwert.
fn get_benchmark_iterations() -> usize {
    std::env::var("CONTEXTRA_BANDIT_LATENCY_ITERATIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&v| v > 0)
        .unwrap_or(DEFAULT_BENCHMARK_ITERATIONS)
}

/// Berechnet das P95-Perzentil aus einer Reihe von Latenzwerten in Microsekunden.
pub fn calculate_p95(latencies_us: &[u64]) -> u64 {
    if latencies_us.is_empty() {
        return 0;
    }
    let mut sorted = latencies_us.to_vec();
    sorted.sort_unstable();
    let p95_idx = ((sorted.len() as f64) * 0.95) as usize;
    let idx = p95_idx.min(sorted.len() - 1);
    sorted[idx]
}

/// Berechnet den Median aus einer Reihe von P95-Werten.
pub fn calculate_median(values: &[u64]) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let len = sorted.len();
    if len % 2 == 1 {
        sorted[len / 2]
    } else {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2
    }
}

/// Evaluierte eine Liste von Einzel-P95-Laufwerten gegen ein Budget.
pub fn evaluate_p95_runs(p95_runs: &[u64], budget_us: u64) -> Result<u64, String> {
    let median_p95 = calculate_median(p95_runs);
    let runs_formatted: Vec<String> = p95_runs
        .iter()
        .enumerate()
        .map(|(i, &p)| format!("Lauf {}: {} µs", i + 1, p))
        .collect();

    if median_p95 <= budget_us {
        println!(
            "Bandit Decision + Update Latency (Median P95 über {} Läufe): {} µs (Budget: {} µs)",
            p95_runs.len(),
            median_p95,
            budget_us
        );
        println!("Einzelne P95-Läufe: [{}]", runs_formatted.join(", "));
        println!("✅ Gate passed: Bandit latency median P95 is within budget.");
        Ok(median_p95)
    } else {
        let err_msg = format!(
            "❌ Gate failed: Bandit median P95 decision latency ({} µs) exceeds budget ({} µs). Einzel-P95-Werte über {} Läufe: [{}]",
            median_p95, budget_us, p95_runs.len(), runs_formatted.join(", ")
        );
        eprintln!("{}", err_msg);
        Err(err_msg)
    }
}

pub fn check_bandit_latency_budget() -> Result<(), String> {
    println!("=== Gate: Check Bandit Latency Budget ===");

    let runs = get_benchmark_runs();
    let iterations = get_benchmark_iterations();

    let mut profile_state = BanditProfileState::cold_start(FEATURE_DIM, 0.5);
    profile_state.implementation = contextra_router::BanditImplementation::DiagonalApproximation;
    let x = vec![0.5f32; FEATURE_DIM];
    let cost = 0.2f32;
    let is_cloud = false;
    let reward = 0.8f32;

    // Warmup (mehrere hundert Iterationen vor der Messung, Ergebnisse verworfen)
    for _ in 0..WARMUP_ITERATIONS {
        let _ = profile_state.score(&x, cost, is_cloud);
        let _ = profile_state.update(&x, reward, cost, is_cloud);
    }

    let mut p95_runs = Vec::with_capacity(runs);

    // Multiple Measurement Loops
    for run in 0..runs {
        let mut latencies_us: Vec<u64> = Vec::with_capacity(iterations);
        for _ in 0..iterations {
            let start = Instant::now();
            let _score = profile_state.score(&x, cost, is_cloud);
            let _ = profile_state.update(&x, reward, cost, is_cloud);
            let elapsed_us = start.elapsed().as_micros() as u64;
            latencies_us.push(elapsed_us);
        }
        let p95 = calculate_p95(&latencies_us);
        p95_runs.push(p95);
        println!("Durchlauf {}/{}: P95 = {} µs", run + 1, runs, p95);
    }

    evaluate_p95_runs(&p95_runs, BANDIT_LATENCY_BUDGET_P95_US).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_p95() {
        let mut latencies: Vec<u64> = (1..=100).collect();
        // 95% index of 100 items is 95 -> index 95 is value 96
        assert_eq!(calculate_p95(&latencies), 96);

        latencies.reverse();
        assert_eq!(calculate_p95(&latencies), 96);
    }

    #[test]
    fn test_calculate_median() {
        assert_eq!(calculate_median(&[100, 200, 300, 400, 500]), 300);
        assert_eq!(calculate_median(&[500, 100, 400, 200, 300]), 300);
        assert_eq!(calculate_median(&[100, 200, 300, 400]), 250);
    }

    #[test]
    fn test_single_outlier_does_not_fail_gate() {
        let budget = 1000;
        // 5 runs with normal latencies ~100 µs, but run 3 has a huge spike (5000 µs Noisy Neighbor)
        let runs_with_outlier = vec![120, 110, 5000, 105, 115];

        let result = evaluate_p95_runs(&runs_with_outlier, budget);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 115); // Median is 115 µs
    }

    #[test]
    fn test_persistent_trend_fails_gate() {
        let budget = 1000;
        // Persistent regression across runs: median is 1200 µs
        let runs_regression = vec![1200, 1150, 1300, 1050, 1250];

        let result = evaluate_p95_runs(&runs_regression, budget);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Verify failure message contains all 5 individual P95 values
        assert!(err.contains("1200 µs"));
        assert!(err.contains("1150 µs"));
        assert!(err.contains("1300 µs"));
        assert!(err.contains("1050 µs"));
        assert!(err.contains("1250 µs"));
    }
}
