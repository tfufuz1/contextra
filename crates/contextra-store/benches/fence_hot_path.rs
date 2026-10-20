use contextra_ports::StorageEngine;
use contextra_types::TxId;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn bench_fence_put_and_commit(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("fence_hot_path");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(5));

    group.throughput(Throughput::Elements(100));
    group.bench_function("put_commit_100_single_thread", |b| {
        b.to_async(&rt).iter(|| async {
            let tmp = TempDir::new().unwrap();
            let config = LsmConfig {
                path: tmp.path().to_path_buf(),
                ..Default::default()
            };
            let engine = LsmStorage::open(config).await.unwrap();
            let tx = TxId::try_new(1).unwrap();

            for i in 0..100u64 {
                let key = format!("key_{:08}", i);
                engine.put(tx, key.as_bytes(), b"val").await.unwrap();
            }
            engine.commit(tx).await.unwrap();

            black_box(engine);
        });
    });

    for n_writers in [1usize, 4, 16] {
        group.throughput(Throughput::Elements((n_writers * 100) as u64));
        group.bench_with_input(
            BenchmarkId::new("concurrent_commits", n_writers),
            &n_writers,
            |b, &nw| {
                b.to_async(&rt).iter(|| async move {
                    let tmp = TempDir::new().unwrap();
                    let config = LsmConfig {
                        path: tmp.path().to_path_buf(),
                        ..Default::default()
                    };
                    let engine = Arc::new(LsmStorage::open(config).await.unwrap());
                    let handles: Vec<_> = (0..nw)
                        .map(|w| {
                            let e = Arc::clone(&engine);
                            tokio::spawn(async move {
                                let tx = TxId::try_new(w as u64 + 1).unwrap();
                                for i in 0..100u64 {
                                    let key = format!("w{}_key_{:08}", w, i);
                                    e.put(tx, key.as_bytes(), b"val").await.unwrap();
                                }
                                e.commit(tx).await.unwrap();
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

criterion_group!(benches, bench_fence_put_and_commit);
criterion_main!(benches);
