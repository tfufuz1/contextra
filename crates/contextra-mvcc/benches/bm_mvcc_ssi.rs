// ANCHOR[contextra-mvcc:BENCH-SSI-ROOT]
//! # Benchmark BM-MVCC-SSI-VALIDATE
//!
//! Benchmark für die SSI-Validierung (`SequenceLogSsiValidator::validate`):
//! - Read-Set-Größen: 10, 100, 1.000, 10.000
//! - Konfliktquoten: 0 %, 10 %, 50 %
//! - Deterministischer Seed
//! - Schreibt JSON-Bericht nach `target/bench-report/contextra-mvcc.json`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_ports::{Clock, SystemClock};
use contextra_types::TxId;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize, Clone)]
struct BenchResultEntry {
    name: String,
    read_set_size: usize,
    conflict_ratio: f64,
    iterations: u64,
    elapsed_nanos: u128,
    avg_nanos_per_op: f64,
}

#[derive(Serialize)]
struct BenchReport {
    benchmark_name: String,
    crate_name: String,
    results: Vec<BenchResultEntry>,
}

fn generate_deterministic_key(idx: usize) -> Vec<u8> {
    format!("item_key_{idx:08}").into_bytes()
}

fn bm_mvcc_ssi_validate(c: &mut Criterion) {
    let mut group = c.benchmark_group("BM-MVCC-SSI-VALIDATE");

    // Sizes according to spec: 10 .. 10^4
    let read_set_sizes = [10, 100, 1_000, 10_000];
    // conflict percentages: 0%, 10%, 50%
    let conflict_configs = [(0usize, 0.0f64), (10usize, 0.10f64), (50usize, 0.50f64)];

    let mut report_entries = Vec::new();

    for &size in &read_set_sizes {
        for &(percent, conflict_ratio) in &conflict_configs {
            let validator = SequenceLogSsiValidator::new();
            let mut read_set = ReadSet::new();

            // Populate ReadSet with snapshot_seq = 100
            for i in 0..size {
                let k = generate_deterministic_key(i);
                read_set.record_read(k, 100);
            }

            // Populate committed writes to match desired conflict ratio
            let num_conflicts = match percent {
                0 => 0,
                10 => size / 10,
                50 => size / 2,
                other => (size * other) / 100,
            };
            for i in 0..num_conflicts {
                let k = generate_deterministic_key(i);
                // commit_seq 105 > snapshot_seq 100 -> Conflict
                validator.record_commit_key(&k, 105);
            }

            let id_str = format!("size_{}_conflict_{:.0}%", size, conflict_ratio * 100.0);
            let tx = TxId::new(1);

            group.bench_with_input(
                BenchmarkId::new("validate", &id_str),
                &(&validator, &read_set),
                |b, (val, rs)| {
                    b.iter(|| {
                        let _ = black_box(val.validate(black_box(tx), black_box(rs)));
                    });
                },
            );

            // Manual timing collection for reporting JSON using Clock port
            let clock = SystemClock::new();
            let iters = 1000u64;
            let start_nanos = clock.monotonic_nanos();
            for _ in 0..iters {
                let _ = black_box(validator.validate(black_box(tx), black_box(&read_set)));
            }
            let end_nanos = clock.monotonic_nanos();
            let elapsed_nanos = (end_nanos.saturating_sub(start_nanos)) as u128;
            let avg_nanos = (elapsed_nanos as f64) / (iters as f64);

            report_entries.push(BenchResultEntry {
                name: id_str,
                read_set_size: size,
                conflict_ratio,
                iterations: iters,
                elapsed_nanos,
                avg_nanos_per_op: avg_nanos,
            });
        }
    }

    group.finish();

    // Write JSON report to target/bench-report/contextra-mvcc.json
    // Support running from crate root or repo root
    let report_dirs = [
        Path::new("target/bench-report"),
        Path::new("../../target/bench-report"),
    ];
    for report_dir in report_dirs {
        let _ = fs::create_dir_all(report_dir);
        let report_file = report_dir.join("contextra-mvcc.json");
        let report = BenchReport {
            benchmark_name: "BM-MVCC-SSI-VALIDATE".to_string(),
            crate_name: "contextra-mvcc".to_string(),
            results: report_entries.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&report) {
            let _ = fs::write(report_file, json);
        }
    }
}

criterion_group!(benches, bm_mvcc_ssi_validate);
criterion_main!(benches);
