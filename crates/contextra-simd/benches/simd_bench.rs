#![allow(unsafe_code)]

use contextra_simd::dispatch::{cosine_distance, dot_product_distance, euclidean_distance};
use contextra_simd::kernels::scalar::{
    cosine_distance_scalar, dot_product_scalar, euclidean_distance_scalar,
};

#[cfg(target_arch = "x86_64")]
use contextra_simd::kernels::avx2::{
    cosine_distance_avx2, dot_product_avx2, euclidean_distance_avx2,
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

        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            group.bench_with_input(BenchmarkId::new("cosine_avx2", dim), &dim, |b_bench, _| {
                b_bench.iter(|| unsafe { cosine_distance_avx2(black_box(&a), black_box(&b)) })
            });
        }

        group.bench_with_input(
            BenchmarkId::new("cosine_dispatch", dim),
            &dim,
            |b_bench, _| b_bench.iter(|| cosine_distance(black_box(&a), black_box(&b)).unwrap()),
        );

        // Euclidean
        group.bench_with_input(
            BenchmarkId::new("euclidean_scalar", dim),
            &dim,
            |b_bench, _| b_bench.iter(|| euclidean_distance_scalar(black_box(&a), black_box(&b))),
        );

        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            group.bench_with_input(
                BenchmarkId::new("euclidean_avx2", dim),
                &dim,
                |b_bench, _| {
                    b_bench
                        .iter(|| unsafe { euclidean_distance_avx2(black_box(&a), black_box(&b)) })
                },
            );
        }

        group.bench_with_input(
            BenchmarkId::new("euclidean_dispatch", dim),
            &dim,
            |b_bench, _| b_bench.iter(|| euclidean_distance(black_box(&a), black_box(&b)).unwrap()),
        );

        // Dot Product
        group.bench_with_input(
            BenchmarkId::new("dot_product_scalar", dim),
            &dim,
            |b_bench, _| b_bench.iter(|| dot_product_scalar(black_box(&a), black_box(&b))),
        );

        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            group.bench_with_input(
                BenchmarkId::new("dot_product_avx2", dim),
                &dim,
                |b_bench, _| {
                    b_bench.iter(|| unsafe { dot_product_avx2(black_box(&a), black_box(&b)) })
                },
            );
        }

        group.bench_with_input(
            BenchmarkId::new("dot_product_dispatch", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| dot_product_distance(black_box(&a), black_box(&b)).unwrap())
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_simd_kernels);
criterion_main!(benches);
