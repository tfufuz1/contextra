// FILE-CONTEXT
// ZWECK: Criterion-Benchmark zur Messung des Recall-Abfalls und der Latenz von search_filtered
// INVARIANTEN: Deterministische Vektoren und DocIDs, feste Seed-Initialisierung
// HOTSPOTS: 60-140
// STAND: 2026-11-11
// BEREICH: AGENTS §6, SPEC §11.4 — DiskANN gefilterte Suche und Selektivitäts-Trade-offs

// ANCHOR[contextra-vector:W11-BENCH-010]

mod common;

use common::DEFAULT_BENCH_SEED;
use contextra_ports::VectorIndex;
use contextra_types::{DocId, ScoredDocument};
use contextra_vector::types::DistanceMetric;
use contextra_vector::diskann::{DiskAnnConfig, DiskAnnIndex};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::BTreeSet;

fn compute_ground_truth(
    vectors: &[Vec<f32>],
    doc_ids: &[DocId],
    query: &[f32],
    filter: &(dyn Fn(DocId) -> bool),
    k: usize,
) -> Vec<DocId> {
    let metric = DistanceMetric::Euclidean;
    let mut scored: Vec<(DocId, f32)> = vectors
        .iter()
        .zip(doc_ids.iter())
        .filter(|(_, &doc_id)| filter(doc_id))
        .map(|(v, &doc_id)| {
            let dist = contextra_vector::distance::compute_distance_trusted(query, v, metric)
                .expect("Valid distance computation");
            let score = 1.0 / (1.0 + dist);
            (doc_id, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scored.truncate(k);
    scored.into_iter().map(|(id, _)| id).collect()
}

fn calculate_recall(retrieved: &[ScoredDocument], ground_truth: &[DocId]) -> f32 {
    if ground_truth.is_empty() {
        return 1.0;
    }
    let retrieved_ids: BTreeSet<DocId> = retrieved.iter().map(|doc| doc.doc_id).collect();
    let hits = ground_truth
        .iter()
        .filter(|id| retrieved_ids.contains(id))
        .count();
    hits as f32 / ground_truth.len() as f32
}

fn bench_filtered_search_selectivity(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime required for selectivity benchmark");

    let temp_dir = tempfile::tempdir().expect("Tempdir creation failed");
    let index_path = temp_dir.path().join("selectivity_bench.diskann");

    let n = 5_000;
    let dim = 16;
    let k = 10;
    let num_queries = 20;

    let mut rng = StdRng::seed_from_u64(DEFAULT_BENCH_SEED);

    let vectors: Vec<Vec<f32>> = (0..n)
        .map(|_| (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect())
        .collect();
    let doc_ids: Vec<DocId> = (1..=n as u64).map(DocId::from).collect();

    let queries: Vec<Vec<f32>> = (0..num_queries)
        .map(|_| (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect())
        .collect();

    let config = DiskAnnConfig {
        index_path,
        dimension: dim,
        max_degree: 32,
        beam_width: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config).expect("DiskAnnIndex creation failed");

    rt.block_on(async {
        index
            .build(&vectors, &doc_ids)
            .await
            .expect("Index build failed");
    });

    let filters: [(&str, Box<dyn Fn(DocId) -> bool + Send + Sync>); 4] = [
        (
            "selectivity_50_percent",
            Box::new(|id: DocId| id.inner() % 2 == 0),
        ),
        (
            "selectivity_20_percent",
            Box::new(|id: DocId| id.inner() % 5 == 0),
        ),
        (
            "selectivity_05_percent",
            Box::new(|id: DocId| id.inner() % 20 == 0),
        ),
        (
            "selectivity_01_percent",
            Box::new(|id: DocId| id.inner() % 100 == 0),
        ),
    ];

    let mut group = c.benchmark_group("FilteredSearch_Selectivity");

    for (label, filter) in &filters {
        group.bench_function(*label, |b| {
            let mut query_idx = 0;
            b.iter(|| {
                let query = &queries[query_idx % num_queries];
                query_idx += 1;
                rt.block_on(async {
                    let res = index
                        .search_filtered(
                            black_box(query),
                            black_box(k),
                            black_box(Some(filter.as_ref())),
                        )
                        .await;
                    black_box(res).expect("Search failed");
                });
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_filtered_search_selectivity);
criterion_main!(benches);
