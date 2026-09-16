use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memfuse_store::sstable::BlockCache;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn bench_block_cache_multi_threaded(c: &mut Criterion) {
    let Ok(rt) = Runtime::new() else { return };

    let num_shards = 64;
    let cache = Arc::new(BlockCache::new_with_shards(64, num_shards));

    // Populate a hot set of 1024 cached blocks across shards
    let file_id = 42u64;
    let num_blocks = 1024u64;
    for i in 0..num_blocks {
        let offset = i * 4096;
        let payload = Bytes::from(format!("block_data_payload_for_bench_{}", i));
        cache.insert(file_id, offset, payload);
    }

    let mut group = c.benchmark_group("BlockCache_MultiThreaded_HotSet");

    for &num_threads in &[8usize, 16usize, 32usize] {
        group.bench_function(format!("readers_{}_threads", num_threads), |b| {
            b.to_async(&rt).iter(|| async {
                let mut handles = Vec::with_capacity(num_threads);
                for t in 0..num_threads {
                    let cache_ref = Arc::clone(&cache);
                    handles.push(tokio::spawn(async move {
                        let mut hits = 0usize;
                        // Each thread performs 100 reads on the hot set
                        for i in 0..100u64 {
                            let block_idx = (t as u64 * 31 + i) % num_blocks;
                            let offset = block_idx * 4096;
                            if let Some(val) = cache_ref.get(file_id, offset) {
                                hits += val.len();
                            }
                        }
                        hits
                    }));
                }

                let mut total_bytes = 0usize;
                for h in handles {
                    if let Ok(res) = h.await {
                        total_bytes += res;
                    }
                }
                black_box(total_bytes)
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_block_cache_multi_threaded);
criterion_main!(benches);
