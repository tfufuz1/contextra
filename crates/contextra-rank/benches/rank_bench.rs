// FILE-CONTEXT
// ZWECK: Benchmarks für RRF-Rangfusion und Kalibrierung (Isotonic, Platt).

//! # Benchmark-Suite: `rank_bench`
//!
//! Misst Durchsatz und Latenz von Reciprocal Rank Fusion und Kalibrierungsmethoden
//! unter Verwendung des Criterion-Frameworks (SPEC §9.3).

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use contextra_rank::calibration::{IsotonicCalibrator, PlattScaler};
use contextra_rank::fusion::{
    reciprocal_rank_fusion, weighted_reciprocal_rank_fusion, SearchResult,
};

fn bench_fusion(c: &mut Criterion) {
    let mut group = c.benchmark_group("rank_fusion");

    for num_docs in [10, 50, 200] {
        let set1: Vec<SearchResult> = (0..num_docs)
            .map(|i| SearchResult {
                id: format!("doc_{i}"),
                score: (num_docs - i) as f32,
                metadata: None,
                matched_signals: vec!["vector".to_string()],
                provenance: None,
            })
            .collect();

        let set2: Vec<SearchResult> = (0..num_docs)
            .rev()
            .map(|i| SearchResult {
                id: format!("doc_{i}"),
                score: (i + 1) as f32,
                metadata: None,
                matched_signals: vec!["text".to_string()],
                provenance: None,
            })
            .collect();

        group.bench_with_input(
            criterion::BenchmarkId::new("rrf_2signals", num_docs),
            &num_docs,
            |b, _| {
                b.iter(|| reciprocal_rank_fusion(black_box(vec![set1.clone(), set2.clone()]), 10))
            },
        );

        let weighted_inputs = vec![
            ("vector".to_string(), set1.clone(), 0.6),
            ("text".to_string(), set2.clone(), 0.4),
        ];

        group.bench_with_input(
            criterion::BenchmarkId::new("weighted_rrf_2signals", num_docs),
            &num_docs,
            |b, _| {
                b.iter(|| weighted_reciprocal_rank_fusion(black_box(weighted_inputs.clone()), 10))
            },
        );
    }

    group.finish();
}

fn bench_calibration(c: &mut Criterion) {
    let mut group = c.benchmark_group("rank_calibration");

    let mut isotonic = IsotonicCalibrator::new(10, 500);
    for i in 0..100 {
        let score = (i as f32) / 100.0;
        isotonic.record_outcome(score, i % 2 == 0);
    }
    isotonic.force_rebuild();

    let obs = vec![
        (-2.0, false),
        (-1.0, false),
        (0.0, false),
        (1.0, true),
        (2.0, true),
    ];
    let platt = PlattScaler::fit(&obs);

    group.bench_function("isotonic_lookup", |b| {
        b.iter(|| isotonic.calibrated_probability(black_box(0.55)))
    });

    group.bench_function("platt_predict", |b| {
        b.iter(|| platt.predict(black_box(0.55)))
    });

    group.finish();
}

criterion_group!(benches, bench_fusion, bench_calibration);
criterion_main!(benches);
