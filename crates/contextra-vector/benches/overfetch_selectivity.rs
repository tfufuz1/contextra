use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};
use contextra_ports::SeededRng;
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn bench_overfetch_selectivity(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dim = 64;
    let num_vecs = 1000;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(128)
        .ef_search(64)
        .distance_metric(DistanceMetric::Cosine)
        .build()
        .expect("Valid config");

    let idx = HnswIndex::try_new_with_rng(config, Arc::new(SeededRng::new(123456)))
        .expect("Index creation");

    rt.block_on(async {
        let tx = TxId::new(1);
        let mut seed = 12345u64;
        for i in 1..=num_vecs {
            let doc_id = DocId::new(i as u64);
            let vec: Vec<f32> = (0..dim)
                .map(|_| {
                    seed = seed
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    ((seed >> 32) as f32 / 4294967296.0) * 2.0 - 1.0
                })
                .collect();
            idx.insert(tx, doc_id, &vec).await.unwrap();
        }
        idx.commit(tx).await.unwrap();
    });

    let query: Vec<f32> = vec![0.1f32; dim];

    let mut group = c.benchmark_group("OverfetchSelectivity");

    group.bench_function("selectivity_1pct", |b| {
        b.iter(|| {
            rt.block_on(async {
                let filter = |id: DocId| id.inner() % 100 == 0; // 1% selectivity
                black_box(
                    idx.search_filtered(black_box(&query), 10, Some(&filter))
                        .await
                        .unwrap(),
                );
            });
        });
    });

    group.bench_function("selectivity_5pct", |b| {
        b.iter(|| {
            rt.block_on(async {
                let filter = |id: DocId| id.inner() % 20 == 0; // 5% selectivity
                black_box(
                    idx.search_filtered(black_box(&query), 10, Some(&filter))
                        .await
                        .unwrap(),
                );
            });
        });
    });

    group.bench_function("selectivity_20pct", |b| {
        b.iter(|| {
            rt.block_on(async {
                let filter = |id: DocId| id.inner() % 5 == 0; // 20% selectivity
                black_box(
                    idx.search_filtered(black_box(&query), 10, Some(&filter))
                        .await
                        .unwrap(),
                );
            });
        });
    });

    group.finish();
}

criterion_group!(benches, bench_overfetch_selectivity);
criterion_main!(benches);
