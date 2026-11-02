// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Criterion-Benchmark BM-SYS-FDATASYNC, BM-SYS-FSYNC, BM-SYS-RENAME-DURABLE für Latenz/Durchsatz (Regel 5, D-30)

use criterion::{criterion_group, criterion_main, Criterion};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

fn run_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("contextra_sys_durability");

    // BM-SYS-FDATASYNC
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let file_path = temp_dir.path().join("fdatasync.bench");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file_path)
        .expect("open file");

    let mut fdatasync_durations = Vec::new();
    group.bench_function("BM-SYS-FDATASYNC", |b| {
        b.iter(|| {
            let start = Instant::now();
            let _ = file.write_all(b"bench payload 4096 bytes");
            let _ = contextra_sys::durability::flush_file(
                &file,
                contextra_sys::durability::FlushKind::Data,
            );
            fdatasync_durations.push(start.elapsed().as_nanos() as u64);
        });
    });

    // BM-SYS-FSYNC
    let fsync_file_path = temp_dir.path().join("fsync.bench");
    let mut fsync_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&fsync_file_path)
        .expect("open file");

    let mut fsync_durations = Vec::new();
    group.bench_function("BM-SYS-FSYNC", |b| {
        b.iter(|| {
            let start = Instant::now();
            let _ = fsync_file.write_all(b"bench payload 4096 bytes");
            let _ = contextra_sys::durability::flush_file(
                &fsync_file,
                contextra_sys::durability::FlushKind::All,
            );
            fsync_durations.push(start.elapsed().as_nanos() as u64);
        });
    });

    // BM-SYS-RENAME-DURABLE
    let src = temp_dir.path().join("rename_src.bench");
    let dst = temp_dir.path().join("rename_dst.bench");

    let mut rename_durations = Vec::new();
    group.bench_function("BM-SYS-RENAME-DURABLE", |b| {
        b.iter(|| {
            let start = Instant::now();
            let _ = std::fs::write(&src, b"rename durable content");
            let _ = contextra_sys::durability::rename_durable(&src, &dst);
            rename_durations.push(start.elapsed().as_nanos() as u64);
        });
    });

    group.finish();

    let calc_percentiles = |mut v: Vec<u64>| -> (u64, u64, u64) {
        if v.is_empty() {
            return (0, 0, 0);
        }
        v.sort_unstable();
        let p50 = v[v.len() * 50 / 100];
        let p95 = v[v.len() * 95 / 100];
        let p99 = v[v.len() * 99 / 100];
        (p50, p95, p99)
    };

    let (fdata_p50, fdata_p95, fdata_p99) = calc_percentiles(fdatasync_durations);
    let (fsync_p50, fsync_p95, fsync_p99) = calc_percentiles(fsync_durations);
    let (rename_p50, rename_p95, rename_p99) = calc_percentiles(rename_durations);

    // JSON-Bericht nach target/bench-report/contextra-sys.json schreiben (Regel 5)
    let report_dir = PathBuf::from("target/bench-report");
    let _ = std::fs::create_dir_all(&report_dir);
    let report_file = report_dir.join("contextra-sys.json");
    let report_content = format!(
        r#"{{
  "crate": "contextra-sys",
  "benchmarks": [
    {{
      "name": "BM-SYS-FDATASYNC",
      "p50_ns": {fdata_p50},
      "p95_ns": {fdata_p95},
      "p99_ns": {fdata_p99},
      "unit": "ns"
    }},
    {{
      "name": "BM-SYS-FSYNC",
      "p50_ns": {fsync_p50},
      "p95_ns": {fsync_p95},
      "p99_ns": {fsync_p99},
      "unit": "ns"
    }},
    {{
      "name": "BM-SYS-RENAME-DURABLE",
      "p50_ns": {rename_p50},
      "p95_ns": {rename_p95},
      "p99_ns": {rename_p99},
      "unit": "ns"
    }}
  ]
}}"#
    );
    let _ = std::fs::write(report_file, report_content);
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = run_benchmarks
}
criterion_main!(benches);
