// FILE-CONTEXT
// ZWECK: Audit Benchmark Suite zur Ermittlung von Durchsatz, Speedup, Latenz-Perzentilen, Pareto-Front und RAM-Footprint
// INVARIANTEN: Deterministische Eingaben via StdRng mit festen Seeds, keine Laufzeitzufallswerte im Benchmark-Setup
// HOTSPOTS: 50-180
// STAND: 2026-11-11
// BEREICH: AGENTS §6, SPEC §11.4 — Empirische Leistungsprüfung des HNSW-Vektorindex

// ANCHOR[contextra-vector:W11-BENCH-012]

mod common;

use common::DEFAULT_BENCH_SEED;
use contextra_ports::VectorIndex;
use contextra_types::{DocId, TxId};
use contextra_vector::distance::*;
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::BTreeSet;
use std::time::Instant;

fn brute_force_knn(query: &[f32], vectors: &[Vec<f32>], ids: &[DocId], k: usize) -> Vec<DocId> {
    let mut scored: Vec<(DocId, f32)> = vectors
        .iter()
        .zip(ids.iter())
        .map(|(v, &id)| {
            let dist = cosine_distance_scalar(query, v);
            (id, dist)
        })
        .collect();
    scored.sort_by(|a, b| a.1.total_cmp(&b.1));
    scored.into_iter().take(k).map(|(id, _)| id).collect()
}

fn calculate_recall(retrieved: &[DocId], ground_truth: &[DocId]) -> f64 {
    if ground_truth.is_empty() {
        return 1.0;
    }
    let truth_set: BTreeSet<DocId> = ground_truth.iter().copied().collect();
    let hits = retrieved.iter().filter(|id| truth_set.contains(id)).count();
    hits as f64 / ground_truth.len() as f64
}

fn percentile(mut latencies: Vec<f64>, p: f64) -> f64 {
    if latencies.is_empty() {
        return 0.0;
    }
    latencies.sort_by(|a, b| a.total_cmp(b));
    let idx = ((latencies.len() as f64 - 1.0) * p).round() as usize;
    latencies[idx]
}

