#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::disallowed_types,
    unused_imports
)]

use contextra_kvcache::{KvReusePolicy, KvSegment, PrefixRadixTree, TenantIsolatedKvStore};
use contextra_types::TenantId;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_radix_tree_lookup_and_insert(c: &mut Criterion) {
    let tenant = TenantId::try_new(1).unwrap();
    let mut tree = PrefixRadixTree::new(tenant);

    for i in 0..100 {
        let tokens: Vec<u32> = (0..32).map(|x| (x + i * 32) as u32).collect();
        tree.insert(&tokens, i as u64).unwrap();
    }

    c.bench_function("radix_tree_find_longest_prefix", |b| {
        let search_tokens: Vec<u32> = (0..32).collect();
        b.iter(|| {
            let res = tree.find_longest_prefix(black_box(&search_tokens), KvReusePolicy::Always);
            black_box(res)
        })
    });
}

fn bench_store_insert_and_get(c: &mut Criterion) {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(42).unwrap();

    for i in 0..50 {
        let segment = KvSegment::new(tenant, i as u64, vec![0xAA; 1024]);
        store.insert_segment(tenant, segment);
    }

    c.bench_function("store_get_segment_bytes", |b| {
        b.iter(|| {
            let seg = store.get_segment_bytes(black_box(tenant), black_box(10));
            black_box(seg)
        })
    });
}

criterion_group!(
    benches,
    bench_radix_tree_lookup_and_insert,
    bench_store_insert_and_get
);
criterion_main!(benches);
