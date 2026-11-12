// FILE-CONTEXT
// ZWECK: Criterion-Benchmark für F-02 Partial HNSW Rebuild Recall-Regressionskurve über wachsende Tombstone-Raten
// INVARIANTEN: Deterministische Vektorgenerierung via StdRng mit festem Seed (INV-INDEX-APPLY-1)
// HOTSPOTS: 35-95
// STAND: 2026-11-11
// BEREICH: AGENTS §6, SPEC §11.4 — Wartungsprimitiven und Rebuild-Recall

// ANCHOR[contextra-vector:W11-BENCH-009]

mod common;

use common::DEFAULT_BENCH_SEED;
use contextra_ports::VectorIndex;
use contextra_types::{DocId, TxId};
use contextra_vector::types::DistanceMetric;
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::BTreeSet;

fn generate_vectors(count: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::with_capacity(count);
    for _ in 0..count {
        let mut v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in v.iter_mut() {
                *x /= norm;
            }
        }
        vectors.push(v);
    }
    vectors
}

fn compute_ground_truth(
    query: &[f32],
    vectors: &[Vec<f32>],
    deleted_set: &BTreeSet<u64>,
    k: usize,
) -> Vec<DocId> {
    let mut scored: Vec<(DocId, f32)> = vectors
        .iter()
        .enumerate()
        .filter(|(idx, _)| !deleted_set.contains(&((*idx + 1) as u64)))
        .map(|(idx, v)| {
            let dist = contextra_vector::distance::compute_distance_trusted(
                query,
                v,
                DistanceMetric::Cosine,
            )
            .expect("Valid distance computation");
            (DocId::new((idx + 1) as u64), dist)
        })
        .collect();

    scored.sort_by(|a, b| a.1.total_cmp(&b.1));
    scored.truncate(k);
    scored.into_iter().map(|(id, _)| id).collect()
}

fn calculate_recall(retrieved: &[contextra_types::ScoredDocument], ground_truth: &[DocId]) -> f64 {
    if ground_truth.is_empty() {
        return 1.0;
    }
    let gt_set: BTreeSet<DocId> = ground_truth.iter().copied().collect();
    let hits = retrieved
        .iter()
        .filter(|doc| gt_set.contains(&doc.doc_id))
        .count();
    hits as f64 / ground_truth.len() as f64
}

fn bench_partial_rebuild_recall(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime required");

    let dim = 16;
    let n = 200;
    let k = 10;
    let vectors = generate_vectors(n, dim, DEFAULT_BENCH_SEED);
    let queries = generate_vectors(10, dim, DEFAULT_BENCH_SEED ^ 0x1234);

    let tombstone_ratios = [0.10, 0.30, 0.50];

    let mut group = c.benchmark_group("PartialRebuildRecall");

    for &ratio in &tombstone_ratios {
        let tombstone_count = (n as f64 * ratio) as usize;
        let tombstone_ids: Vec<u64> = (1..=tombstone_count as u64).collect();

        group.bench_with_input(
            BenchmarkId::new("recall_eval", format!("ratio_{:.0}pct", ratio * 100.0)),
            &ratio,
            |b, _| {
                b.iter_batched(
                    || {
                        rt.block_on(async {
                            let config = HnswConfig {
                                dimension: dim,
                                m: 16,
                                ef_construction: 64,
                                ef_search: 64,
                                rebuild_threshold: 0.0,
                                ..Default::default()
                            };
                            let index = HnswIndex::try_new(config).expect("Index creation");

                            let tx = TxId::new(1);
                            for (idx, vec) in vectors.iter().enumerate() {
                                index
                                    .insert(tx, DocId::new((idx + 1) as u64), vec)
                                    .await
                                    .unwrap();
                            }
                            index.commit(tx).await.unwrap();

                            let tx_del = TxId::new(2);
                            for &id in &tombstone_ids {
                                index.delete(tx_del, DocId::new(id)).await.unwrap();
                            }
                            index.commit(tx_del).await.unwrap();

                            index
                        })
                    },
                    |index| {
                        rt.block_on(async {
                            let tombstone_set: BTreeSet<u64> =
                                tombstone_ids.iter().copied().collect();
                            for q in &queries {
                                let res = index.search(black_box(q), k).await.unwrap();
                                let gt = compute_ground_truth(q, &vectors, &tombstone_set, k);
                                let rec = calculate_recall(&res, &gt);
                                black_box(rec);
                            }
                        });
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_partial_rebuild_recall);
criterion_main!(benches);
