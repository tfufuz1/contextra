//! # Benchmark: SSTable Build, Lookups und Scan
//!
//! Bewertet Build MB/s, Punkt-Lookups mit und ohne Cache sowie Scan MB/s.

use contextra_store::sstable::{create_block_cache, BlockCache, SstableBuilder, SstableReader};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn bench_sstable(c: &mut Criterion) {
    let Ok(rt) = Runtime::new() else { return };
    let Ok(tmp) = TempDir::new() else { return };

    let count = 5_000usize;
    let entry_bytes = 100usize;
    let total_payload_bytes = (count * entry_bytes) as u64;

    // 1. Build MB/s
    {
        let mut group = c.benchmark_group("sstable_build");
        group.throughput(Throughput::Bytes(total_payload_bytes));
        let iter_counter = Arc::new(AtomicU64::new(0));
        group.bench_function(BenchmarkId::new("streaming_build", count), |b| {
            let iter_counter = Arc::clone(&iter_counter);
            b.to_async(&rt).iter(move || {
                let current_iter = iter_counter.fetch_add(1, Ordering::Relaxed);
                let tmp_path = tmp.path().to_path_buf();
                async move {
                    let file_path = tmp_path.join(format!("build_{}_{}.sst", count, current_iter));
                    let mut builder = SstableBuilder::create(&file_path).await.unwrap();
                    for i in 0..count {
                        let key = format!("key_{:07}", i);
                        let val = [0xabu8; 100];
                        builder
                            .add(key.as_bytes(), &val, i as u64 + 1, 1)
                            .await
                            .unwrap();
                    }
                    let meta = builder.finish().await.unwrap();
                    black_box(meta);
                }
            });
        });
        group.finish();
    }

    // Prepare an SSTable for lookup & scan benchmarks
    let bench_sst_path = tmp.path().join("bench_data.sst");
    rt.block_on(async {
        let mut builder = SstableBuilder::create(&bench_sst_path).await.unwrap();
        for i in 0..count {
            let key = format!("key_{:07}", i);
            let val = [0x42u8; 100];
            builder
                .add(key.as_bytes(), &val, i as u64 + 1, 1)
                .await
                .unwrap();
        }
        builder.finish().await.unwrap();
    });

    let cached_bc = create_block_cache(64);
    let no_cache_bc = Arc::new(BlockCache::new(1));

    let reader_cached = rt.block_on(async {
        SstableReader::open(&bench_sst_path, cached_bc)
            .await
            .unwrap()
    });
    let reader_no_cache = rt.block_on(async {
        SstableReader::open(&bench_sst_path, no_cache_bc)
            .await
            .unwrap()
    });

    // Warm-up cache
    rt.block_on(async {
        for i in 0..count {
            let key = format!("key_{:07}", i);
            let _ = reader_cached.get(key.as_bytes()).await;
        }
    });

    // 2. Punkt-Lookups mit und ohne Cache
    {
        let mut group = c.benchmark_group("sstable_lookups");
        group.bench_function("point_lookup_with_cache", |b| {
            b.to_async(&rt).iter(|| async {
                let key = format!("key_{:07}", 2500);
                let res = reader_cached.get(black_box(key.as_bytes())).await.unwrap();
                black_box(res);
            });
        });

        group.bench_function("point_lookup_without_cache", |b| {
            b.to_async(&rt).iter(|| async {
                let key = format!("key_{:07}", 2500);
                let res = reader_no_cache
                    .get(black_box(key.as_bytes()))
                    .await
                    .unwrap();
                black_box(res);
            });
        });
        group.finish();
    }

    // 3. Scan MB/s
    {
        let reader_arc = Arc::new(reader_cached);
        let mut group = c.benchmark_group("sstable_scan");
        group.throughput(Throughput::Bytes(total_payload_bytes));
        group.bench_function(BenchmarkId::new("sequential_scan", count), |b| {
            b.to_async(&rt).iter(|| async {
                let mut stream = reader_arc.stream().await.unwrap();
                let mut read_entries = 0usize;
                while let Some(entry) = stream.next().await.unwrap() {
                    read_entries += 1;
                    black_box(entry);
                }
                black_box(read_entries);
            });
        });
        group.finish();
    }
}

criterion_group!(benches, bench_sstable);
criterion_main!(benches);
