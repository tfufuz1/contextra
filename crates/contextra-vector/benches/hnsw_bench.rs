// FILE-CONTEXT
// ZWECK: Criterion-Benchmark für HNSW Hot-Paths (Insertion Throughput, Query Latency mit k=10 und k=50)
// INVARIANTEN: Deterministische Vektorgenerierung über festen Seed, iter_batched zur Vermeidung von Speicherallokation im Messbereich
// HOTSPOTS: 35-100
// STAND: 2026-11-11
// BEREICH: AGENTS §6, SPEC §11.4 — HNSW-Suchkomplexität O(log n * ef * d) und Einfügedurchsatz

// ANCHOR[contextra-vector:W11-BENCH-004]

mod common;

use common::{generate_raw_vectors, DEFAULT_BENCH_SEED, DIM_MEDIUM};
use contextra_ports::VectorIndex;
use contextra_types::{DocId, TxId};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn bench_hnsw_hot_paths(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime required for HNSW benchmarks");

    let dim = DIM_MEDIUM; // 128
    let num_docs = 100;

    // Deterministische Vektorerzeugung mit festem Seed
    let vectors = generate_raw_vectors(num_docs, dim, DEFAULT_BENCH_SEED);

    // 1. Insertion Throughput Benchmark
    let mut group = c.benchmark_group("HNSW_Insertion");
    group.throughput(Throughput::Elements(num_docs as u64));

    group.bench_function("insert_100_docs", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = HnswConfig {
                    dimension: dim,
                    m: 16,
                    ef_construction: 64,
                    quantize: false,
                    rebuild_threshold: 1.0,
                    ..Default::default()
                };
                let index = HnswIndex::try_new(config).expect("Index creation failed");

                for (idx, vec) in vectors.iter().enumerate() {
                    let tx = TxId::new(1);
                    let doc_id = DocId::new((idx + 1) as u64);
                    index.insert(tx, doc_id, vec).await.expect("Insert failed");
                    index.commit(tx).await.expect("Commit failed");
                }
            });
        });
    });
    group.finish();

    // Setup populated index for search benchmarks
    let populated_index = rt.block_on(async {
        let config = HnswConfig {
            dimension: dim,
            m: 16,
            ef_construction: 64,
            quantize: false,
            rebuild_threshold: 1.0,
            ..Default::default()
        };
        let index = HnswIndex::try_new(config).expect("Index creation failed");

        for (idx, vec) in vectors.iter().enumerate() {
            let tx = TxId::new(1);
            let doc_id = DocId::new((idx + 1) as u64);
            index.insert(tx, doc_id, vec).await.expect("Insert failed");
            index.commit(tx).await.expect("Commit failed");
        }
        index
    });

    let queries = generate_raw_vectors(1, dim, DEFAULT_BENCH_SEED ^ 0xABCD);
    let query = &queries[0];

    // 2. Query Latency Benchmark
    let mut search_group = c.benchmark_group("HNSW_Search");
    search_group.bench_function("search_k10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = populated_index
                    .search(black_box(query), black_box(10))
                    .await;
                black_box(res).expect("Search failed");
            });
        });
    });

    search_group.bench_function("search_k50", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = populated_index
                    .search(black_box(query), black_box(50))
                    .await;
                black_box(res).expect("Search failed");
            });
        });
    });

    search_group.finish();
}

criterion_group!(benches, bench_hnsw_hot_paths);
criterion_main!(benches);