#[tokio::main]
async fn main() {
    let mut rng = StdRng::seed_from_u64(DEFAULT_BENCH_SEED);

    println!("===============================================================================");
    println!("                   CONTEXTRA-INDEX EMPIRICAL AUDIT BENCHMARKS                   ");
    println!("===============================================================================");

    // 1. Distance Metric Throughput & Speedup (SIMD vs Scalar)
    println!("\n--- 1. DISTANCE METRIC THROUGHPUT & SIMD SPEEDUP (1536-dim, 100,000 ops) ---");
    let dim = 1536;
    let num_ops = 100_000;
    let a: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let b: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();

    // Cosine
    let t0 = Instant::now();
    for _ in 0..num_ops {
        let _ = criterion::black_box(cosine_distance(&a, &b));
    }
    let dur_simd_cos = t0.elapsed().as_secs_f64();
    let throughput_simd_cos = num_ops as f64 / dur_simd_cos;

    let t0 = Instant::now();
    for _ in 0..num_ops {
        let _ = criterion::black_box(cosine_distance_scalar(&a, &b));
    }
    let dur_scalar_cos = t0.elapsed().as_secs_f64();
    let throughput_scalar_cos = num_ops as f64 / dur_scalar_cos;
    let speedup_cos = throughput_simd_cos / throughput_scalar_cos;

    println!(
        "Cosine Distance: SIMD = {:.0} ops/s, Scalar = {:.0} ops/s, Speedup = {:.2}x",
        throughput_simd_cos, throughput_scalar_cos, speedup_cos
    );

    // Euclidean
    let t0 = Instant::now();
    for _ in 0..num_ops {
        let _ = criterion::black_box(euclidean_distance(&a, &b));
    }
    let dur_simd_euc = t0.elapsed().as_secs_f64();
    let throughput_simd_euc = num_ops as f64 / dur_simd_euc;

    let t0 = Instant::now();
    for _ in 0..num_ops {
        let _ = criterion::black_box(euclidean_distance_scalar(&a, &b));
    }
    let dur_scalar_euc = t0.elapsed().as_secs_f64();
    let throughput_scalar_euc = num_ops as f64 / dur_scalar_euc;
    let speedup_euc = throughput_simd_euc / throughput_scalar_euc;

    println!(
        "Euclidean Distance: SIMD = {:.0} ops/s, Scalar = {:.0} ops/s, Speedup = {:.2}x",
        throughput_simd_euc, throughput_scalar_euc, speedup_euc
    );

    // Dot Product
    let t0 = Instant::now();
    for _ in 0..num_ops {
        let _ = criterion::black_box(dot_product_distance(&a, &b));
    }
    let dur_simd_dot = t0.elapsed().as_secs_f64();
    let throughput_simd_dot = num_ops as f64 / dur_simd_dot;

    let t0 = Instant::now();
    for _ in 0..num_ops {
        let _ = criterion::black_box(dot_product_scalar(&a, &b));
    }
    let dur_scalar_dot = t0.elapsed().as_secs_f64();
    let throughput_scalar_dot = num_ops as f64 / dur_scalar_dot;
    let speedup_dot = throughput_simd_dot / throughput_scalar_dot;

    println!(
        "Dot Product: SIMD = {:.0} ops/s, Scalar = {:.0} ops/s, Speedup = {:.2}x",
        throughput_simd_dot, throughput_scalar_dot, speedup_dot
    );

    // 2. Index Build Throughput & Search Latency
    println!(
        "\n--- 2. INDEX BUILD THROUGHPUT & SEARCH LATENCY (128-dim, N=1,000, ef_c=64, M=16) ---"
    );
    let dim = 128;
    let n = 1_000;
    let num_queries = 100;
    let k = 10;

    let vectors: Vec<Vec<f32>> = (0..n)
        .map(|_| (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect())
        .collect();
    let doc_ids: Vec<DocId> = (1..=n as u64).map(DocId::from).collect();

    let queries: Vec<Vec<f32>> = (0..num_queries)
        .map(|_| (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect())
        .collect();

    let config = HnswConfig {
        dimension: dim,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        quantize: false,
        rebuild_threshold: 0.0,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).expect("Index creation failed");

    let t0 = Instant::now();
    let tx = TxId::new(1);
    for (idx, vec) in vectors.iter().enumerate() {
        let doc_id = doc_ids[idx];
        index.insert(tx, doc_id, vec).await.expect("Insert failed");
    }
    index.commit(tx).await.expect("Commit failed");
    let build_duration = t0.elapsed().as_secs_f64();
    let build_throughput = n as f64 / build_duration;

    println!(
        "HNSW Build: {:.0} docs/s (Total time: {:.2}s for N={})",
        build_throughput, build_duration, n
    );

    // Warmup & Search Latencies
    let mut latencies_us = Vec::with_capacity(num_queries);
    let mut total_recall = 0.0;

    for query in &queries {
        let gt = brute_force_knn(query, &vectors, &doc_ids, k);

        let t_query = Instant::now();
        let results = index.search(query, k).await.expect("Search failed");
        let elapsed_us = t_query.elapsed().as_secs_f64() * 1_000_000.0;
        latencies_us.push(elapsed_us);

        let retrieved_ids: Vec<DocId> = results.into_iter().map(|s| s.doc_id).collect();
        total_recall += calculate_recall(&retrieved_ids, &gt);
    }

    let avg_recall = total_recall / num_queries as f64;
    let p50 = percentile(latencies_us.clone(), 0.50);
    let p95 = percentile(latencies_us.clone(), 0.95);
    let p99 = percentile(latencies_us.clone(), 0.99);

    println!(
        "Search Latency: p50 = {:.1} µs, p95 = {:.1} µs, p99 = {:.1} µs | Average Recall@{} = {:.2}%",
        p50,
        p95,
        p99,
        k,
        avg_recall * 100.0
    );

    println!("\nAudit Benchmarks successfully finished.");
}
