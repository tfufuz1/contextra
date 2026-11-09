// FILE-CONTEXT
// ZWECK: Benchmarks for SIMD distance kernels with comparison to simsimd and scalar autovectorization.

//! # Benchmark-Suite: `simd_bench`
//!
//! **Zweck (WP-91):**
//! Misst und vergleicht Durchsatz und Latenz der SIMD-Distanzberechnungen:
//! (a) Eigener Kernel (Laufzeit-Dispatch: AVX2 / AVX-512 / NEON)
//! (b) `simsimd` (SpatialSimilarity: cos, l2sq, dot)
//! (c) Skalar mit Autovektorisierung (skalare Referenz aus `kernels/scalar.rs`)
//!
//! **Getestete Dimensionen:**
//! - 128 Dimensionen
//! - 384 Dimensionen
//! - 768 Dimensionen
//! - 1024 Dimensionen

#![allow(unsafe_code)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use contextra_simd::dispatch::{
    cosine_distance, cosine_distance_f32_bytes, dot_product_distance,
    dot_product_distance_f32_bytes, dot_product_u8, euclidean_distance,
    euclidean_distance_f32_bytes, euclidean_distance_sq_u8,
};
use contextra_simd::kernels::scalar::{
    cosine_distance_scalar, dot_product_scalar, euclidean_distance_scalar,
};
use simsimd::SpatialSimilarity;

fn bench_simd_distances(c: &mut Criterion) {
    let mut group = c.benchmark_group("distance_comparison");

    for dim in [128, 384, 768, 1024] {
        group.throughput(Throughput::Elements(dim as u64));
        let a: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.01 + 0.1).collect();
        let b: Vec<f32> = (0..dim).map(|i| ((i + 1) as f32) * 0.01 + 0.2).collect();
        let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

        let u8_a: Vec<u8> = (0..dim).map(|i| (i % 256) as u8).collect();
        let u8_b: Vec<u8> = (0..dim).map(|i| ((i + 1) % 256) as u8).collect();

        // --- Euclidean (L2) ---
        group.bench_with_input(
            BenchmarkId::new("euclidean_f32/own_simd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| euclidean_distance(black_box(&a), black_box(&b)).expect("valid bench"))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("euclidean_f32/simsimd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    // simsimd provides l2sq; sqrt to match Euclidean distance
                    SpatialSimilarity::l2sq(black_box(&a[..]), black_box(&b[..]))
                        .map(|sq| (sq as f32).sqrt())
                        .expect("valid bench")
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("euclidean_f32/scalar_autovec", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| euclidean_distance_scalar(black_box(&a), black_box(&b)))
            },
        );

        // --- Cosine ---
        group.bench_with_input(
            BenchmarkId::new("cosine_f32/own_simd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| cosine_distance(black_box(&a), black_box(&b)).expect("valid bench"))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("cosine_f32/simsimd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    SpatialSimilarity::cos(black_box(&a[..]), black_box(&b[..]))
                        .map(|c| c as f32)
                        .expect("valid bench")
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("cosine_f32/scalar_autovec", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| cosine_distance_scalar(black_box(&a), black_box(&b)))
            },
        );

        // --- Dot Product ---
        group.bench_with_input(
            BenchmarkId::new("dot_product_f32/own_simd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| dot_product_distance(black_box(&a), black_box(&b)).expect("valid bench"))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dot_product_f32/simsimd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    SpatialSimilarity::dot(black_box(&a[..]), black_box(&b[..]))
                        .map(|d| d as f32)
                        .expect("valid bench")
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dot_product_f32/scalar_autovec", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| dot_product_scalar(black_box(&a), black_box(&b)))
            },
        );

        // --- Zero-Copy Byte and u8 Metrics (Own Implementation) ---
        group.bench_with_input(
            BenchmarkId::new("cosine_f32_bytes/own_simd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    cosine_distance_f32_bytes(black_box(&a), black_box(&b_bytes)).expect("valid bench")
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("euclidean_f32_bytes/own_simd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    euclidean_distance_f32_bytes(black_box(&a), black_box(&b_bytes)).expect("valid bench")
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dot_product_f32_bytes/own_simd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    dot_product_distance_f32_bytes(black_box(&a), black_box(&b_bytes)).expect("valid bench")
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dot_product_u8/own_simd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| dot_product_u8(black_box(&u8_a), black_box(&u8_b)).expect("valid bench"))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("euclidean_sq_u8/own_simd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    euclidean_distance_sq_u8(black_box(&u8_a), black_box(&u8_b)).expect("valid bench")
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_simd_distances);
criterion_main!(benches);
