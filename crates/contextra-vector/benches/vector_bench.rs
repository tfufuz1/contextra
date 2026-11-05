#![allow(
    clippy::all,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    warnings
)]
use contextra_ports::{EraseTarget, ErasureLayer, VectorIndex};
use contextra_testmacros::ctx_bench;
use contextra_types::{DocId, TxId};
use contextra_vector::{HnswConfigBuilder, HnswIndex};
use criterion::{criterion_group, criterion_main, Criterion};

#[ctx_bench(id = "BM-VEC")]
fn bench_vector_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let config = HnswConfigBuilder::new(16).build().unwrap();
    let index = HnswIndex::try_new(config).unwrap();

    rt.block_on(async {
        for i in 0..100 {
            let vec = vec![i as f32 * 0.01; 16];
            index
                .insert(TxId(1), DocId::new(i as u64), &vec)
                .await
                .unwrap();
        }
        index.commit(TxId(1)).await.unwrap();
    });

    let query = vec![0.5f32; 16];
    c.bench_function("bm_vec_search", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _ = index.search(&query, 10).await.unwrap();
            });
        });
    });
}

#[ctx_bench(id = "BM-VEC-DEL")]
fn bench_vector_deletion_repair(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("bm_vec_del_repair", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = HnswConfigBuilder::new(8).build().unwrap();
                let index = HnswIndex::try_new(config).unwrap();
                for i in 0..20 {
                    let vec = vec![i as f32 * 0.1; 8];
                    index
                        .insert(TxId(1), DocId::new(i as u64), &vec)
                        .await
                        .unwrap();
                }
                index.commit(TxId(1)).await.unwrap();

                let target = EraseTarget::ExactKeys(vec![0u64.to_be_bytes().to_vec()]);
                let scope = index.resolve(&target).await.unwrap();
                let cx = contextra_ports::PurgeCx::new(10_000);
                let _ = index.purge(&scope, &cx).await.unwrap();
            });
        });
    });
}

#[ctx_bench(id = "BM-VEC-REBUILD")]
fn bench_vector_rebuild(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("bm_vec_rebuild", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = HnswConfigBuilder::new(8).build().unwrap();
                let index = HnswIndex::try_new(config).unwrap();
                for i in 0..20 {
                    let vec = vec![i as f32 * 0.1; 8];
                    index
                        .insert(TxId(1), DocId::new(i as u64), &vec)
                        .await
                        .unwrap();
                }
                index.commit(TxId(1)).await.unwrap();
                index.rebuild().await.unwrap();
            });
        });
    });
}

#[ctx_bench(id = "BM-VEC-MEM")]
fn bench_vector_arena_memory(c: &mut Criterion) {
    c.bench_function("bm_vec_mem_arena", |b| {
        b.iter(|| {
            let arena = contextra_vector::hnsw::HnswArena::new();
            let vec = vec![1.0f32; 16];
            for i in 0..50 {
                arena.store_vector(i, 16, &vec);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_vector_search,
    bench_vector_deletion_repair,
    bench_vector_rebuild,
    bench_vector_arena_memory
);
criterion_main!(benches);
