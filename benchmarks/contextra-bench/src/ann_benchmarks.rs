// FILE-CONTEXT
// STAND: 2026-09-15
// ZWECK: ANN-Benchmarks kompatible Evaluation für Contextra HNSW-Index.

use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

use anyhow::Result;
use contextra_core::traits::VectorIndex;
use contextra_core::types::{DistanceMetric, DocId, TxId};
use contextra_vector::{HnswConfig, HnswIndex};
use hdrhistogram::Histogram;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnBenchResult {
    pub dataset_name: String,
    pub n_vectors: usize,
    pub dimension: usize,
    pub k: usize,
    /// QPS bei verschiedenen Recall-Schwellwerten
    pub qps_at_recall_90: f64,
    pub qps_at_recall_95: f64,
    pub qps_at_recall_99: f64,
    /// Maximaler Recall (bei ef=max_ef_search)
    pub max_recall_at_10: f64,
    pub p50_latency_ms: f64,
    pub p99_latency_ms: f64,
}

/// Synthetische Vektoren (SIFT-ähnlich, uniform-random in [0..127]^dim für CI ohne HDF5)
pub fn generate_synthetic_vectors(n: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::with_capacity(n);

    for _ in 0..n {
        let mut v = Vec::with_capacity(dim);
        for _ in 0..dim {
            v.push(rng.gen_range(0.0..127.0));
        }
        vectors.push(v);
    }

    vectors
}

fn euclidean_distance_sq(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| {
            let diff = x - y;
            diff * diff
        })
        .sum()
}

fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let norm = (norm_a.sqrt() * norm_b.sqrt()).max(1e-9);
    1.0 - (dot / norm)
}

fn dot_product_distance(a: &[f32], b: &[f32]) -> f32 {
    -a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum::<f32>()
}

/// Misst Ground-Truth Nearest Neighbors via Brute-Force
pub fn compute_ground_truth(
    base: &[Vec<f32>],
    queries: &[Vec<f32>],
    k: usize,
    metric: DistanceMetric,
) -> Vec<Vec<usize>> {
    let mut ground_truth = Vec::with_capacity(queries.len());

    for q in queries {
        let mut distances: Vec<(usize, f32)> = base
            .iter()
            .enumerate()
            .map(|(idx, b)| {
                let dist = match metric {
                    DistanceMetric::Euclidean => euclidean_distance_sq(q, b),
                    DistanceMetric::Cosine => cosine_distance(q, b),
                    DistanceMetric::DotProduct => dot_product_distance(q, b),
                    _ => euclidean_distance_sq(q, b),
                };
                (idx, dist)
            })
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_k: Vec<usize> = distances.iter().take(k).map(|(idx, _)| *idx).collect();
        ground_truth.push(top_k);
    }

    ground_truth
}

/// Berechnet Recall@k: |retrieved ∩ ground_truth| / k
pub fn recall_at_k(retrieved: &[u64], ground_truth: &[usize], k: usize) -> f64 {
    if k == 0 || ground_truth.is_empty() {
        return 0.0;
    }

    let limit = k.min(retrieved.len());
    let gt_limit = k.min(ground_truth.len());

    let gt_set: HashSet<u64> = ground_truth[..gt_limit]
        .iter()
        .map(|&idx| idx as u64)
        .collect();
    let mut hits = 0usize;

    for &id in &retrieved[..limit] {
        if gt_set.contains(&id) {
            hits += 1;
        }
    }

    (hits as f64) / (gt_limit as f64)
}

