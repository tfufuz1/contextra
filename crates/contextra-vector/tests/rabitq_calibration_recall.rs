// FILE-CONTEXT
// ZWECK: Recall- und Kalibrierungsmessung von RaBitQ (1-Bit) im Vergleich zu 4-Bit Scalar Quantization.
// INVARIANTEN: 4-Bit Recall >= 1-Bit RaBitQ Recall - 1e-5.
// DATENSATZ: 1000 Vektoren, Dim: 64, 50 Queries, TestRng Seed: 42.
// HOTSPOTS: 35-125
// STAND: 2026-11-12

#![cfg(feature = "experimental-rabitq")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::needless_range_loop)]

#[path = "common_q/mod.rs"]
mod common_q;
use common_q::TestRng;
use contextra_vector::quantize_rabitq::RaBitQQuantizer;

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
            codes_4bit[byte_idx] |= q_level;
        } else {
            codes_4bit[byte_idx] |= q_level << 4;
        }
    }

    (codes_4bit, min_val, range)
}

fn distance_4bit_vector(query: &[f32], code: &[u8], min_val: f32, range: f32) -> f32 {
    let mut dist_sq = 0.0f32;
    for (i, &q) in query.iter().enumerate() {
        let byte_idx = i / 2;
        let q_level = if i % 2 == 0 {
            code[byte_idx] & 0x0F
        } else {
            (code[byte_idx] >> 4) & 0x0F
        };
        let reconstructed = min_val + (q_level as f32 / 15.0) * range;
        let diff = q - reconstructed;
        dist_sq += diff * diff;
    }
    dist_sq
}

#[tokio::test]
async fn test_rabitq_calibration_recall_1bit_vs_4bit() {
    let dim = 64;
    let num_vectors = 1000;
    let num_queries = 50;
    let top_k = 10;
    let seed = 42u64;

    let mut rng = TestRng::new(seed);

    let mut vectors = Vec::with_capacity(num_vectors);
    for _ in 0..num_vectors {
        vectors.push(rng.next_vector(dim, 10.0));
    }

    let quantizer = RaBitQQuantizer::try_train_with_seed(&vectors, dim, seed)
        .expect("RaBitQ quantizer training must succeed");

    let rabitq_codes: Vec<_> = vectors
        .iter()
        .map(|v| quantizer.quantize(v).expect("quantization failed"))
        .collect();

    let sq4_data: Vec<_> = vectors.iter().map(|v| quantize_4bit_vector(v)).collect();

    let mut queries = Vec::with_capacity(num_queries);
    for _ in 0..num_queries {
        queries.push(rng.next_vector(dim, 10.0));
    }

    let mut hits_1bit = 0;
    let mut hits_4bit = 0;
    let total_expected = num_queries * top_k;

    for query in queries {
        let mut gt_distances: Vec<(usize, f32)> = vectors
            .iter()
            .enumerate()
            .map(|(idx, v)| {
                let dist: f32 = query
                    .iter()
                    .zip(v.iter())
                    .map(|(&a, &b)| (a - b) * (a - b))
                    .sum();
                (idx, dist)
            })
            .collect();
        gt_distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let gt_top_k: std::collections::HashSet<usize> =
            gt_distances[..top_k].iter().map(|&(idx, _)| idx).collect();

        let mut dists_1bit: Vec<(usize, f32)> = rabitq_codes
            .iter()
            .enumerate()
            .map(|(idx, code)| {
                let dist = quantizer
                    .asymmetric_distance(&query, code)
                    .expect("asymmetric distance failed");
                (idx, dist)
            })
            .collect();
        dists_1bit.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for &(idx, _) in &dists_1bit[..top_k] {
            if gt_top_k.contains(&idx) {
                hits_1bit += 1;
            }
        }

        let mut dists_4bit: Vec<(usize, f32)> = sq4_data
            .iter()
            .enumerate()
            .map(|(idx, (code, min_val, range))| {
                let dist = distance_4bit_vector(&query, code, *min_val, *range);
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

    println!("=== RaBitQ Calibration Recall Measurement ===");
    println!("RaBitQ 1-Bit Recall@10: {recall_1bit:.4} ({hits_1bit}/{total_expected})");
    println!("SQ 4-Bit Recall@10:     {recall_4bit:.4} ({hits_4bit}/{total_expected})");

    assert!(
        recall_4bit >= recall_1bit - 1e-5,
        "4-Bit recall ({recall_4bit:.4}) must be >= 1-Bit RaBitQ recall ({recall_1bit:.4})"
    );
}
