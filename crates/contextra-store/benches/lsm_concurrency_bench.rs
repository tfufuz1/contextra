use contextra_core::{StorageEngine, TxId};
use contextra_store::{LsmConfig, LsmStorage};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn bench_lsm_concurrent_commits(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("LSM_Concurrent_Commits");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    for n_writers in [1usize, 4, 16, 32] {
        group.throughput(Throughput::Elements((n_writers * 100) as u64));
        group.bench_with_input(
            BenchmarkId::new("tps_writers", n_writers),
            &n_writers,
            |b, &nw| {
                b.to_async(&rt).iter(|| async move {
                    let tmp = TempDir::new().unwrap();
                    let config = LsmConfig {
                        path: tmp.path().to_path_buf(),
                        ..Default::default()
                    };
                    let engine = Arc::new(LsmStorage::new(config).await.unwrap());
                    let handles: Vec<_> = (0..nw)
                        .map(|w| {
                            let e = engine.clone();
                            tokio::spawn(async move {
                                for i in 0..100u64 {
                                    let tx = TxId::new(w as u64 * 100 + i);
                                    e.put(tx, &[w as u8, i as u8], b"v").await.unwrap();
                                    e.commit(tx).await.unwrap();
                                }
                            })
                        })
                        .collect();
                    for h in handles {
                        h.await.unwrap();
                    }
                    black_box(engine);
                });
            },
        );
    }
    group.finish();
}

// REGRESSIONS-GATE: TPS@16-Writer >= 0.6 * 16 * TPS@1-Writer (Group-Commit-Effizienz)
// Baseline: aufgezeichnet in benchmarks/results/lsm_concurrency_baseline.json

/*
Expected Baseline JSON (benchmarks/results/lsm_concurrency_baseline.json):
{
  "tps_writers_1": 12000,
  "tps_writers_4": 42000,
  "tps_writers_16": 115000,
  "tps_writers_32": 180000
}
*/

fn get_process_rss_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/self/statm") {
            let mut parts = content.split_whitespace();
            let _total_size = parts.next()?;
            if let Some(resident_pages_str) = parts.next() {
                if let Ok(pages) = resident_pages_str.parse::<u64>() {
                    let page_size = 4096u64;
                    return Some(pages.saturating_mul(page_size));
                }
            }
        }
    }
    None
}

fn bench_lsm_compaction_extreme_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("LSM_Compaction_Extreme_Load");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(15));

    group.throughput(Throughput::Elements(50_000));
    group.bench_function("50k_puts_large_values", |b| {
        b.to_async(&rt).iter(|| async move {
            let tmp = TempDir::new().unwrap();
            let config = LsmConfig {
                path: tmp.path().to_path_buf(),
                memtable_size_limit: 256 * 1024, // 256 KiB forces frequent flushes and compactions
                ..Default::default()
            };
            let engine = LsmStorage::new(config).await.unwrap();

            let initial_rss = get_process_rss_bytes().unwrap_or(0);
            let mut peak_rss = initial_rss;

            const TOTAL_OPS: usize = 50_000;
            for i in 0..TOTAL_OPS {
                let key = format!("key_{:08}", i);
                // Value between 1 KiB and 16 KiB
                let val_len = 1024 + ((i * 31) % (15 * 1024));
                let val = vec![(i % 256) as u8; val_len];

                let tx = TxId::new(i as u64 + 1);
                engine.put(tx, key.as_bytes(), &val).await.unwrap();
                engine.commit(tx).await.unwrap();

                if i % 5000 == 0 {
                    if let Some(rss) = get_process_rss_bytes() {
                        if rss > peak_rss {
                            peak_rss = rss;
                        }
                    }
                }
            }

            engine.force_flush().await.unwrap();
            let _ = engine.maybe_compact().await;

            if let Some(rss) = get_process_rss_bytes() {
                if rss > peak_rss {
                    peak_rss = rss;
                }
            }

            eprintln!(
                "[LSM_Compaction_Extreme_Load] Completed 50,000 puts. Initial RSS: {:.2} MB, Peak RSS: {:.2} MB",
                initial_rss as f64 / (1024.0 * 1024.0),
                peak_rss as f64 / (1024.0 * 1024.0)
            );

            black_box(engine);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_lsm_concurrent_commits, bench_lsm_compaction_extreme_load);
criterion_main!(benches);
