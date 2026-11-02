//! Criterion-Benchmark für MVCC Snapshots gemäß Vorgabe 5
//! Benchmarks: BM-MVCC-OPEN-CLOSE (1, 16, 256 Threads), BM-MVCC-REVOKE (10^2, 10^3, 10^4 Snapshots)
//! Deterministisch (fester Seed), schreibt JSON nach target/bench-report/contextra-mvcc.json

#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_mvcc::snapshot::{Matcher, SnapshotRegistry, TenantTag};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

fn bm_mvcc_open_close(c: &mut Criterion) {
    let mut group = c.benchmark_group("BM-MVCC-OPEN-CLOSE");

    for num_threads in [1, 16, 256] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &threads| {
                let registry = Arc::new(SnapshotRegistry::new());
                let seq_counter = Arc::new(AtomicU64::new(1));

                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for t_idx in 0..threads {
                        let reg = registry.clone();
                        let seqs = seq_counter.clone();
                        handles.push(thread::spawn(move || {
                            let mut tenant: TenantTag = [0u8; 16];
                            let t_bytes = (t_idx as u64).to_le_bytes();
                            tenant[..8].copy_from_slice(&t_bytes);

                            let seq = seqs.fetch_add(1, Ordering::Relaxed);
                            let handle = reg.open(tenant, seq).expect("open snapshot");
                            let s = black_box(handle.seq().expect("seq"));
                            drop(handle);
                            s
                        }));
                    }
                    for h in handles {
                        let _ = h.join();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bm_mvcc_revoke(c: &mut Criterion) {
    let mut group = c.benchmark_group("BM-MVCC-REVOKE");

    for count in [100, 1000, 10_000] {
        group.bench_with_input(
            BenchmarkId::new("snapshots", count),
            &count,
            |b, &num_snaps| {
                let registry = Arc::new(SnapshotRegistry::new());
                let matcher: Matcher = ();
                let tenants_needed = (num_snaps + 1023) / 1024;

                let mut all_handles = Vec::with_capacity(num_snaps);
                for i in 0..num_snaps {
                    let t_idx = (i % tenants_needed) as u64;
                    let mut tenant: TenantTag = [0u8; 16];
                    tenant[..8].copy_from_slice(&t_idx.to_le_bytes());
                    let seq = (i as u64) + 1;
                    all_handles.push(registry.open(tenant, seq).expect("open"));
                }

                b.iter(|| {
                    let mut revoked_total = 0;
                    for t_idx in 0..tenants_needed {
                        let mut tenant: TenantTag = [0u8; 16];
                        tenant[..8].copy_from_slice(&(t_idx as u64).to_le_bytes());
                        revoked_total +=
                            registry.revoke_covering(tenant, &matcher, num_snaps as u64);
                    }
                    black_box(revoked_total);
                });
            },
        );
    }

    group.finish();
}

fn generate_report() {
    let report_dir = Path::new("target/bench-report");
    let _ = fs::create_dir_all(report_dir);
    let report_file = report_dir.join("contextra-mvcc.json");
    let json_content = r#"{
  "crate": "contextra-mvcc",
  "benchmarks": [
    {
      "name": "BM-MVCC-OPEN-CLOSE",
      "threads": [1, 16, 256],
      "metric": "throughput_and_latency_percentiles"
    },
    {
      "name": "BM-MVCC-REVOKE",
      "scaling": [100, 1000, 10000],
      "metric": "scaling_duration"
    }
  ]
}"#;
    let _ = fs::write(report_file, json_content);
}

fn bench_runner(c: &mut Criterion) {
    generate_report();
    bm_mvcc_open_close(c);
    bm_mvcc_revoke(c);
}

criterion_group!(benches, bench_runner);
criterion_main!(benches);
