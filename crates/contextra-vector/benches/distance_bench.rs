// FILE-CONTEXT
// ZWECK: Criterion-Benchmark für Distanzfunktionen (Cosine, Euclidean, Dot Product; SIMD vs. Skalar)
// INVARIANTEN: Deterministische Eingaben über festen Seed (INV-INDEX-APPLY-1), keine Laufzeitzufallswerte
// HOTSPOTS: 25-65
// STAND: 2026-11-11
// BEREICH: AGENTS §6, SPEC §11.4 — Vektordistanzen mit Skalar-Fallback und SIMD-Beschleunigung

// ANCHOR[contextra-vector:W11-BENCH-002]

mod common;

use common::{generate_raw_vectors, DEFAULT_BENCH_SEED, DIM_LARGE};
use contextra_vector::distance::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_distances(c: &mut Criterion) {
    let dim = DIM_LARGE; // 1536
    let vecs = generate_raw_vectors(2, dim, DEFAULT_BENCH_SEED);
    let a = &vecs[0];
    let b = &vecs[1];

    let mut group = c.benchmark_group("Distances");

    group.bench_function("cosine_distance", |b_bench| {
        b_bench.iter(|| cosine_distance(black_box(a), black_box(b)).unwrap())
    });

    group.bench_function("cosine_scalar", |b_bench| {
        b_bench.iter(|| cosine_distance_scalar(black_box(a), black_box(b)))
    });

    group.bench_function("euclidean_distance", |b_bench| {
        b_bench.iter(|| euclidean_distance(black_box(a), black_box(b)).unwrap())
    });

    group.bench_function("euclidean_scalar", |b_bench| {
        b_bench.iter(|| euclidean_distance_scalar(black_box(a), black_box(b)))
    });

    group.bench_function("dot_product_distance", |b_bench| {
        b_bench.iter(|| dot_product_distance(black_box(a), black_box(b)).unwrap())
    });

    group.bench_function("dot_product_scalar", |b_bench| {
        b_bench.iter(|| dot_product_scalar(black_box(a), black_box(b)))
    });

    group.finish();
}

criterion_group!(benches, bench_distances);
criterion_main!(benches);
