//! # Benchmark-Suite: Atomarer Austausch und Scrubbing (`benches/bm_dfs_atomic.rs`)
//!
//! Zweck: Messung von Durchsatz und Latenz-Perzentilen für Heißpfad-Operationen:
//! - `BM-DFS-WRITE-ATOMIC`: 4 fsyncs im Normalpfad (Beitrag zu BM-ERASE-LAT).
//! - `BM-ERASE-WA`: Scrubbing-Kosten getrennt vor dem Unlink.
//!
//! Schreibt deterministische JSON-Metriken nach `target/bench-report/contextra-durable-fs.json`.

#![allow(
    dead_code,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::disallowed_methods,
    clippy::expect_used,
    clippy::unwrap_used
)]

#[path = "../src/atomic.rs"]
mod atomic;

use atomic::write_atomic;
use contextra_durable_fs::scrub::{scrub_before_unlink, ScrubOutcome};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use tempfile::TempDir;

#[derive(Debug, Clone)]
struct BenchResult {
    name: String,
    iterations: usize,
    total_bytes: u64,
    throughput_mb_s: f64,
    latency_p50_us: f64,
    latency_p90_us: f64,
    latency_p95_us: f64,
    latency_p99_us: f64,
}

fn compute_percentile(sorted_latencies: &[f64], pct: f64) -> f64 {
    if sorted_latencies.is_empty() {
        return 0.0;
    }
    let idx = ((sorted_latencies.len() as f64) * pct).floor() as usize;
    let clamped_idx = idx.min(sorted_latencies.len().saturating_sub(1));
    *sorted_latencies.get(clamped_idx).unwrap_or(&0.0)
}

fn bench_write_atomic(temp_dir: &Path, iterations: usize) -> BenchResult {
    let payload_size = 64 * 1024; // 64 KiB
    let payload = vec![0x5au8; payload_size];
    let mut latencies_us = Vec::with_capacity(iterations);
    let start_total = Instant::now();

    for i in 0..iterations {
        let file_path = temp_dir.join(format!("bench_atomic_{i}.dat"));
        let start = Instant::now();
        write_atomic(&file_path, &payload).expect("write_atomic in bench failed");
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.0;
        latencies_us.push(elapsed);
        let _ = fs::remove_file(&file_path);
    }

    let total_duration_secs = start_total.elapsed().as_secs_f64();
    let total_bytes = (payload_size * iterations) as u64;
    let throughput_mb_s = (total_bytes as f64 / 1_048_576.0) / total_duration_secs;

    latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    BenchResult {
        name: "BM-DFS-WRITE-ATOMIC".to_string(),
        iterations,
        total_bytes,
        throughput_mb_s,
        latency_p50_us: compute_percentile(&latencies_us, 0.50),
        latency_p90_us: compute_percentile(&latencies_us, 0.90),
        latency_p95_us: compute_percentile(&latencies_us, 0.95),
        latency_p99_us: compute_percentile(&latencies_us, 0.99),
    }
}

fn bench_erase_wa(temp_dir: &Path, iterations: usize) -> BenchResult {
    let file_size = 64 * 1024; // 64 KiB
    let dummy_data = vec![0xa5u8; file_size];
    let mut latencies_us = Vec::with_capacity(iterations);
    let start_total = Instant::now();

    for i in 0..iterations {
        let file_path = temp_dir.join(format!("bench_scrub_{i}.dat"));
        fs::write(&file_path, &dummy_data).expect("Initial write failed");

        let start = Instant::now();
        // D-83: rotational == true && !is_cow misst die getrennten Scrubbing-Kosten
        let outcome = scrub_before_unlink(&file_path, true, false).expect("scrub failed");
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.0;
        latencies_us.push(elapsed);

        assert!(matches!(outcome, ScrubOutcome::Scrubbed(_)));
        let _ = fs::remove_file(&file_path);
    }

    let total_duration_secs = start_total.elapsed().as_secs_f64();
    let total_bytes = (file_size * iterations) as u64;
    let throughput_mb_s = (total_bytes as f64 / 1_048_576.0) / total_duration_secs;

    latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    BenchResult {
        name: "BM-ERASE-WA".to_string(),
        iterations,
        total_bytes,
        throughput_mb_s,
        latency_p50_us: compute_percentile(&latencies_us, 0.50),
        latency_p90_us: compute_percentile(&latencies_us, 0.90),
        latency_p95_us: compute_percentile(&latencies_us, 0.95),
        latency_p99_us: compute_percentile(&latencies_us, 0.99),
    }
}

fn write_json_report(results: &[BenchResult], out_paths: &[&Path]) {
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str("  \"crate\": \"contextra-durable-fs\",\n");
    json.push_str("  \"benchmarks\": [\n");

    for (idx, r) in results.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!("      \"name\": \"{}\",\n", r.name));
        json.push_str(&format!("      \"iterations\": {},\n", r.iterations));
        json.push_str(&format!("      \"total_bytes\": {},\n", r.total_bytes));
        json.push_str(&format!(
            "      \"throughput_mb_s\": {:.2},\n",
            r.throughput_mb_s
        ));
        json.push_str(&format!(
            "      \"latency_p50_us\": {:.2},\n",
            r.latency_p50_us
        ));
        json.push_str(&format!(
            "      \"latency_p90_us\": {:.2},\n",
            r.latency_p90_us
        ));
        json.push_str(&format!(
            "      \"latency_p95_us\": {:.2},\n",
            r.latency_p95_us
        ));
        json.push_str(&format!(
            "      \"latency_p99_us\": {:.2}\n",
            r.latency_p99_us
        ));
        if idx + 1 < results.len() {
            json.push_str("    },\n");
        } else {
            json.push_str("    }\n");
        }
    }

    json.push_str("  ]\n");
    json.push_str("}\n");

    for out_path in out_paths {
        if let Some(parent) = out_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = File::create(out_path) {
            let _ = file.write_all(json.as_bytes());
        }
    }
}

fn run_benchmarks() {
    let temp = TempDir::new().expect("TempDir failed");
    let iterations = 50;

    println!("Running benchmark BM-DFS-WRITE-ATOMIC ({iterations} iterations)...");
    let res_atomic = bench_write_atomic(temp.path(), iterations);
    println!(
        "  BM-DFS-WRITE-ATOMIC: {:.2} MB/s | p50: {:.2} µs | p95: {:.2} µs | p99: {:.2} µs",
        res_atomic.throughput_mb_s,
        res_atomic.latency_p50_us,
        res_atomic.latency_p95_us,
        res_atomic.latency_p99_us
    );

    println!("Running benchmark BM-ERASE-WA ({iterations} iterations)...");
    let res_erase_wa = bench_erase_wa(temp.path(), iterations);
    println!(
        "  BM-ERASE-WA: {:.2} MB/s | p50: {:.2} µs | p95: {:.2} µs | p99: {:.2} µs",
        res_erase_wa.throughput_mb_s,
        res_erase_wa.latency_p50_us,
        res_erase_wa.latency_p95_us,
        res_erase_wa.latency_p99_us
    );

    let report_paths = [
        Path::new("target/bench-report/contextra-durable-fs.json"),
        Path::new("../../target/bench-report/contextra-durable-fs.json"),
    ];
    write_json_report(&[res_atomic, res_erase_wa], &report_paths);
    println!(
        "Benchmark report successfully written to target/bench-report/contextra-durable-fs.json"
    );
}

#[test]
fn bm_dfs_benchmarks() {
    run_benchmarks();
}

fn main() {
    run_benchmarks();
}
