//! Criterion benchmarks for contextra-durable-fs filesystem probing (DF-06, D-30).

#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_durable_fs::probe::probe_dir;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::Path;
use std::time::Instant;
use tempfile::tempdir;

// ANCHOR[contextra-durable-fs:BENCH-PROBE-STARTUP-001]
// Prüft die Latenz der vollständigen Startprobe (`probe_dir`) zur Systeminitialisierung.
fn bm_dfs_probe_startup(c: &mut Criterion) {
    // Misst die Latenz der vollständigen Startprobe (probe_dir).
    let dir = tempdir().expect("tempdir");
    let mut group = c.benchmark_group("BM-DFS-PROBE-STARTUP");

    group.bench_function("probe_dir_latency", |b| {
        b.iter(|| {
            let probe = probe_dir(black_box(dir.path())).expect("probe_dir");
            black_box(probe);
        });
    });

    group.finish();
}

// ANCHOR[contextra-durable-fs:BENCH-FSYNC-P50-001]
// Misst die Stabilität der fsync_p50-Stichprobenmessung und schreibt den JSON-Benchmark-Report.
fn bm_dfs_fsync_p50(c: &mut Criterion) {
    // Misst die Stabilität der fsync_p50-Stichprobe und schreibt den Benchmark-Report.
    let dir = tempdir().expect("tempdir");
    let mut group = c.benchmark_group("BM-DFS-FSYNC-P50");

    group.bench_function("fsync_p50_sample_stability", |b| {
        b.iter(|| {
            let probe = probe_dir(black_box(dir.path())).expect("probe_dir");
            black_box(probe.fsync_p50);
        });
    });

    group.finish();

    // Dynamically measure actual metrics and write real bench report JSON
    write_real_bench_report(dir.path());
}

// ANCHOR[contextra-durable-fs:BENCH-WRITE-REAL-REPORT-001]
// Schreibt den strukturierten Benchmark-Report mit p50-, p99- und Durchsatzwerten nach target/bench-report/contextra-durable-fs.json.
fn write_real_bench_report(dir: &Path) {
    // NOTE: Hilfsfunktion zum Generieren der JSON-Benchmark-Metriken.
    let mut sample_latencies_us = Vec::with_capacity(20);
    let mut sample_fsync_us = Vec::with_capacity(20);

    for _ in 0..20 {
        let t0 = Instant::now();
        if let Ok(probe) = probe_dir(dir) {
            let elapsed_us = t0.elapsed().as_micros() as u64;
            sample_latencies_us.push(elapsed_us);
            sample_fsync_us.push(probe.fsync_p50.as_micros() as u64);
        }
    }

    sample_latencies_us.sort();
    sample_fsync_us.sort();

    let p50_startup = sample_latencies_us
        .get(sample_latencies_us.len() / 2)
        .copied()
        .unwrap_or(1000);
    let p99_startup = sample_latencies_us
        .get(sample_latencies_us.len().saturating_sub(1))
        .copied()
        .unwrap_or(p50_startup);
    let p50_fsync = sample_fsync_us
        .get(sample_fsync_us.len() / 2)
        .copied()
        .unwrap_or(100);
    let p99_fsync = sample_fsync_us
        .get(sample_fsync_us.len().saturating_sub(1))
        .copied()
        .unwrap_or(p50_fsync);

    let throughput = if p50_startup > 0 {
        1_000_000 / p50_startup
    } else {
        1000
    };

    let json_content = format!(
        r#"{{
  "crate": "contextra-durable-fs",
  "benchmarks": [
    {{
      "name": "BM-DFS-PROBE-STARTUP",
      "p50_us": {},
      "p99_us": {},
      "throughput_ops_per_sec": {}
    }},
    {{
      "name": "BM-DFS-FSYNC-P50",
      "p50_us": {},
      "p99_us": {},
      "sample_count": {}
    }}
  ]
}}"#,
        p50_startup,
        p99_startup,
        throughput,
        p50_fsync,
        p99_fsync,
        sample_fsync_us.len()
    );

    // Write to both workspace root and local crate target
    let root_report_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/bench-report");
    let _ = fs::create_dir_all(&root_report_dir);
    let _ = fs::write(
        root_report_dir.join("contextra-durable-fs.json"),
        &json_content,
    );

    let local_report_dir = Path::new("target/bench-report");
    let _ = fs::create_dir_all(local_report_dir);
    let _ = fs::write(
        local_report_dir.join("contextra-durable-fs.json"),
        &json_content,
    );
}

criterion_group!(benches, bm_dfs_probe_startup, bm_dfs_fsync_p50);
criterion_main!(benches);
