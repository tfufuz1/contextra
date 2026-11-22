// ADR-002 / SPEC §27: Test assertions and test dataset casts
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
// FILE-CONTEXT
// ZWECK: Quantized SQ8 and mixed-precision (f32 <-> u8) distance kernel test suite across dimensions 1..=257.

// ADR-002 / SPEC §27: Test assertions and test dataset casts

use contextra_simd::dispatch::{
    cosine_similarity_parts_u8, dot_product_u8, euclidean_distance_sq_u8,
};
use contextra_simd::kernels::scalar::{
    cosine_similarity_parts_f32_u8, cosine_similarity_parts_u8_scalar, dot_product_f32_u8,
    dot_product_u8_scalar, euclidean_distance_sq_f32_u8, euclidean_distance_sq_u8_scalar,
    normalize_inplace,
};

/// Deterministic LCG pseudo-random generator for quantization tests.
struct QuantTestRng {
    state: u64,
}

impl QuantTestRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_f32(&mut self, min: f32, max: f32) -> f32 {
        let val = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        (min as f64 + val * (max as f64 - min as f64)) as f32
    }

    fn next_u8(&mut self) -> u8 {
        (self.next_u64() & 0xFF) as u8
    }

    fn gen_f32(&mut self, len: usize) -> Vec<f32> {
        (0..len).map(|_| self.next_f32(-10.0, 10.0)).collect()
    }

    fn gen_u8(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next_u8()).collect()
    }
}

/// Simple SQ8 scalar quantizer converting float vectors to 8-bit unsigned integer vectors.
fn quantize_sq8(v: &[f32]) -> (Vec<u8>, f32, f32) {
    if v.is_empty() {
        return (vec![], 0.0, 0.0);
    }
    let mut min_val = f32::MAX;
    let mut max_val = f32::MIN;
    for &x in v {
        if x < min_val {
            min_val = x;
        }
        if x > max_val {
            max_val = x;
        }
    }
    if (max_val - min_val).abs() < 1e-6 {
        return (vec![0u8; v.len()], 0.0, min_val);
    }
    let alpha = (max_val - min_val) / 255.0;
    let inv_alpha = 1.0 / alpha;
    let q = v
        .iter()
        .map(|&x| ((x - min_val) * inv_alpha).round().clamp(0.0, 255.0) as u8)
        .collect();
    (q, alpha, min_val)
}

#[test]
fn test_sq8_quantization_reconstruction_accuracy() {
    let mut rng = QuantTestRng::new(0x3141_5926_5358_9793);

    for dim in [1, 16, 32, 64, 128, 256] {
        let original = rng.gen_f32(dim);
        let (q_u8, alpha, min_val) = quantize_sq8(&original);

        for (i, &q) in q_u8.iter().enumerate() {
            let reconstructed = (q as f32) * alpha + min_val;
            let diff = (original[i] - reconstructed).abs();
            assert!(
                diff <= alpha.max(1e-6),
                "SQ8 quantization reconstruction error at index {i}, dim {dim} exceeded step size {alpha}: {diff}"
            );
        }
    }
}

#[test]
fn test_dequantized_euclidean_distance_f32_u8() {
    let mut rng = QuantTestRng::new(0x2718_2818_2845_9045);

    for dim in 1..=257 {
        let a = rng.gen_f32(dim);
        let raw_b = rng.gen_f32(dim);
        let (b_u8, alpha, min_val) = quantize_sq8(&raw_b);

        let alphas = vec![alpha; dim];
        let mins = vec![min_val; dim];

        let euc_sq = euclidean_distance_sq_f32_u8(&a, &b_u8, &alphas, &mins);
        assert!(euc_sq >= 0.0);

        // Calculate expected distance against dequantized b
        let dequantized_b: Vec<f32> = b_u8.iter().map(|&q| (q as f32) * alpha + min_val).collect();
        let mut expected_sq = 0.0f32;
        for i in 0..dim {
            let diff = a[i] - dequantized_b[i];
            expected_sq += diff * diff;
        }

        assert!(
            (euc_sq - expected_sq).abs() < 1e-4 * euc_sq.abs().max(1.0),
            "Dequantized Euclidean sq mismatch at dim {dim}: got {euc_sq}, expected {expected_sq}"
        );
    }
}

#[test]
fn test_mixed_precision_dot_product_f32_u8_differential() {
    let mut rng = QuantTestRng::new(0x1618_0339_8874_9894);

    for dim in 1..=257 {
        let a = rng.gen_f32(dim);
        let b_u8 = rng.gen_u8(dim);

        let scalar_dot = dot_product_f32_u8(&a, &b_u8);

        let mut expected_dot = 0.0f32;
        for i in 0..dim {
            expected_dot += a[i] * (b_u8[i] as f32);
        }

        assert!(
            (scalar_dot - expected_dot).abs() < 1e-4 * scalar_dot.abs().max(1.0),
            "Mixed dot product mismatch at dim {dim}: got {scalar_dot}, expected {expected_dot}"
        );
    }
}

