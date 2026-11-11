// FILE-CONTEXT
// ZWECK: Criterion-Benchmark zur Messung des Durchsatzes bei unterschiedlichen Overfetch-Faktoren und Selektivitätsstufen
// INVARIANTEN: Deterministische Vektoren und Filterkriterien über modulare DocIDs
// HOTSPOTS: 35-85
// STAND: 2026-11-11
// BEREICH: AGENTS §6, SPEC §11.4 — Prädikatgefilterte Suche und Overfetch-Trade-offs

// ANCHOR[contextra-vector:W11-BENCH-006]

mod common;

use common::DEFAULT_BENCH_SEED;
use contextra_ports::Rng;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Debug)]
struct SeededRng {
    state: std::sync::atomic::AtomicU64,
}

impl SeededRng {
    fn new(seed: u64) -> Self {
        Self {
            state: std::sync::atomic::AtomicU64::new(seed),
        }
    }
}

impl Rng for SeededRng {
    fn next_u32(&self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&self) -> u64 {
        let mut current = self.state.load(std::sync::atomic::Ordering::Relaxed);
        loop {
            let next = current
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            match self.state.compare_exchange_weak(
                current,
                next,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => return next,
                Err(actual) => current = actual,
            }
        }
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            let val = self.next_u64().to_le_bytes();
            let len = chunk.len().min(8);
            chunk[..len].copy_from_slice(&val[..len]);
        }
    }
}

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

    let idx = HnswIndex::try_new_with_rng(config, Arc::new(SeededRng::new(DEFAULT_BENCH_SEED)))
        .expect("Index creation");

    rt.block_on(async {
        let tx = TxId::new(1);
        let mut seed = DEFAULT_BENCH_SEED;
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
