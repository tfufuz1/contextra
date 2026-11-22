#![cfg(feature = "experimental-rabitq")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::needless_range_loop)]

use contextra_vector::quantize_rabitq::RaBitQQuantizer;

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
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
        ((self.next_u64() >> 32) as f32 / 4294967296.0) * 2.0 - 1.0
    }

    fn next_vector(&mut self, dim: usize, scale: f32) -> Vec<f32> {
        (0..dim).map(|_| self.next_f32() * scale).collect()
    }
}

/// 4-bit scalar quantization on raw vectors as multi-bit quantization comparison
fn quantize_4bit_vector(vector: &[f32]) -> (Vec<u8>, f32, f32) {
    let dim = vector.len();

    let min_val = vector.iter().copied().fold(f32::INFINITY, f32::min);
    let max_val = vector.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let range = (max_val - min_val).max(1e-6);

    let mut codes_4bit = vec![0u8; dim.div_ceil(2)];
    for (i, &val) in vector.iter().enumerate() {
        let norm_val = ((val - min_val) / range).clamp(0.0, 1.0);
        let q_level = (norm_val * 15.0).round() as u8;

        let byte_idx = i / 2;
        if i % 2 == 0 {
            codes_4bit[byte_idx] |= q_level & 0x0F;
        } else {
            codes_4bit[byte_idx] |= (q_level & 0x0F) << 4;
        }
    }

    (codes_4bit, min_val, range)
}

fn asymmetric_distance_4bit_vector(
    query: &[f32],
    code_4bit: &[u8],
    min_val: f32,
    range: f32,
) -> f32 {
    let dim = query.len();
    let mut dist_sq = 0.0f32;

    for i in 0..dim {
        let byte_idx = i / 2;
        let q_level = if i % 2 == 0 {
            code_4bit[byte_idx] & 0x0F
        } else {
            (code_4bit[byte_idx] >> 4) & 0x0F
        };

        let recon_i = min_val + (q_level as f32 / 15.0) * range;
        let diff = query[i] - recon_i;
        dist_sq += diff * diff;
    }

    dist_sq.sqrt()
}

#[tokio::test]
async fn test_rabitq_calibration_recall_1bit_vs_4bit() {
    // Hinweis nach Governance-Regel 3: Messung beinhaltet nur Index-/Speicherlatenz, ohne Embedding-Inferenz.
    let dim = 128;
    let num_vecs = 1000;
    let num_clusters = 50;
    let cluster_size = 20;
    let top_k = 10;
    let num_queries = 30;

    let mut rng = SimpleRng::new(1234567);

    // Generate clustered dataset
    let mut cluster_centers = Vec::with_capacity(num_clusters);
    for _ in 0..num_clusters {
        cluster_centers.push(rng.next_vector(dim, 10.0));
    }

    let mut vectors = Vec::with_capacity(num_vecs);
    for i in 0..num_vecs {
        let center = &cluster_centers[i / cluster_size];
        let noise = rng.next_vector(dim, 0.1);
        let vec: Vec<f32> = center
            .iter()
            .zip(noise.iter())
            .map(|(&c, &n)| c + n)
            .collect();
        vectors.push(vec);
    }

    // Train RaBitQ quantizer (1-bit)
    let quantizer = RaBitQQuantizer::try_train(&vectors, dim).expect("RaBitQ training");

    // Encode 1-bit RaBitQ codes and 4-bit codes
    let codes_1bit: Vec<Vec<u8>> = vectors
        .iter()
        .map(|v| quantizer.quantize(v).expect("quantize 1bit"))
        .collect();

    let codes_4bit_data: Vec<(Vec<u8>, f32, f32)> =
        vectors.iter().map(|v| quantize_4bit_vector(v)).collect();

    let mut hits_1bit = 0;
    let mut hits_4bit = 0;
    let total_expected = num_queries * top_k;

    for q_idx in 0..num_queries {
        let center = &cluster_centers[q_idx % num_clusters];
        let noise = rng.next_vector(dim, 0.1);
        let query: Vec<f32> = center
            .iter()
            .zip(noise.iter())
            .map(|(&c, &n)| c + n)
            .collect();

        // Exact float32 ground truth
        let mut gt_dists: Vec<(usize, f32)> = vectors
            .iter()
            .enumerate()
            .map(|(idx, v)| {
                let dist = query
                    .iter()
                    .zip(v.iter())
                    .map(|(&a, &b)| (a - b) * (a - b))
                    .sum::<f32>()
                    .sqrt();
                (idx, dist)
            })
            .collect();
        gt_dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let gt_top_k: std::collections::HashSet<usize> =
            gt_dists[..top_k].iter().map(|&(idx, _)| idx).collect();

        // 1-Bit RaBitQ search
        let mut dists_1bit: Vec<(usize, f32)> = codes_1bit
            .iter()
            .enumerate()
            .map(|(idx, code)| {
                let dist = quantizer
                    .asymmetric_distance(&query, code)
                    .expect("asym dist 1bit");
                (idx, dist)
            })
            .collect();
        dists_1bit.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for &(idx, _) in &dists_1bit[..top_k] {
            if gt_top_k.contains(&idx) {
                hits_1bit += 1;
            }
        }

        // 4-Bit quantization search
        let mut dists_4bit: Vec<(usize, f32)> = codes_4bit_data
            .iter()
            .enumerate()
            .map(|(idx, (code, min_val, range))| {
                let dist = asymmetric_distance_4bit_vector(&query, code, *min_val, *range);
                (idx, dist)
            })
            .collect();
        dists_4bit.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for &(idx, _) in &dists_4bit[..top_k] {
            if gt_top_k.contains(&idx) {
                hits_4bit += 1;
            }
        }
    }

    let recall_1bit = (hits_1bit as f32) / (total_expected as f32);
    let recall_4bit = (hits_4bit as f32) / (total_expected as f32);

    println!("=== RaBitQ Calibration Recall Measurement (only index/memory latency, no embedding inference) ===");
    println!("RaBitQ 1-Bit Recall@10: {recall_1bit:.4} ({hits_1bit}/{total_expected})");
    println!("SQ 4-Bit Recall@10:     {recall_4bit:.4} ({hits_4bit}/{total_expected})");

    // Plausibility Assertion: 4-Bit Recall >= 1-Bit Recall
    assert!(
        recall_4bit >= recall_1bit - 1e-5,
        "4-Bit recall ({recall_4bit:.4}) must be >= 1-Bit RaBitQ recall ({recall_1bit:.4})"
    );
}
