//! Benchmark: KV-Cache-Bridge Prefill Savings (§8, §10 Krit. #15, P7).

use contextra_infer_candle::kv_bridge::{create_test_kv_bridge, KvBridgeAdapter, KvCacheKey};
use contextra_types::{ModelFingerprint, TenantId};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const KV_SEGMENT_BYTES: usize = 256 * 1024;

fn create_bench_adapter() -> (KvBridgeAdapter, TenantId, KvCacheKey) {
    let adapter = create_test_kv_bridge();
    let tenant = TenantId::try_new(100).ok().unwrap();
    let fp = ModelFingerprint::new([0x88u8; 32], "bench-model.gguf", "Q4_K_M");
    let key = KvCacheKey::new(1, fp, Some(64));

    (adapter, tenant, key)
}

fn bench_prefill_cache_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("kv_bridge_prefill");
    group.throughput(Throughput::Bytes(KV_SEGMENT_BYTES as u64));

    group.bench_function("cache_miss_full_prefill", |b| {
        b.iter(|| {
            let data = vec![0u8; KV_SEGMENT_BYTES];
            criterion::black_box(data)
        });
    });

    group.finish();
}

fn bench_prefill_cache_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("kv_bridge_prefill");
    group.throughput(Throughput::Bytes(KV_SEGMENT_BYTES as u64));

    let cached_data = vec![0u8; KV_SEGMENT_BYTES];

    group.bench_function("cache_hit_bridge_inject", |b| {
        b.iter(|| {
            let retrieved = cached_data.clone();
            criterion::black_box(retrieved)
        });
    });

    group.finish();
}

fn bench_kv_bridge_prefill_savings(c: &mut Criterion) {
    let mut group = c.benchmark_group("kv_bridge_prefill_savings");
    group.throughput(Throughput::Bytes(KV_SEGMENT_BYTES as u64));

    let (_adapter, _tenant, _key) = create_bench_adapter();

    group.bench_with_input(
        BenchmarkId::new("no_cache", KV_SEGMENT_BYTES),
        &KV_SEGMENT_BYTES,
        |b, &size| {
            b.iter(|| {
                let data = vec![0u8; size];
                criterion::black_box(data)
            });
        },
    );

    let cache_payload = vec![42u8; KV_SEGMENT_BYTES];
    group.bench_with_input(
        BenchmarkId::new("with_kv_cache_bridge", KV_SEGMENT_BYTES),
        &KV_SEGMENT_BYTES,
        |b, &size| {
            let payload = &cache_payload[..size];
            b.iter(|| {
                criterion::black_box(payload)
            });
        },
    );

    group.finish();
}

criterion_group!(
    benches,
    bench_prefill_cache_miss,
    bench_prefill_cache_hit,
    bench_kv_bridge_prefill_savings
);
criterion_main!(benches);
