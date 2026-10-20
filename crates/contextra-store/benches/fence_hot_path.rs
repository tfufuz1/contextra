use contextra_ports::StorageEngine;
use contextra_types::TxId;
use contextra_store::{LsmConfig, LsmStorage};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn bench_fence_hot_path_put_and_commit(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("fence_hot_path");
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(5));

    // 1. Single Put + Commit loop
    group.throughput(Throughput::Elements(100));
    group.bench_function("put_and_commit_active_no_fence", |b| {
        b.to_async(&rt).iter_custom(|iters| async move {
            let tmp = TempDir::new().unwrap();
            let config = LsmConfig {
                path: tmp.path().to_path_buf(),
                ..Default::default()
            };
            let engine = LsmStorage::open(config).await.unwrap();

            let start = std::time::Instant::now();
            for i in 0..iters {
                let tx = TxId::try_new(i + 1).unwrap();
                let key = format!("bench_key_{:08}", i % 100);
                let val = b"bench_value_payload_data_64_bytes_padding_buffer_for_fence_test";
                engine.put(tx, key.as_bytes(), val).await.unwrap();
                engine.commit(tx).await.unwrap();
            }
            let duration = start.elapsed();
            black_box(&engine);
            engine.close().await.unwrap();
            duration
        });
    });

    // 2. Staging Puts only (measuring put check overhead)
    group.throughput(Throughput::Elements(1000));
    group.bench_function("staging_puts_active_no_fence", |b| {
        b.to_async(&rt).iter_custom(|iters| async move {
            let tmp = TempDir::new().unwrap();
            let config = LsmConfig {
                path: tmp.path().to_path_buf(),
                ..Default::default()
            };
            let engine = LsmStorage::open(config).await.unwrap();

            let key_base = b"staging_key_prefix_0000_";
            let val = b"val_data";

            let start = std::time::Instant::now();
            for i in 0..iters {
                let tx = TxId::try_new((i / 1000) + 1).unwrap();
                engine.put(tx, key_base, val).await.unwrap();
            }
            let duration = start.elapsed();
            black_box(&engine);
            engine.close().await.unwrap();
            duration
        });
    });

    // 3. Batch commit (measuring commit check overhead for batch of 100 keys)
    group.throughput(Throughput::Elements(100));
    group.bench_function("batch_commit_100_keys", |b| {
        b.to_async(&rt).iter_custom(|iters| async move {
            let tmp = TempDir::new().unwrap();
            let config = LsmConfig {
                path: tmp.path().to_path_buf(),
                ..Default::default()
            };
            let engine = LsmStorage::open(config).await.unwrap();

            let start = std::time::Instant::now();
            for i in 0..iters {
                let tx = TxId::try_new(i + 1).unwrap();
                for k in 0..100 {
                    let key = format!("batch_key_{:04}_{:04}", i % 10, k);
                    engine.put(tx, key.as_bytes(), b"value").await.unwrap();
                }
                engine.commit(tx).await.unwrap();
            }
            let duration = start.elapsed();
            black_box(&engine);
            engine.close().await.unwrap();
            duration
        });
    });

    group.finish();
}

criterion_group!(benches, bench_fence_hot_path_put_and_commit);
criterion_main!(benches);
