// FILE-CONTEXT
// ZWECK: Criterion-Benchmark für HNSW-Suchlatenz bei 0% vs. 30% gelöschten Datensätzen (Tombstone-Overhead)
// INVARIANTEN: Deterministische Vektoren, feste DocIDs, kein thread_rng (INV-INDEX-APPLY-1)
// HOTSPOTS: 30-100
// STAND: 2026-11-11
// BEREICH: AGENTS §6, SPEC §11.4 — VETO-F02 Tombstone-Overhead und Suchlatenz

// ANCHOR[contextra-vector:W11-BENCH-005]

mod common;

use common::{generate_raw_vectors, DEFAULT_BENCH_SEED, DIM_MEDIUM};
use contextra_ports::VectorIndex;
use contextra_types::{DocId, TxId};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_hnsw_delete_search(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime required");

    let dim = DIM_MEDIUM; // 128
    let num_docs = 1000;

    let vectors = generate_raw_vectors(num_docs, dim, DEFAULT_BENCH_SEED);
    let queries = generate_raw_vectors(1, dim, DEFAULT_BENCH_SEED ^ 0x9999);
    let query = &queries[0];

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
                    .search(black_box(query), black_box(10))
                    .await;
                black_box(res).expect("Search failed");
            });
        });
    });

    group.bench_function("search_30_percent_deletes", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = index_30_deletes
                    .search(black_box(query), black_box(10))
                    .await;
                black_box(res).expect("Search failed");
            });
        });
    });

    group.finish();
}

criterion_group!(benches, bench_hnsw_delete_search);
criterion_main!(benches);
