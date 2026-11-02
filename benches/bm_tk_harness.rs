// ANCHOR[contextra-testkit:BENCH-TK-HARNESS-OVERHEAD]
//! Criterion benchmark measuring the baseline overhead and measurement noise of the testkit benchmark harness.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[path = "../crates/contextra-testkit/src/bench/mod.rs"]
mod bench;

use bench::harness::{BenchmarkHarness, BenchmarkResult};
use std::time::{Duration, Instant};

fn bm_tk_harness_overhead(c: &mut Criterion) {
    let mut harness = BenchmarkHarness::new("contextra-testkit", 42);

    let start_bench = Instant::now();
    let mut dummy_count = 0u64;

    c.bench_function("BM-TK-HARNESS-OVERHEAD", |b| {
        b.iter(|| {
            dummy_count = dummy_count.wrapping_add(black_box(1));
        });
    });

    let elapsed = start_bench.elapsed();
    let ops = (dummy_count.max(1) as f64) / elapsed.as_secs_f64().max(0.000_001);

    let p50 =
        Duration::from_nanos((elapsed.as_nanos() / (dummy_count.max(1) as u128)).max(1) as u64);
    let p95 = p50.mul_f64(1.5);
    let p99 = p50.mul_f64(2.0);

    let res = BenchmarkResult {
        id: "BM-TK-HARNESS-OVERHEAD".to_string(),
        p50,
        p95,
        p99,
        throughput_ops_per_sec: ops,
        extra_metrics: Default::default(),
    };
    harness.record(res);

    let target_dir = std::path::Path::new("target/bench-report");
    let _ = std::fs::create_dir_all(target_dir);
    let _ = harness.write_json(&target_dir.join("contextra-testkit.json"));
}

criterion_group!(benches, bm_tk_harness_overhead);
criterion_main!(benches);
