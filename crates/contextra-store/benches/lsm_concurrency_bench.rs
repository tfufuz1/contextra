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

criterion_group!(benches, bench_lsm_concurrent_commits);
criterion_main!(benches);
