use std::collections::HashSet;
use std::error::Error;

use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use rand::Rng;
use rand::SeedableRng;

/// Computes distance using independent scalar logic for ground-truth calculation.
///
/// Ground truth MUST NOT rely on the vector index's own SIMD kernels to ensure
/// true differential / reference test isolation.
fn compute_distance_scalar(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::Cosine => {
            let mut dot = 0.0f32;
            let mut norm_a = 0.0f32;
            let mut norm_b = 0.0f32;
            for (&x, &y) in a.iter().zip(b.iter()) {
                dot += x * y;
                norm_a += x * x;
                norm_b += y * y;
            }
            if norm_a == 0.0 || norm_b == 0.0 {
                1.0
            } else {
                1.0 - (dot / (norm_a.sqrt() * norm_b.sqrt()))
            }
        }
        DistanceMetric::Euclidean => {
            let mut sum_sq = 0.0f32;
            for (&x, &y) in a.iter().zip(b.iter()) {
                let diff = x - y;
                sum_sq += diff * diff;
            }
            sum_sq.sqrt()
        }
        DistanceMetric::DotProduct => {
            let mut dot = 0.0f32;
            for (&x, &y) in a.iter().zip(b.iter()) {
                dot += x * y;
            }
            -dot
        }
        other => panic!("Unsupported metric for brute force reference: {other:?}"),
    }
}

/// Exact O(N) brute force k-NN ground truth scan.
fn brute_force_knn(
    data: &[Vec<f32>],
    query: &[f32],
    k: usize,
    metric: DistanceMetric,
) -> Vec<usize> {
    let mut scored: Vec<(usize, f32)> = data
        .iter()
        .enumerate()
        .map(|(i, v)| (i, compute_distance_scalar(query, v, metric)))
        .collect();
    scored.sort_by(|a, b| a.1.total_cmp(&b.1));
    scored.into_iter().take(k).map(|(i, _)| i).collect()
}

/// Helper to build and query index against brute force ground truth.
async fn run_recall_test(
    metric: DistanceMetric,
    k: usize,
    num_vectors: usize,
    num_queries: usize,
    dim: usize,
    seed: u64,
) -> Result<f64, Box<dyn Error>> {
    // Chosen HNSW Configuration Parameters:
    // - m = 32: High node connection degree ensuring strong graph connectivity across 5000 vectors.
    // - ef_construction = 200: High candidate pool size during graph construction to optimize edge quality.
    // - ef_search = 100: Ample dynamic search candidate pool to achieve high recall accuracy (>= 0.95 / 0.99).
    let config = HnswConfig {
        dimension: dim,
        max_elements: num_vectors + 1000,
        m: 32,
        ef_construction: 200,
        ef_search: 100,
        distance_metric: metric,
        quantize: false,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config)?;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let mut data = Vec::with_capacity(num_vectors);
    for _ in 0..num_vectors {
        let v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
        data.push(v);
    }

    let tx = TxId::new(1);
    for (i, v) in data.iter().enumerate() {
        index.insert(tx, DocId::new(i as u64), v).await?;
    }
    index.commit(tx).await?;

    let mut total_recall = 0.0;
    for _ in 0..num_queries {
        let query: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();

        let ground_truth_indices = brute_force_knn(&data, &query, k, metric);
        let ground_truth_set: HashSet<u64> =
            ground_truth_indices.iter().map(|&i| i as u64).collect();

        let hnsw_results = index.search(&query, k).await?;
        let hits = hnsw_results
            .iter()
            .filter(|r| ground_truth_set.contains(&r.doc_id.inner()))
            .count();

        total_recall += hits as f64 / k as f64;
    }

    Ok(total_recall / num_queries as f64)
}

#[tokio::test]
async fn recall_at_10_vs_brute_force_cosine() -> Result<(), Box<dyn Error>> {
    let avg_recall = run_recall_test(DistanceMetric::Cosine, 10, 5000, 200, 128, 42).await?;
    println!("Cosine Average Recall@10: {:.4}", avg_recall);
    assert!(
        avg_recall >= 0.95,
        "Cosine Recall@10 too low: {:.4} (expected >= 0.95)",
        avg_recall
    );
    Ok(())
}

#[tokio::test]
async fn recall_at_10_vs_brute_force_euclidean() -> Result<(), Box<dyn Error>> {
    let avg_recall = run_recall_test(DistanceMetric::Euclidean, 10, 5000, 200, 128, 43).await?;
    println!("Euclidean Average Recall@10: {:.4}", avg_recall);
    assert!(
        avg_recall >= 0.95,
        "Euclidean Recall@10 too low: {:.4} (expected >= 0.95)",
        avg_recall
    );
    Ok(())
}

#[tokio::test]
async fn recall_at_10_vs_brute_force_dot_product() -> Result<(), Box<dyn Error>> {
    let avg_recall = run_recall_test(DistanceMetric::DotProduct, 10, 5000, 200, 128, 44).await?;
    println!("DotProduct Average Recall@10: {:.4}", avg_recall);
    assert!(
        avg_recall >= 0.95,
        "DotProduct Recall@10 too low: {:.4} (expected >= 0.95)",
        avg_recall
    );
    Ok(())
}

#[tokio::test]
async fn recall_at_1_vs_brute_force() -> Result<(), Box<dyn Error>> {
    let avg_recall = run_recall_test(DistanceMetric::Cosine, 1, 5000, 200, 128, 45).await?;
    println!("Average Recall@1: {:.4}", avg_recall);
    assert!(
        avg_recall >= 0.99,
        "Recall@1 too low: {:.4} (expected >= 0.99)",
        avg_recall
    );
    Ok(())
}
