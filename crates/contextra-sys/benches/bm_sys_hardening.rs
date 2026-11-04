// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Benchmarks BM-SYS-LOCK-PAGES and BM-SYS-APPLY-HARDENING for contextra-sys
// INVARIANTEN: Deterministic seed, writes JSON report to target/bench-report/contextra-sys.json

#![allow(clippy::disallowed_methods)]

use std::fs;
use std::path::Path;
use std::time::Instant;

use contextra_sys::{apply_process_hardening, lock_pages, HardeningPolicy};

fn calculate_percentiles(mut samples_ns: Vec<f64>) -> (f64, f64, f64) {
    samples_ns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if samples_ns.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let len = samples_ns.len() as f64;
    let p50_idx = ((len * 0.50).floor() as usize).min(samples_ns.len() - 1);
    let p95_idx = ((len * 0.95).floor() as usize).min(samples_ns.len() - 1);
    let p99_idx = ((len * 0.99).floor() as usize).min(samples_ns.len() - 1);
    (
        samples_ns.get(p50_idx).copied().unwrap_or(0.0),
        samples_ns.get(p95_idx).copied().unwrap_or(0.0),
        samples_ns.get(p99_idx).copied().unwrap_or(0.0),
    )
}

fn main() {
    println!("Running BM-SYS-LOCK-PAGES and BM-SYS-APPLY-HARDENING...");

    // 1. BM-SYS-APPLY-HARDENING
    let iters_apply = 1_000;
    let mut apply_samples = Vec::with_capacity(iters_apply);
    let start_apply_total = Instant::now();
    for _ in 0..iters_apply {
        let t0 = Instant::now();
        let _ = apply_process_hardening(HardeningPolicy::BestEffort);
        apply_samples.push(t0.elapsed().as_nanos() as f64);
    }
    let elapsed_apply = start_apply_total.elapsed();
    let avg_apply_ns = elapsed_apply.as_nanos() as f64 / iters_apply as f64;
    let (p50_apply, p95_apply, p99_apply) = calculate_percentiles(apply_samples);

    // 2. BM-SYS-LOCK-PAGES (per MiB cost)
    let mib_size = 1024 * 1024;
    let test_buf = vec![0x42u8; mib_size];
    let iters_lock = 500;
    let mut lock_samples = Vec::with_capacity(iters_lock);
    let start_lock_total = Instant::now();
    for _ in 0..iters_lock {
        let t0 = Instant::now();
        let _ = lock_pages(&test_buf, HardeningPolicy::BestEffort);
        lock_samples.push(t0.elapsed().as_nanos() as f64);
    }
    let elapsed_lock = start_lock_total.elapsed();
    let avg_lock_per_mib_ns = elapsed_lock.as_nanos() as f64 / iters_lock as f64;
    let (p50_lock, p95_lock, p99_lock) = calculate_percentiles(lock_samples);

    let report_dir = Path::new("target/bench-report");
    let _ = fs::create_dir_all(report_dir);
    let report_file = report_dir.join("contextra-sys.json");

    let json_content = format!(
        r#"{{
  "crate": "contextra-sys",
  "benchmarks": [
    {{
      "name": "BM-SYS-APPLY-HARDENING",
      "iterations": {},
      "total_time_ms": {:.3},
      "avg_latency_ns": {:.2},
      "p50_latency_ns": {:.2},
      "p95_latency_ns": {:.2},
      "p99_latency_ns": {:.2}
    }},
    {{
      "name": "BM-SYS-LOCK-PAGES",
      "iterations": {},
      "size_bytes": {},
      "total_time_ms": {:.3},
      "avg_latency_per_mib_ns": {:.2},
      "p50_latency_ns": {:.2},
      "p95_latency_ns": {:.2},
      "p99_latency_ns": {:.2}
    }}
  ]
}}"#,
        iters_apply,
        elapsed_apply.as_secs_f64() * 1000.0,
        avg_apply_ns,
        p50_apply,
        p95_apply,
        p99_apply,
        iters_lock,
        mib_size,
        elapsed_lock.as_secs_f64() * 1000.0,
        avg_lock_per_mib_ns,
        p50_lock,
        p95_lock,
        p99_lock
    );

    let _ = fs::write(&report_file, &json_content);
    println!("Benchmark report written to {}", report_file.display());
    println!("{}", json_content);
}