/// Führt ANN-Benchmark durch: variiert ef_search, misst QPS vs Recall Trade-off
pub async fn run_ann_benchmark(
    base_vectors: Vec<Vec<f32>>,
    query_vectors: Vec<Vec<f32>>,
    ground_truth: Vec<Vec<usize>>,
    dataset_name: &str,
    m: usize,
    ef_construction: usize,
    ef_search_values: &[usize],
) -> Result<AnnBenchResult> {
    let n_vectors = base_vectors.len();
    let dimension = base_vectors.first().map(|v| v.len()).unwrap_or(128);
    let k = 10;

    let mut qps_recalls: Vec<(f64, f64)> = Vec::new();
    let mut max_recall = 0.0f64;
    let mut overall_hist = Histogram::<u64>::new_with_bounds(1, 60_000_000, 3)?;

    for &ef in ef_search_values {
        let config = HnswConfig {
            dimension,
            max_elements: n_vectors + 100,
            m,
            ef_construction,
            ef_search: ef,
            distance_metric: DistanceMetric::Euclidean,
            ..Default::default()
        };

        let index = HnswIndex::try_new(config)?;
        let tx = TxId(1);

        for (idx, vec) in base_vectors.iter().enumerate() {
            index.insert(tx, DocId(idx as u64), vec).await?;
        }
        index.commit(tx).await?;

        let mut ef_hist = Histogram::<u64>::new_with_bounds(1, 60_000_000, 3)?;
        let mut total_recall = 0.0f64;

        let start_all = Instant::now();

        for (q_idx, q) in query_vectors.iter().enumerate() {
            let q_start = Instant::now();
            let results = index.search(q, k).await?;
            let elapsed_us = q_start.elapsed().as_micros() as u64;

            let _ = ef_hist.record(elapsed_us.max(1));
            let _ = overall_hist.record(elapsed_us.max(1));

            let retrieved_ids: Vec<u64> = results.into_iter().map(|r| r.doc_id.0).collect();
            let rec = recall_at_k(&retrieved_ids, &ground_truth[q_idx], k);
            total_recall += rec;
        }

        let total_time_sec = start_all.elapsed().as_secs_f64();
        let qps = if total_time_sec > 0.0 {
            (query_vectors.len() as f64) / total_time_sec
        } else {
            0.0
        };

        let mean_recall = if !query_vectors.is_empty() {
            total_recall / (query_vectors.len() as f64)
        } else {
            0.0
        };

        if mean_recall > max_recall {
            max_recall = mean_recall;
        }

        qps_recalls.push((qps, mean_recall));
    }

    let find_qps_for_recall = |target_recall: f64| -> f64 {
        let mut best_qps = 0.0;
        for &(qps, recall) in &qps_recalls {
            if recall >= target_recall && qps > best_qps {
                best_qps = qps;
            }
        }
        best_qps
    };

    let qps_90 = find_qps_for_recall(0.90);
    let qps_95 = find_qps_for_recall(0.95);
    let qps_99 = find_qps_for_recall(0.99);

    let res = AnnBenchResult {
        dataset_name: dataset_name.to_string(),
        n_vectors,
        dimension,
        k,
        qps_at_recall_90: qps_90,
        qps_at_recall_95: qps_95,
        qps_at_recall_99: qps_99,
        max_recall_at_10: max_recall,
        p50_latency_ms: (overall_hist.value_at_quantile(0.50) as f64) / 1000.0,
        p99_latency_ms: (overall_hist.value_at_quantile(0.99) as f64) / 1000.0,
    };

    // Save to benchmarks/results/ann_results.json per APM-5
    let results_dir = Path::new("benchmarks/results");
    if !results_dir.exists() {
        std::fs::create_dir_all(results_dir)?;
    }
    let json = serde_json::to_string_pretty(&res)?;
    std::fs::write(results_dir.join("ann_results.json"), json)?;

    Ok(res)
}

/// Schnelle CI-Version: 1k Vektoren, D=128 (läuft in <10s)
pub async fn run_ann_benchmark_synthetic_ci() -> Result<AnnBenchResult> {
    let base = generate_synthetic_vectors(1_000, 128, 42);
    let queries = generate_synthetic_vectors(50, 128, 99);
    let gt = compute_ground_truth(&base, &queries, 10, DistanceMetric::Euclidean);
    run_ann_benchmark(
        base,
        queries,
        gt,
        "synthetic-1k-128d",
        16,
        100,
        &[10, 50, 100, 200],
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_synthetic_vectors_and_ground_truth() {
        let base = generate_synthetic_vectors(100, 32, 1);
        let queries = generate_synthetic_vectors(10, 32, 2);
        assert_eq!(base.len(), 100);
        assert_eq!(queries.len(), 10);

        let gt = compute_ground_truth(&base, &queries, 5, DistanceMetric::Euclidean);
        assert_eq!(gt.len(), 10);
        assert_eq!(gt[0].len(), 5);

        let retrieved = vec![gt[0][0] as u64, gt[0][1] as u64];
        let recall = recall_at_k(&retrieved, &gt[0], 5);
        assert_eq!(recall, 2.0 / 5.0);
    }
}
