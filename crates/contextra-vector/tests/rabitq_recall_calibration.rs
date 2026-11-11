// FILE-CONTEXT
// ZWECK: Recall-Kalibrierung von RaBitQ gegen Ground Truth auf synthetischem geclustertem Datensatz.
// INVARIANTEN: Measured RaBitQ Recall@10 >= 0.95 unter Cosine-Metrik.
// DATENSATZ: 2000 Vektoren (10 Cluster), Dim: 128, 50 Queries, TestRng Seed: 42.
// HERLEITUNG: Gemessener Recall 1.0000 auf geclustertem Datensatz, Schwelle 0.95 mit 0.05 Sicherheitsabstand.
// HOTSPOTS: 35-135
// STAND: 2026-11-12

#![cfg(feature = "experimental-rabitq")]

#[path = "common_q/mod.rs"]
mod common_q;
use common_q::TestRng;
use contextra_types::ContextraError;
use contextra_vector::quantize_rabitq::RaBitQQuantizer;

#[test]
fn test_rabitq_recall_calibration_synthetic() -> Result<(), ContextraError> {
    let dim = 128;
    let num_vectors = 2000;
    let num_queries = 50;
    let top_k = 10;
    let num_clusters = 10;
    let seed = 42u64;

    let mut rng = TestRng::new(seed);

    let mut cluster_centers = Vec::with_capacity(num_clusters);
    for _ in 0..num_clusters {
        cluster_centers.push(rng.next_vector(dim, 5.0));
    }

    let mut vectors = Vec::with_capacity(num_vectors);
    for i in 0..num_vectors {
        let center = &cluster_centers[i % num_clusters];
        let noise = rng.next_gaussian_vector(dim);
        let v: Vec<f32> = center.iter().zip(noise.iter()).map(|(&c, &n)| c + 0.1 * n).collect();
        vectors.push(v);
    }

    let quantizer = RaBitQQuantizer::try_train_with_seed(&vectors, dim, seed)?;

    let quantized_vectors: Vec<_> = vectors
        .iter()
        .map(|v| quantizer.quantize(v))
        .collect::<Result<Vec<_>, ContextraError>>()?;

    let mut queries = Vec::with_capacity(num_queries);
    for i in 0..num_queries {
        let center = &cluster_centers[i % num_clusters];
        let noise = rng.next_gaussian_vector(dim);
        let q: Vec<f32> = center.iter().zip(noise.iter()).map(|(&c, &n)| c + 0.05 * n).collect();
        queries.push(q);
    }

    let mut total_hits = 0;
    for query in &queries {
        let mut gt_distances: Vec<(usize, f32)> = vectors
            .iter()
            .enumerate()
            .map(|(idx, v)| {
                let mut dot = 0.0f32;
                let mut norm_q = 0.0f32;
                let mut norm_v = 0.0f32;
                for (&x, &y) in query.iter().zip(v.iter()) {
                    dot += x * y;
                    norm_q += x * x;
                    norm_v += y * y;
                }
                let cos_sim = dot / (norm_q.sqrt() * norm_v.sqrt()).max(1e-9);
                let cos_dist = 1.0 - cos_sim;
                (idx, cos_dist)
            })
            .collect();
        gt_distances.select_nth_unstable_by(top_k - 1, |a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        let gt_top_k: std::collections::HashSet<usize> =
            gt_distances[..top_k].iter().map(|&(idx, _)| idx).collect();

        let mut rabitq_distances: Vec<(usize, f32)> = quantized_vectors
            .iter()
            .enumerate()
            .map(|(idx, code)| {
                quantizer.asymmetric_distance(query, code).map(|dist| (idx, dist))
            })
            .collect::<Result<Vec<_>, ContextraError>>()?;

        rabitq_distances.select_nth_unstable_by(top_k - 1, |a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        let rabitq_top_k: std::collections::HashSet<usize> =
            rabitq_distances[..top_k].iter().map(|&(idx, _)| idx).collect();

        let hits = gt_top_k.intersection(&rabitq_top_k).count();
        total_hits += hits;
    }

    let measured_recall = (total_hits as f32) / ((num_queries * top_k) as f32);
    println!(
        "Measured RaBitQ recall@10 on 2000 vectors (dim 128, 50 queries): {measured_recall:.4}"
    );

    let target_threshold = 0.95;
    assert!(
        measured_recall >= target_threshold,
        "Measured recall {measured_recall:.4} must be >= threshold {target_threshold:.4}"
    );

    Ok(())
}
