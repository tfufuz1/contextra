// FILE-CONTEXT
// ZWECK: Benchmarks for SIMD distance kernels.

//! # Benchmark-Suite: `simd_bench`
//!
//! **Zweck:**
//! Misst Durchsatz und Latenz der SIMD-Distanzberechnungen (`cosine_distance`,
//! `euclidean_distance`, `dot_product_distance`, `*_f32_bytes`, `*_u8`)
//! unter Verwendung des Criterion-Frameworks.
//!
//! **Laufzeit-Kernel-Auswahl:**
//! Der Dispatcher ermittelt die verfügbaren CPU-Erweiterungen einmalig dynamisch
//! (x86_64: AVX-512 VNNI / AVX-512F -> AVX2 + FMA -> Skalar; AArch64: NEON -> Skalar).
//!
//! **Getestete Dimensionen:**
//! - 128 Dimensionen: Kleine Embedding-Modelle (z. B. MiniLM-L6) und Audio-Features.
//! - 384 Dimensionen: All-MiniLM-L6-v2, BGE-small.
//! - 768 Dimensionen: BERT-base, Nomic-Embed-Text, E5-base.
//! - 1536 Dimensionen: OpenAI text-embedding-3-small, ada-002.

#![allow(unsafe_code)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use contextra_simd::dispatch::{
    cosine_distance, cosine_distance_f32_bytes, dot_product_distance,
    dot_product_distance_f32_bytes, dot_product_u8, euclidean_distance,
    euclidean_distance_f32_bytes, euclidean_distance_sq_u8,
};

// Benchmark-Ablauf:
// 1. Initialisiert synthetische Testvektoren mit deterministischen Werten für Dimensionen 128, 384, 768, 1024, 1536, 3072.
// 2. Misst die Ausführungszeit und Durchsatz (Elements/sec) von f32-, byte-optimierten und u8-Distanzberechnungen im Criterion-Loop.
// 3. Verhindert mit `black_box`, dass der Compiler konstante Berechnungen wegoptimiert.
fn bench_simd_distances(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_distances");

    for dim in [128, 384, 768, 1024, 1536, 3072] {
        group.throughput(Throughput::Elements(dim as u64));
        let a: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.01 + 0.1).collect();
        let b: Vec<f32> = (0..dim).map(|i| ((i + 1) as f32) * 0.01 + 0.2).collect();
        let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

        let u8_a: Vec<u8> = (0..dim).map(|i| (i % 256) as u8).collect();
        let u8_b: Vec<u8> = (0..dim).map(|i| ((i + 1) % 256) as u8).collect();

        group.bench_with_input(
            criterion::BenchmarkId::new("cosine_f32", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| cosine_distance(black_box(&a), black_box(&b)).expect("valid bench"))
            },
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("euclidean_f32", dim),
            &dim,
            |b_bench, _| {
                b_bench
                    .iter(|| euclidean_distance(black_box(&a), black_box(&b)).expect("valid bench"))
            },
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("dot_product_f32", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    dot_product_distance(black_box(&a), black_box(&b)).expect("valid bench")
                })
            },
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("cosine_f32_bytes", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    cosine_distance_f32_bytes(black_box(&a), black_box(&b_bytes))
                        .expect("valid bench")
                })
            },
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("euclidean_f32_bytes", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    euclidean_distance_f32_bytes(black_box(&a), black_box(&b_bytes))
                        .expect("valid bench")
                })
            },
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("dot_product_f32_bytes", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    dot_product_distance_f32_bytes(black_box(&a), black_box(&b_bytes))
                        .expect("valid bench")
                })
            },
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("dot_product_u8", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    dot_product_u8(black_box(&u8_a), black_box(&u8_b)).expect("valid bench")
                })
            },
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("euclidean_sq_u8", dim),
            &dim,
            |b_bench, _| {
                b_bench.iter(|| {
                    euclidean_distance_sq_u8(black_box(&u8_a), black_box(&u8_b))
                        .expect("valid bench")
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_simd_distances);
criterion_main!(benches);
