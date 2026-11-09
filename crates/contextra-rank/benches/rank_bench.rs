// ZWECK: Benchmarks für RRF-Rangfusion und Kalibrierung (Isotonic, Platt).

//! # Benchmark-Suite: `rank_bench`
//!
//! Misst Durchsatz und Latenz von Reciprocal Rank Fusion und Kalibrierungsmethoden
//! unter Verwendung des Criterion-Frameworks (SPEC §9.3).

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use contextra_rank::calibration::{calibrate_score, Calibrator, IsotonicCalibrator, PlattScaler};
use contextra_rank::fusion::{
    reciprocal_rank_fusion, rrf_fuse, weighted_reciprocal_rank_fusion, ScoredItem, SearchResult,
};

fn bench_fusion(c: &mut Criterion) {
    let mut group = c.benchmark_group("rank_fusion");

    for num_lists in [2, 4, 8] {
        for num_docs in [100, 1_000, 10_000] {
            let lists: Vec<Vec<ScoredItem>> = (0..num_lists)
                .map(|list_idx| {
                    (0..num_docs)
                        .map(|i| {
                            let doc_num = (i + list_idx * 17) % (num_docs + 50);
                            ScoredItem::new(format!("doc_{doc_num}"), (num_docs - i) as f32)
                        })
                        .collect()
                })
                .collect();

            let param_id = format!("{num_lists}lists_{num_docs}docs");

            group.bench_with_input(
                BenchmarkId::new("rrf_fuse", &param_id),
                &lists,
                |b, input_lists| b.iter(|| rrf_fuse(black_box(input_lists), 60)),
            );

            // Also benchmark legacy SearchResult batch fusion for 2 signals at 100 docs
            if num_lists == 2 && num_docs == 100 {
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

                group.bench_function("reciprocal_rank_fusion_2x100", |b| {
                    b.iter(|| {
                        reciprocal_rank_fusion(black_box(vec![set1.clone(), set2.clone()]), 10)
                    })
                });

                let weighted_inputs = vec![
                    ("vector".to_string(), set1, 0.6),
                    ("text".to_string(), set2, 0.4),
                ];

                group.bench_function("weighted_reciprocal_rank_fusion_2x100", |b| {
                    b.iter(|| {
                        weighted_reciprocal_rank_fusion(black_box(weighted_inputs.clone()), 10)
                    })
                });
            }
        }
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

    let iso_calibrator = Calibrator::isotonic(isotonic.clone());
    let platt_calibrator = Calibrator::platt(platt.clone());

    group.bench_function("isotonic_lookup", |b| {
        b.iter(|| isotonic.calibrated_probability(black_box(0.55)))
    });

    group.bench_function("platt_predict", |b| {
        b.iter(|| platt.predict(black_box(0.55)))
    });

    group.bench_function("calibrate_score_isotonic", |b| {
        b.iter(|| calibrate_score(black_box(0.55), &iso_calibrator))
    });

    group.bench_function("calibrate_score_platt", |b| {
        b.iter(|| calibrate_score(black_box(0.55), &platt_calibrator))
    });

    group.finish();
}

criterion_group!(benches, bench_fusion, bench_calibration);
criterion_main!(benches);
