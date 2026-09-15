use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memfuse_core::traits::VectorIndex;
use memfuse_core::types::{DocId, TxId};
use memfuse_index::hnsw::{HnswConfig, HnswIndex};
use rand::Rng;

fn bench_hnsw_delete_search(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime required");

    let dim = 128;
    let num_docs = 1000;
    let mut rng = rand::thread_rng();

    let vectors: Vec<Vec<f32>> = (0..num_docs)
        .map(|_| (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect())
        .collect();

    let query: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();

    // 0% Deletes index
    let index_0_deletes = rt.block_on(async {
        let config = HnswConfig {
            dimension: dim,
            m: 16,
            ef_construction: 64,
            ef_search: 64,
            rebuild_threshold: 0.0, // Prevent auto rebuild
            ..Default::default()
        };
        let index = HnswIndex::try_new(config).expect("Index creation failed");

        let tx = TxId::new(1);
        for (idx, vec) in vectors.iter().enumerate() {
            let doc_id = DocId::new((idx + 1) as u64);
            index.insert(tx, doc_id, vec).await.unwrap();
        }
        index.commit(tx).await.unwrap();
        index
    });

    // 30% Deletes index
    let index_30_deletes = rt.block_on(async {
        let config = HnswConfig {
            dimension: dim,
            m: 16,
            ef_construction: 64,
            ef_search: 64,
            rebuild_threshold: 0.0, // Prevent auto rebuild
            ..Default::default()
        };
        let index = HnswIndex::try_new(config).expect("Index creation failed");

        let tx1 = TxId::new(1);
        for (idx, vec) in vectors.iter().enumerate() {
            let doc_id = DocId::new((idx + 1) as u64);
            index.insert(tx1, doc_id, vec).await.unwrap();
        }
        index.commit(tx1).await.unwrap();

        // Delete 30% of docs
        let tx2 = TxId::new(2);
        for idx in 1..=(num_docs * 30 / 100) {
            index.delete(tx2, DocId::new(idx as u64)).await.unwrap();
        }
        index.commit(tx2).await.unwrap();

        index
    });

    let mut group = c.benchmark_group("HNSW_Delete_Search");

    group.bench_function("search_0_percent_deletes", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = index_0_deletes
                    .search(black_box(&query), black_box(10))
                    .await;
                black_box(res).expect("Search failed");
            });
        });
    });

    group.bench_function("search_30_percent_deletes", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = index_30_deletes
                    .search(black_box(&query), black_box(10))
                    .await;
                black_box(res).expect("Search failed");
            });
        });
    });

    group.finish();
}

criterion_group!(benches, bench_hnsw_delete_search);
criterion_main!(benches);
