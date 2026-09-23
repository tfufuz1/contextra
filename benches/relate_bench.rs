// ANCHOR[PERF:BENCH-RELATE] STATUS:DONE (TS:2026-09-17T00:00:00Z) — Collection::relate() Benchmark
// ZIEL: Criterion-basierte Messung von relate() Latenz und Durchsatz für AK-8 Regressionsnachweis

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use contextra_db::Contextra;
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn bench_relate_performance(c: &mut Criterion) {
    let rt = match Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to initialize tokio runtime: {}", e);
            return;
        }
    };
    let tmp = match TempDir::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to create tempdir: {}", e);
            return;
        }
    };
    let db = match rt.block_on(Contextra::open(tmp.path())) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Contextra DB: {}", e);
            return;
        }
    };

    // Prepare corpus documents
    rt.block_on(async {
        for i in 0..500 {
            if let Err(e) = db
                .insert(
                    &format!("doc-{:05}", i),
                    &vec![0.1f32; 768],
                    Some(serde_json::json!({
                        "text": format!("Document chunk {:05} for relation benchmark testing.", i)
                    })),
                )
                .await
            {
                eprintln!("Failed to insert setup doc: {}", e);
            }
        }
    });

    let mut group = c.benchmark_group("collection_relate");
    group.sample_size(50);

    let counter = AtomicU64::new(0);
    group.bench_function("relate_single_directional", |b| {
        b.to_async(&rt).iter(|| async {
            let val = counter.fetch_add(1, Ordering::Relaxed);
            let from = format!("doc-{:05}", val % 500);
            let to = format!("doc-{:05}", (val * 7 + 3) % 500);
            let _ = db.relate(&from, &to, "references").await;
        });
    });

    let bcounter = AtomicU64::new(0);
    group.bench_function("relate_bidirectional", |b| {
        b.to_async(&rt).iter(|| async {
            let val = bcounter.fetch_add(1, Ordering::Relaxed);
            let from = format!("doc-{:05}", val % 500);
            let to = format!("doc-{:05}", (val * 11 + 5) % 500);
            let _ = db.relate_bidirectional(&from, &to, "co_occurs_with").await;
        });
    });

    let batch_sizes = [10, 100];
    for &n in &batch_sizes {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("relate_batch", n), &n, |b, &size| {
            let b_iter = AtomicU64::new(0);
            b.to_async(&rt).iter(|| async {
                let iter_val = b_iter.fetch_add(1, Ordering::Relaxed);
                for i in 0..size {
                    let from = format!("doc-{:05}", (iter_val * 13 + i as u64) % 500);
                    let to = format!("doc-{:05}", (iter_val * 17 + i as u64 + 1) % 500);
                    let _ = db.relate(&from, &to, "batch_link").await;
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_relate_performance);
criterion_main!(benches);