#[test]
fn test_mixed_precision_cosine_similarity_parts_f32_u8() {
    let mut rng = QuantTestRng::new(0x0577_2156_6490_1532);

    for dim in 1..=257 {
        let a = rng.gen_f32(dim);
        let b_u8 = rng.gen_u8(dim);

        let parts = cosine_similarity_parts_f32_u8(&a, &b_u8);

        let mut exp_dot = 0.0f32;
        let mut exp_norm_a_sq = 0.0f32;
        let mut exp_norm_b_sq = 0.0f32;

        for i in 0..dim {
            let x = a[i];
            let y = b_u8[i] as f32;
            exp_dot += x * y;
            exp_norm_a_sq += x * x;
            exp_norm_b_sq += y * y;
        }

        assert!((parts.dot - exp_dot).abs() < 1e-4 * exp_dot.abs().max(1.0));
        assert!((parts.norm_a_sq - exp_norm_a_sq).abs() < 1e-4 * exp_norm_a_sq.abs().max(1.0));
        assert!((parts.norm_b_sq - exp_norm_b_sq).abs() < 1e-4 * exp_norm_b_sq.abs().max(1.0));
    }
}

#[test]
fn test_u8_metrics_all_zero_inputs() {
    let zero = vec![0u8; 128];
    assert_eq!(dot_product_u8(&zero, &zero).unwrap(), 0);
    assert_eq!(dot_product_u8_scalar(&zero, &zero), 0);

    assert_eq!(euclidean_distance_sq_u8(&zero, &zero).unwrap(), 0);
    assert_eq!(euclidean_distance_sq_u8_scalar(&zero, &zero), 0);

    let parts = cosine_similarity_parts_u8(&zero, &zero).unwrap();
    let parts_s = cosine_similarity_parts_u8_scalar(&zero, &zero);
    assert_eq!(parts, parts_s);
    assert_eq!(parts.dot, 0);
    assert_eq!(parts.norm_a_sq, 0);
    assert_eq!(parts.norm_b_sq, 0);
}

#[test]
fn test_u8_metrics_all_ones_inputs() {
    let ones = vec![1u8; 128];
    assert_eq!(dot_product_u8(&ones, &ones).unwrap(), 128);
    assert_eq!(euclidean_distance_sq_u8(&ones, &ones).unwrap(), 0);

    let parts = cosine_similarity_parts_u8(&ones, &ones).unwrap();
    assert_eq!(parts.dot, 128);
    assert_eq!(parts.norm_a_sq, 128);
    assert_eq!(parts.norm_b_sq, 128);
}

#[test]
fn test_normalized_vector_self_cosine_distance_is_zero() {
    let mut rng = QuantTestRng::new(0x1234_4321_5678_8765);

    for dim in [8, 16, 32, 64, 128, 256] {
        let mut v = rng.gen_f32(dim);
        normalize_inplace(&mut v);

        let dist = contextra_simd::dispatch::cosine_distance(&v, &v).unwrap();
        assert!(
            dist.abs() < 1e-6,
            "Self cosine distance of normalized vector at dim {dim} must be 0.0, got {dist}"
        );
    }
}

#[test]
fn test_euclidean_distance_sq_f32_u8_per_dimension_alphas_and_mins() {
    let mut rng = QuantTestRng::new(0x9876_5432_10FE_DCBA);

    for dim in [4, 8, 16, 32, 64, 128, 256] {
        let a = rng.gen_f32(dim);
        let b = rng.gen_u8(dim);
        let alphas = rng.gen_f32(dim);
        let mins = rng.gen_f32(dim);

        let dist_sq = euclidean_distance_sq_f32_u8(&a, &b, &alphas, &mins);
        assert!(dist_sq >= 0.0);

        let mut expected = 0.0f32;
        for i in 0..dim {
            let y = (b[i] as f32) * alphas[i] + mins[i];
            let diff = a[i] - y;
            expected += diff * diff;
        }

        assert!((dist_sq - expected).abs() < 1e-3 * expected.abs().max(1.0));
    }
}

#[test]
fn test_cosine_similarity_parts_u8_varying_vector_magnitudes() {
    let mut rng = QuantTestRng::new(0x8765_4321_0FED_CBA9);

    for dim in [16, 32, 64, 128, 256] {
        let a = rng.gen_u8(dim);
        let b = rng.gen_u8(dim);

        let parts_dispatch = cosine_similarity_parts_u8(&a, &b).unwrap();
        let parts_scalar = cosine_similarity_parts_u8_scalar(&a, &b);

        assert_eq!(parts_dispatch.dot, parts_scalar.dot);
        assert_eq!(parts_dispatch.norm_a_sq, parts_scalar.norm_a_sq);
        assert_eq!(parts_dispatch.norm_b_sq, parts_scalar.norm_b_sq);
    }
}

#[test]
fn test_u8_dot_product_large_range_values() {
    let mut rng = QuantTestRng::new(0x7654_3210_FEDC_BA98);

    for dim in [10, 50, 100, 500, 1000] {
        let a = rng.gen_u8(dim);
        let b = rng.gen_u8(dim);

        let dot_d = dot_product_u8(&a, &b).unwrap();
        let dot_s = dot_product_u8_scalar(&a, &b);

        assert_eq!(dot_d, dot_s);
    }
}
