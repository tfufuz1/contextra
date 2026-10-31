// FILE-CONTEXT
// ZWECK: Benchmarks for SIMD distance kernels.

//! # Benchmark-Suite: `simd_bench`
//!
//! **Zweck:**
//! Misst Durchsatz und Latenz der SIMD-Distanzberechnungen (`cosine_distance`,
//! `euclidean_distance`, `dot_product_distance`) unter Verwendung des Criterion-Frameworks.
//!
//! **Getestete Dimensionen:**
//! - 128 Dimensionen: Typisch für kleine Embedding-Modelle (z. B. MiniLM) und Audio-Features.
//! - 512 Dimensionen: Mittlere Text-Embeddings und CLIP-Bildvektoren.
//! - 1536 Dimensionen: Große Standard-Embeddings (z. B. OpenAI text-embedding-3-small/ada-002).

#![allow(unsafe_code)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use contextra_simd::dispatch::{cosine_distance, dot_product_distance, euclidean_distance};

// ANCHOR[contextra-simd:BENCH-SIMD-DISTANCES]
// Benchmark-Ablauf:
// 1. Initialisiert synthetische Testvektoren mit aufsteigenden Werten für Dimensionen 128, 512 und 1536.
// 2. Misst die Ausführungszeit von `cosine_distance`, `euclidean_distance` und `dot_product_distance` im Criterion-Loop.
// 3. Verhindert mit `black_box`, dass der Compiler konstante Berechnungen wegoptimiert.
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
