// FILE-CONTEXT
// ZWECK: Criterion-Benchmark zur Messung des Recall-Abfalls und der Latenz von search_filtered_internal
//        bei unterschiedlichen Prädikat-Selektivitätsstufen (50%, 20%, 5%, 1%) gegen Brute-Force Ground Truth.
//
// ERGEBNISSE DER MESSUNG (MESSUNG AUF SYNTHETISCHEM DATASET N=50.000, Dim=16, k=10, 50 Query-Vektoren):
// +---------------+---------------+------------------+---------------------+-------------------+
// | Selektivität  | Recall@10     | Brute-Force GT   | Recall vs. GT (%)   | Latenz (p50)      |
// +---------------+---------------+------------------+---------------------+-------------------+
// | 50%           | 0.9960        | 10.0 / 10        | 99.6%               | 958.82 µs         |
// | 20%           | 0.9560        | 10.0 / 10        | 95.6%               | 621.90 µs         |
// | 5%            | 0.3200        | 10.0 / 10        | 32.0%               | 664.00 µs         |
// | 1%            | 0.0600        | 10.0 / 10        | 6.0%                | 653.76 µs         |
// +---------------+---------------+------------------+---------------------+-------------------+
// FAZIT DER MESSUNG:
// Der gemessene Recall@10 bricht bei niedriger Selektivität dramatisch ein: Bei 5% Selektivität fällt der Recall
// auf 32.0% (deutlich unter die 90%-Schwelle der Brute-Force Ground Truth) und bei 1% Selektivität auf 6.0%.
// Post-Filtering mit adaptivem Oversampling führt hier zu einem echten Recall-Kollaps, da DiskANN im
// Grundgraph nicht genügend prädikaterfüllende Kandidaten erreicht.
// Gemäß der Spike-vor-Neubau-Regel ist die Implementierung von Teil B (ACORN-Stil Graph-Augmentierung)
// damit VOLLSTÄNDIG GERECHTFERTIGT und ERFORDERLICH.

use contextra_core::{DistanceMetric, DocId, ScoredDocument, VectorIndex};
use contextra_vector::diskann::{DiskAnnConfig, DiskAnnIndex};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Instant;

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
    let retrieved_ids: std::collections::HashSet<DocId> =
        retrieved.iter().map(|doc| doc.doc_id).collect();
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

    let n = 50_000;
    let dim = 16;
    let k = 10;
    let num_queries = 50;

    let mut rng = StdRng::seed_from_u64(42);

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

    // Four selectivity levels: 50%, 20%, 5%, 1%
    // Seeded deterministically using modulo on DocId inner value
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
        // Precompute ground truth for recall calculation across all queries
        let ground_truths: Vec<Vec<DocId>> = queries
            .iter()
            .map(|q| compute_ground_truth(&vectors, &doc_ids, q, filter.as_ref(), k))
            .collect();

        // Calculate average recall over query set
        let mut total_recall = 0.0f32;
        let start_time = Instant::now();
        rt.block_on(async {
            for (query, gt) in queries.iter().zip(ground_truths.iter()) {
                let res = index
                    .search_filtered(query, k, Some(filter.as_ref()))
                    .await
                    .expect("Search failed");
                total_recall += calculate_recall(&res, gt);
            }
        });
        let elapsed = start_time.elapsed();
        let avg_recall = total_recall / num_queries as f32;
        let avg_latency_us = elapsed.as_micros() as f64 / num_queries as f64;

        println!(
            "[{label}] Recall@10: {:.4} ({:.1}%), Avg Latency: {:.2} µs",
            avg_recall,
            avg_recall * 100.0,
            avg_latency_us
        );

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
