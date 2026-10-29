// FILE-CONTEXT
// ZWECK: Benchmarks for SIMD distance kernels.

#![allow(unsafe_code)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use contextra_simd::dispatch::{cosine_distance, dot_product_distance, euclidean_distance};

fn bench_simd_distances(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_distances");

    for dim in [128, 512, 1536] {
        let a: Vec<f32> = (0..dim).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..dim).map(|i| (i + 1) as f32).collect();

        group.bench_with_input(
            criterion::BenchmarkId::new("cosine", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| cosine_distance(black_box(&a), black_box(&b)).expect("valid bench"))
            },
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("euclidean", dim),
            &dim,
            |b_bench, _| {
                b_bench
                    .iter(|| euclidean_distance(black_box(&a), black_box(&b)).expect("valid bench"))
            },
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("dot_product", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    dot_product_distance(black_box(&a), black_box(&b)).expect("valid bench")
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_simd_distances);
criterion_main!(benches);
