// FILE-CONTEXT
// STAND: 2026-10-22
// ZWECK: ANN-Benchmarks kompatible Evaluation für Contextra HNSW-Index.

use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

use anyhow::Result;
use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::{HnswConfig, HnswIndex};
use hdrhistogram::Histogram;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Configurable dataset generator for synthetic vector indexing evaluation.
pub struct SyntheticDataset {
    pub dimension: usize,
    pub num_vectors: usize,
    pub num_queries: usize,
    pub metric: DistanceMetric,
}

impl SyntheticDataset {
    pub fn new(
        dimension: usize,
        num_vectors: usize,
        num_queries: usize,
        metric: DistanceMetric,
    ) -> Self {
        Self {
            dimension,
            num_vectors,
            num_queries,
            metric,
        }
    }

    /// Generates normalized or unnormalized random vectors deterministically.
    pub fn generate_vectors(&self, seed: u64) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut base_vectors = Vec::with_capacity(self.num_vectors);

        for _ in 0..self.num_vectors {
            let mut v = Vec::with_capacity(self.dimension);
            for _ in 0..self.dimension {
                v.push(rng.gen_range(-1.0..1.0));
            }
            if matches!(self.metric, DistanceMetric::Cosine) {
                normalize_in_place(&mut v);
            }
            base_vectors.push(v);
        }

        let mut query_vectors = Vec::with_capacity(self.num_queries);
        for _ in 0..self.num_queries {
            let mut v = Vec::with_capacity(self.dimension);
            for _ in 0..self.dimension {
                v.push(rng.gen_range(-1.0..1.0));
            }
            if matches!(self.metric, DistanceMetric::Cosine) {
                normalize_in_place(&mut v);
            }
            query_vectors.push(v);
        }

        (base_vectors, query_vectors)
    }
}

fn normalize_in_place(v: &mut [f32]) {
    let norm_sq: f32 = v.iter().map(|x| x * x).sum();
    if norm_sq > 0.0 {
        let norm = norm_sq.sqrt();
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

/// Metrics output from ANN benchmark execution.
#[derive(Debug, Clone)]
pub struct AnnBenchmarkReport {
    pub build_time_secs: f64,
    pub qps: f64,
    pub recall_at_k: f64,
    pub latency_p50_us: u64,
    pub latency_p95_us: u64,
    pub latency_p99_us: u64,
}

/// Computes exact Ground Truth Top-K using brute-force scanning.
pub fn compute_ground_truth(
    base_vectors: &[Vec<f32>],
    query: &[f32],
    k: usize,
    metric: DistanceMetric,
) -> HashSet<u64> {
    let mut distances: Vec<(u64, f32)> = base_vectors
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            let dist = compute_distance(v, query, metric);
            (idx as u64, dist)
        })
        .collect();

    // Distance metrics in HNSW: smaller distance is better
    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    distances.iter().take(k).map(|(idx, _)| *idx).collect()
}

fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::L2 => a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum(),
        DistanceMetric::Cosine | DistanceMetric::DotProduct => {
            // Negative dot product so smaller is better
            -a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>()
        }
    }
}

/// Runs ANN Benchmark on HnswIndex for given dataset parameters.
pub async fn run_ann_benchmark(
    dataset: &SyntheticDataset,
    k: usize,
    ef_construction: usize,
    m: usize,
    ef_search: usize,
    dir: &Path,
) -> Result<AnnBenchmarkReport> {
    let (base_vecs, query_vecs) = dataset.generate_vectors(42);

    let config = HnswConfig {
        dimension: dataset.dimension,
        metric: dataset.metric,
        m,
        ef_construction,
        ef_search,
    };

    let index_path = dir.join("hnsw_ann.index");
    let mut index = HnswIndex::open_or_create(&index_path, config)?;

    let build_start = Instant::now();
    for (i, v) in base_vecs.iter().enumerate() {
        let doc_id = DocId::from_u64(i as u64);
        let tx_id = TxId::try_new(i as u64 + 1).unwrap();
        index.insert(doc_id, v, tx_id).await?;
    }
    let build_time_secs = build_start.elapsed().as_secs_f64();

    let mut histogram = Histogram::<u64>::new_with_bounds(1, 1_000_000, 3)?;
    let mut total_recall = 0.0;

    let search_start = Instant::now();
    for query in &query_vecs {
        let gt = compute_ground_truth(&base_vecs, query, k, dataset.metric);

        let q_start = Instant::now();
        let results = index.search(query, k).await?;
        let elapsed_us = q_start.elapsed().as_micros() as u64;
        let _ = histogram.record(elapsed_us.max(1));

        let retrieved: HashSet<u64> = results.iter().map(|(doc, _)| doc.as_u64()).collect();
        let hits = gt.intersection(&retrieved).count();
        total_recall += hits as f64 / k as f64;
    }
    let search_time_secs = search_start.elapsed().as_secs_f64();

    let qps = dataset.num_queries as f64 / search_time_secs;
    let recall_at_k = total_recall / dataset.num_queries as f64;

    Ok(AnnBenchmarkReport {
        build_time_secs,
        qps,
        recall_at_k,
        latency_p50_us: histogram.value_at_quantile(0.50),
        latency_p95_us: histogram.value_at_quantile(0.95),
        latency_p99_us: histogram.value_at_quantile(0.99),
    })
}
