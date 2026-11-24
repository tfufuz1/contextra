// FILE-CONTEXT
// ZWECK: Criterion-Benchmark für SQ8 Skalar-Quantisierung (Train, Quantize, Dequantize, Asymmetrische/Symmetrische Distanzen)
// INVARIANTEN: Deterministische Trainings- und Testdaten über festen Seed, keine thread_rng-Nutzung
// HOTSPOTS: 25-70
// STAND: 2026-11-11
// BEREICH: AGENTS §6, SPEC §11.4 — SQ8-Quantisierung und Nachranking

// ANCHOR[contextra-vector:W11-BENCH-003]

mod common;

use common::{generate_raw_vectors, DEFAULT_BENCH_SEED, DIM_LARGE};
use contextra_vector::types::DistanceMetric;
use contextra_vector::quantize::ScalarQuantizer;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_sq8(c: &mut Criterion) {
    let dim = DIM_LARGE; // 1536
    let batch = generate_raw_vectors(10, dim, DEFAULT_BENCH_SEED);
    let refs: Vec<&[f32]> = batch.iter().map(|v| v.as_slice()).collect();
    let quantizer = ScalarQuantizer::train(&refs, dim);

    let test_vecs = generate_raw_vectors(2, dim, DEFAULT_BENCH_SEED ^ 0xFEED);
    let a = &test_vecs[0];
    let b = &test_vecs[1];

    let a_q = quantizer.quantize(a).expect("quantize");
    let b_q = quantizer.quantize(b).expect("quantize");

    let mut group = c.benchmark_group("SQ8");

    group.bench_function("quantize", |b_bench| {
        b_bench.iter(|| quantizer.quantize(black_box(a)).unwrap())
    });

    group.bench_function("dequantize", |b_bench| {
        b_bench.iter(|| quantizer.dequantize(black_box(&a_q)).unwrap())
    });

    group.bench_function("asymmetric_dist_cosine", |b_bench| {
        b_bench.iter(|| {
            quantizer.asymmetric_dist(black_box(a), black_box(&b_q), DistanceMetric::Cosine)
        })
    });

    group.bench_function("symmetric_dist_cosine", |b_bench| {
        b_bench.iter(|| {
            quantizer.symmetric_dist(black_box(&a_q), black_box(&b_q), DistanceMetric::Cosine)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_sq8);
criterion_main!(benches);
