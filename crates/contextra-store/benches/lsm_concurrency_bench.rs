use contextra_core::{StorageEngine, TxId};
use contextra_store::{LsmConfig, LsmStorage};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn get_rss_bytes() -> usize {
    if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
        let parts: Vec<&str> = statm.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(pages) = parts[1].parse::<usize>() {
                return pages * 4096;
            }
        }
    }
    0
}

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

fn bench_lsm_compaction_extreme_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("LSM_Compaction_Extreme_Load");
    group.sample_size(10);
    group.throughput(Throughput::Elements(50_000));

    group.bench_function("50k_puts_large_values_under_compaction", |b| {
        b.to_async(&rt).iter(|| async move {
            let tmp = TempDir::new().unwrap();
            let config = LsmConfig {
                path: tmp.path().to_path_buf(),
                memtable_size_limit: 256 * 1024, // Low limit (256 KiB) forces frequent flushes & compactions
                ..Default::default()
            };
            let engine = LsmStorage::new(config).await.unwrap();

            let mut peak_rss = 0usize;
            let value_buf = vec![0xABu8; 16 * 1024];

            let batch_size = 100;
            let total_puts = 50_000usize;
            let mut global_op_idx = 0u64;

            for batch_idx in 0..(total_puts / batch_size) {
                let tx = TxId::new(batch_idx as u64 + 1);
                for _ in 0..batch_size {
                    let key = format!("extreme_load_key_{:08}", global_op_idx);
                    let val_len = 1024 + ((global_op_idx as usize * 37) % (15 * 1024));
                    let val = &value_buf[..val_len];

                    engine.put(tx, key.as_bytes(), val).await.unwrap();
                    global_op_idx += 1;
                }
                engine.commit(tx).await.unwrap();

                if batch_idx % 10 == 0 {
                    let cur_rss = get_rss_bytes();
                    if cur_rss > peak_rss {
                        peak_rss = cur_rss;
                    }
                }
            }

            let final_rss = get_rss_bytes();
            if final_rss > peak_rss {
                peak_rss = final_rss;
            }

            eprintln!(
                "[LSM_Compaction_Extreme_Load] 50,000 puts completed. Peak RSS: {} MB ({} bytes)",
                peak_rss / (1024 * 1024),
                peak_rss
            );

            black_box(engine);
        });
    });

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

criterion_group!(benches, bench_lsm_concurrent_commits, bench_lsm_compaction_extreme_load);
criterion_main!(benches);
