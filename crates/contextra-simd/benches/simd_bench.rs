// FILE-CONTEXT
// ZWECK: Benchmarks for SIMD vs. Scalar distance kernels with throughput measurement.

//! # Benchmark-Suite: `simd_bench`
//!
//! **Zweck:**
//! Misst Durchsatz (Elemente/s) und Latenz der Distanzberechnungen (`cosine`,
//! `euclidean`, `dot_product`) für die Skalar-Referenz sowie den dynamischen SIMD-Dispatch.
//! Es werden ausschließlich tatsächliche Messwerte ausgegeben (keine Zielwerte als Messung).
//!
//! **Getestete Dimensionen:**
//! - 128 Dimensionen: Typisch für kleine Embedding-Modelle (z. B. MiniLM) und Audio-Features.
//! - 512 Dimensionen: Mittlere Text-Embeddings und CLIP-Bildvektoren.
//! - 1536 Dimensionen: Große Standard-Embeddings (z. B. OpenAI text-embedding-3-small/ada-002).

#![allow(unsafe_code)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use contextra_simd::dispatch::{cosine_distance, dot_product_distance, euclidean_distance};
use contextra_simd::kernels::scalar::{
    cosine_distance_scalar, dot_product_scalar, euclidean_distance_scalar,
};

// ANCHOR[contextra-simd:BENCH-SIMD-DISTANCES]
// Benchmark-Ablauf:
// 1. Initialisiert synthetische Testvektoren für Dimensionen 128, 512 und 1536.
// 2. Misst die Ausführungszeit von Skalar vs. SIMD-Dispatch im Criterion-Loop.
// 3. Konfiguriert Throughput::Elements(dim as u64) für direkte Vergleichbarkeit des Durchsatzes.
fn bench_simd_distances(c: &mut Criterion) {
    let mut group = c.benchmark_group("distance_kernels");

    for dim in [128usize, 512, 1536] {
        let a: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.01 - 1.0).collect();
        let b: Vec<f32> = (0..dim).map(|i| ((i + 1) as f32) * 0.01 - 1.0).collect();

        group.throughput(Throughput::Elements(dim as u64));

        // --- Cosine ---
        group.bench_with_input(
            criterion::BenchmarkId::new("cosine_scalar", dim),
            &dim,
            |b_bench, _| b_bench.iter(|| cosine_distance_scalar(black_box(&a), black_box(&b))),
        );
        group.bench_with_input(
            criterion::BenchmarkId::new("cosine_simd", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| cosine_distance(black_box(&a), black_box(&b)).expect("valid bench"))
            },
        );

        // --- Euclidean ---
        group.bench_with_input(
            criterion::BenchmarkId::new("euclidean_scalar", dim),
            &dim,
            |b_bench, _| b_bench.iter(|| euclidean_distance_scalar(black_box(&a), black_box(&b))),
        );
        group.bench_with_input(
            criterion::BenchmarkId::new("euclidean_simd", dim),
            &dim,
            |b_bench, _| {
                b_bench
                    .iter(|| euclidean_distance(black_box(&a), black_box(&b)).expect("valid bench"))
            },
        );

        // --- Dot Product ---
        group.bench_with_input(
            criterion::BenchmarkId::new("dot_product_scalar", dim),
            &dim,
            |b_bench, _| b_bench.iter(|| dot_product_scalar(black_box(&a), black_box(&b))),
        );
        group.bench_with_input(
            criterion::BenchmarkId::new("dot_product_simd", dim),
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
