use contextra_core::DistanceMetric;
use contextra_vector::distance::{compute_distance, cosine_distance};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_distance_nan_impact(c: &mut Criterion) {
    let dim = 768;
    let a: Vec<f32> = (0..dim).map(|i| (i as f32 * 0.01).sin()).collect();
    let b: Vec<f32> = (0..dim).map(|i| (i as f32 * 0.01).cos()).collect();

    let mut group = c.benchmark_group("distance_nan_impact");

    group.bench_function("compute_distance_no_nan_scan", |bench| {
        bench.iter(|| {
            black_box(compute_distance(
                black_box(&a),
                black_box(&b),
                DistanceMetric::Cosine,
            ))
        });
    });

    group.bench_function("cosine_distance_direct", |bench| {
        bench.iter(|| black_box(cosine_distance(black_box(&a), black_box(&b))));
    });

    group.finish();
}

criterion_group!(benches, bench_distance_nan_impact);
criterion_main!(benches);
