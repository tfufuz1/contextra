#![allow(unsafe_code)]

use contextra_simd::dispatch::{
    cosine_distance, cosine_distance_with_features, detect, dot_product_distance,
    dot_product_distance_with_features, euclidean_distance, euclidean_distance_with_features,
};
use contextra_simd::kernels::scalar::{
    cosine_distance_scalar, dot_product_scalar, euclidean_distance_scalar,
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

fn bench_simd_kernels(c: &mut Criterion) {
    let dimensions = [128, 512, 1536, 4096];

    let mut group = c.benchmark_group("SIMD_vs_Scalar");
    group.measurement_time(Duration::from_millis(500));
    group.warm_up_time(Duration::from_millis(200));
    group.sample_size(20);

    for &dim in &dimensions {
        let a: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.01 - 1.0).collect();
        let b: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.02 - 0.5).collect();

        // Cosine
        group.bench_with_input(
            BenchmarkId::new("cosine_scalar", dim),
            &dim,
            |b_bench, _| b_bench.iter(|| cosine_distance_scalar(black_box(&a), black_box(&b))),
        );

        let features = detect();
        group.bench_with_input(
            BenchmarkId::new("cosine_with_features", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    let _ = cosine_distance_with_features(black_box(&a), black_box(&b), &features);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("cosine_dispatch", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    let _ = cosine_distance(black_box(&a), black_box(&b));
                })
            },
        );

        // Euclidean
        group.bench_with_input(
            BenchmarkId::new("euclidean_scalar", dim),
            &dim,
            |b_bench, _| b_bench.iter(|| euclidean_distance_scalar(black_box(&a), black_box(&b))),
        );

        group.bench_with_input(
            BenchmarkId::new("euclidean_with_features", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    let _ =
                        euclidean_distance_with_features(black_box(&a), black_box(&b), &features);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("euclidean_dispatch", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    let _ = euclidean_distance(black_box(&a), black_box(&b));
                })
            },
        );

        // Dot Product
        group.bench_with_input(
            BenchmarkId::new("dot_product_scalar", dim),
            &dim,
            |b_bench, _| b_bench.iter(|| dot_product_scalar(black_box(&a), black_box(&b))),
        );

        group.bench_with_input(
            BenchmarkId::new("dot_product_with_features", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    let _ =
                        dot_product_distance_with_features(black_box(&a), black_box(&b), &features);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dot_product_dispatch", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    let _ = dot_product_distance(black_box(&a), black_box(&b));
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_simd_kernels);
criterion_main!(benches);
