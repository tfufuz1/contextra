use contextra_core::TxId;
use contextra_crypto::crypto::KeyManager;
use contextra_store::wal::{Wal, WalConfig, WalFlusherConfig, WalOp};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn bench_wal_encryption(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let batch_sizes = [8, 32, 128];

    let mut group = c.benchmark_group("WAL_Encryption");

    for size in batch_sizes {
        group.bench_with_input(
            BenchmarkId::new("batch_preparation", size),
            &size,
            |b, &s| {
                b.to_async(&rt).iter(|| async move {
                    let tmp = TempDir::new().unwrap();
                    let wal_path = tmp.path().join("batch_enc.wal");
                    let km = Arc::new(
                        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890")
                            .unwrap(),
                    );
                    let wal = Wal::open_with_key_manager(&wal_path, Some(km))
                        .await
                        .unwrap();

                    let ops: Vec<_> = (0..s)
                        .map(|i| {
                            (
                                WalOp::Put {
                                    tx_id: TxId::new(i as u64),
                                    key: format!("key_{:05}", i).into_bytes(),
                                    value: format!("value_{:05}", i).into_bytes(),
                                },
                                i as u64,
                            )
                        })
                        .collect();

                    let (_batch, _) = wal.prepare_batch(ops).await.unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_wal_group_commit_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let concurrency_levels = [1, 8, 32, 128];
    let batch_windows = [0u64, 100u64, 500u64];

    let mut group = c.benchmark_group("WAL_GroupCommit_Throughput");

    for &batch_window in &batch_windows {
        for &concurrency in &concurrency_levels {
            let param_id = format!("window_{}us_producers_{}", batch_window, concurrency);

            group.throughput(Throughput::Elements(concurrency as u64));
            group.bench_function(
                BenchmarkId::new("append_batch_concurrency", param_id),
                |b| {
                    b.to_async(&rt).iter(|| async move {
                        let tmp = TempDir::new().unwrap();
                        let wal_path = tmp.path().join("group_commit_bench.wal");

                        let config = WalConfig {
                            flusher_config: WalFlusherConfig {
                                batch_window_micros: batch_window,
                                queue_capacity: 10_000,
                            },
                            ..Default::default()
                        };

                        let wal = Arc::new(Wal::open_with_config(&wal_path, config).await.unwrap());

                        let mut handles = Vec::with_capacity(concurrency);
                        for producer_id in 0..concurrency {
                            let wal_clone = Arc::clone(&wal);
                            handles.push(tokio::spawn(async move {
                                let seq = (producer_id + 1) as u64;
                                let op = WalOp::Put {
                                    tx_id: TxId::new(seq),
                                    key: format!("p_{producer_id}_k").into_bytes(),
                                    value: format!("p_{producer_id}_v").into_bytes(),
                                };
                                let (batch, _) = wal_clone.prepare_batch(vec![(op, seq)]).await.unwrap();
                                wal_clone.append_batch(batch).await.unwrap();
                            }));
                        }

                        for handle in handles {
                            handle.await.unwrap();
                        }
                    });
                },
            );
        }
    }

    group.finish();
}

#[cfg(feature = "fault-injection")]
fn bench_wal_fault_injection_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("WAL_GroupCommit_FaultInjection_Overhead");

    group.throughput(Throughput::Elements(16));
    group.bench_function("append_batch_instrumented_default", |b| {
        b.to_async(&rt).iter(|| async move {
            let tmp = TempDir::new().unwrap();
            let wal_path = tmp.path().join("fault_overhead.wal");

            let config = WalConfig {
                flusher_config: WalFlusherConfig {
                    batch_window_micros: 100,
                    queue_capacity: 1_024,
                },
                ..Default::default()
            };

            let wal = Arc::new(Wal::open_with_config(&wal_path, config).await.unwrap());

            let mut handles = Vec::with_capacity(16);
            for i in 0..16 {
                let wal_clone = Arc::clone(&wal);
                handles.push(tokio::spawn(async move {
                    let seq = (i + 1) as u64;
                    let op = WalOp::Put {
                        tx_id: TxId::new(seq),
                        key: format!("fi_k_{i}").into_bytes(),
                        value: format!("fi_v_{i}").into_bytes(),
                    };
                    let (batch, _) = wal_clone.prepare_batch(vec![(op, seq)]).await.unwrap();
                    wal_clone.append_batch(batch).await.unwrap();
                }));
            }

            for handle in handles {
                handle.await.unwrap();
            }
        });
    });

    group.finish();
}

#[cfg(feature = "fault-injection")]
criterion_group!(
    benches,
    bench_wal_encryption,
    bench_wal_group_commit_throughput,
    bench_wal_fault_injection_overhead
);

#[cfg(not(feature = "fault-injection"))]
criterion_group!(
    benches,
    bench_wal_encryption,
    bench_wal_group_commit_throughput
);

criterion_main!(benches);
