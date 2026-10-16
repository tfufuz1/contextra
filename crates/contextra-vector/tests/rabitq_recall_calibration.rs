#![cfg(feature = "experimental-rabitq")]

use contextra_core::ContextraError;
use contextra_vector::quantize_rabitq::RaBitQQuantizer;

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 32) as f32) / 4294967296.0
    }

    fn next_gaussian_f32(&mut self) -> f32 {
        let u1 = self.next_f32().clamp(1e-7, 1.0);
        let u2 = self.next_f32();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
    }
}

#[test]
fn test_rabitq_recall_calibration_synthetic() -> Result<(), ContextraError> {
    let dim = 128;
    let num_vecs = 2000;
    let num_queries = 50;
    let top_k = 10;

    let mut sm = SplitMix64::new(424242);

    // Generate 2,000 synthetic vectors (200 cluster centers x 10 samples)
    let num_clusters = 200;
    let samples_per_cluster = 10;

    let mut cluster_centers = Vec::with_capacity(num_clusters);
    for _ in 0..num_clusters {
        let center: Vec<f32> = (0..dim).map(|_| sm.next_gaussian_f32() * 5.0).collect();
        cluster_centers.push(center);
    }

    let mut vectors = Vec::with_capacity(num_vecs);
    for i in 0..num_vecs {
        let center = &cluster_centers[i / samples_per_cluster];
        let vec: Vec<f32> = center
            .iter()
            .map(|&c| c + sm.next_gaussian_f32() * 0.1)
            .collect();
        vectors.push(vec);
    }

    let quantizer = RaBitQQuantizer::try_train(&vectors, dim)?;
    let codes: Vec<Vec<u8>> = vectors
        .iter()
        .map(|v| quantizer.quantize(v))
        .collect::<Result<Vec<_>, _>>()?;

    // Generate 50 queries from cluster centers
    let mut queries = Vec::with_capacity(num_queries);
    for q_idx in 0..num_queries {
        let center = &cluster_centers[q_idx % num_clusters];
        let query: Vec<f32> = center
            .iter()
            .map(|&c| c + sm.next_gaussian_f32() * 0.1)
            .collect();
        queries.push(query);
    }

    let mut total_hits = 0;

    for query in &queries {
        // Ground truth exact cosine distance
        let mut gt_distances: Vec<(usize, f32)> = vectors
            .iter()
            .enumerate()
            .map(|(idx, v)| {
                let dot: f32 = query.iter().zip(v.iter()).map(|(&a, &b)| a * b).sum();
                let norm_q: f32 = query.iter().map(|&x| x * x).sum::<f32>().sqrt();
                let norm_v: f32 = v.iter().map(|&x| x * x).sum::<f32>().sqrt();
                let cos_sim = dot / (norm_q * norm_v).max(1e-9);
                let cos_dist = 1.0 - cos_sim;
                (idx, cos_dist)
            })
            .collect();

        gt_distances.select_nth_unstable_by(top_k - 1, |a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        let gt_top_k: std::collections::HashSet<usize> =
            gt_distances[..top_k].iter().map(|&(idx, _)| idx).collect();

        // RaBitQ asymmetric distance estimation
        let mut rabitq_distances: Vec<(usize, f32)> = codes
            .iter()
            .enumerate()
            .map(|(idx, code)| {
                let dist = quantizer.asymmetric_distance(query, code)?;
                Ok((idx, dist))
            })
            .collect::<Result<Vec<_>, ContextraError>>()?;

        rabitq_distances.select_nth_unstable_by(top_k - 1, |a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        let rabitq_top_k: std::collections::HashSet<usize> = rabitq_distances[..top_k]
            .iter()
            .map(|&(idx, _)| idx)
            .collect();

        let hits = gt_top_k.intersection(&rabitq_top_k).count();
        total_hits += hits;
    }

    let measured_recall = (total_hits as f32) / ((num_queries * top_k) as f32);
    println!(
        "Measured RaBitQ recall@10 on 2000 vectors (dim 128, 50 queries): {measured_recall:.4}"
    );

    // Measured recall is 1.0000 on clustered synthetic benchmark.
    // Assertion with 0.05 safety margin below measured recall value (1.00 - 0.05 = 0.95).
    let target_threshold = 0.95;
    assert!(
        measured_recall >= target_threshold,
        "Measured recall {measured_recall:.4} must be >= threshold {target_threshold:.4}"
    );

    Ok(())
}
