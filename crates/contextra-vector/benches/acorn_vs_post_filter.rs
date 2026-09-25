// FILE-CONTEXT
// ZWECK: Criterion Benchmark für ACORN Naive-Reference Search vs. Post-Filtering Baseline.
// INVARIANTEN: O(n) Reference Benchmark für Recall & Latency Baselines.

use contextra_core::DocId;
use contextra_vector::acorn::{compute_gamma_edge_budget, FilteredIndex, NaiveReferenceIndex};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_acorn_vs_post_filter(c: &mut Criterion) {
    let n = 1_000;
    let dim = 16;
    let k = 10;

    let mut vectors = Vec::with_capacity(n);
    for i in 0..n {
        let v: Vec<f32> = (0..dim).map(|j| ((i + j) as f32 * 0.1).sin()).collect();
        vectors.push((DocId::from(i as u64 + 1), v));
    }

    let index = NaiveReferenceIndex::from_vectors(vectors);
    let query: Vec<f32> = (0..dim).map(|j| (j as f32 * 0.15).cos()).collect();

    let selectivities: [(&str, f32, Box<dyn Fn(DocId) -> bool>); 3] = [
        ("selectivity_50_percent", 0.50, Box::new(|id: DocId| id.inner() % 2 == 0)),
        ("selectivity_10_percent", 0.10, Box::new(|id: DocId| id.inner() % 10 == 0)),
        ("selectivity_01_percent", 0.01, Box::new(|id: DocId| id.inner() % 100 == 0)),
    ];

    let mut group = c.benchmark_group("ACORN_Vs_PostFilter_Baseline");

    for (label, selectivity, filter) in &selectivities {
        let gamma_budget = compute_gamma_edge_budget(16, *selectivity) as u32;

        group.bench_function(*label, |b| {
            b.iter(|| {
                let res = index.search_knn_acorn(
                    black_box(&query),
                    black_box(k),
                    black_box(filter.as_ref()),
                    black_box(gamma_budget),
                );
                black_box(res).expect("Naive search should succeed");
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_acorn_vs_post_filter);
criterion_main!(benches);
