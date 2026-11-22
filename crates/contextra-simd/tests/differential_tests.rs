// ADR-002 / SPEC §27: Test assertions and test dataset casts
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
// FILE-CONTEXT
// ZWECK: Deterministic differential test suite across vector dimensions 1..=257 and unaligned slice windows.

// ADR-002 / SPEC §27: Test assertions and test dataset casts

use contextra_simd::dispatch::{
    cosine_distance, cosine_distance_f32_bytes, cosine_similarity_parts_u8, dot_product_distance,
    dot_product_distance_f32_bytes, dot_product_u8, euclidean_distance,
    euclidean_distance_f32_bytes, euclidean_distance_sq_u8,
};
use contextra_simd::kernels::scalar::{
    cosine_distance_f32_bytes_scalar, cosine_distance_scalar, cosine_similarity_parts_f32_u8,
    cosine_similarity_parts_u8_scalar, dot_product_f32_bytes_scalar, dot_product_f32_u8,
    dot_product_scalar, dot_product_u8_scalar, euclidean_distance_f32_bytes_scalar,
    euclidean_distance_scalar, euclidean_distance_sq_f32_u8, euclidean_distance_sq_u8_scalar,
    normalize_inplace,
};

/// Simple deterministic linear congruential generator (LCG) for reproducible pseudo-random test inputs.
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn with_seed(seed: u64) -> Self {
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

    fn gen_f32_vec(&mut self, len: usize, min: f32, max: f32) -> Vec<f32> {
        (0..len).map(|_| self.next_f32(min, max)).collect()
    }

    fn gen_u8_vec(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next_u8()).collect()
    }
}

// Relative ULP tolerance explanation:
// SIMD kernels use fused multiply-add (FMA) instructions (`fmadd`), which keep infinite/higher intermediate precision
// before a single rounding, whereas scalar code performs separate `a * b` and `+ sum` roundings.
// The resulting precision difference is typically within 1e-4 relative tolerance for normal float values.

#[test]
fn test_sequential_dimensions_cosine_f32() {
    let mut rng = DeterministicRng::with_seed(0x1234_5678_9ABC_DEF0);

    for dim in 1..=257 {
        let a = rng.gen_f32_vec(dim, -5.0, 5.0);
        let b = rng.gen_f32_vec(dim, -5.0, 5.0);

        let scalar = cosine_distance_scalar(&a, &b);
        let dispatch = cosine_distance(&a, &b).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-4 * scalar.abs().max(dispatch.abs()).max(1.0);

        assert!(
            diff <= tol,
            "Cosine f32 mismatch at dim {dim}: scalar={scalar}, dispatch={dispatch}, diff={diff}, tol={tol}"
        );
    }
}

#[test]
fn test_sequential_dimensions_euclidean_f32() {
    let mut rng = DeterministicRng::with_seed(0x2345_6789_ABCD_EF01);

    for dim in 1..=257 {
        let a = rng.gen_f32_vec(dim, -10.0, 10.0);
        let b = rng.gen_f32_vec(dim, -10.0, 10.0);

        let scalar = euclidean_distance_scalar(&a, &b);
        let dispatch = euclidean_distance(&a, &b).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);

        assert!(
            diff <= tol,
            "Euclidean f32 mismatch at dim {dim}: scalar={scalar}, dispatch={dispatch}, diff={diff}, tol={tol}"
        );
    }
}

#[test]
fn test_sequential_dimensions_dot_product_f32() {
    let mut rng = DeterministicRng::with_seed(0x3456_789A_BCDE_F012);

    for dim in 1..=257 {
        let a = rng.gen_f32_vec(dim, -10.0, 10.0);
        let b = rng.gen_f32_vec(dim, -10.0, 10.0);

        let scalar = dot_product_scalar(&a, &b);
        let dispatch = dot_product_distance(&a, &b).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);

        assert!(
            diff <= tol,
            "Dot product f32 mismatch at dim {dim}: scalar={scalar}, dispatch={dispatch}, diff={diff}, tol={tol}"
        );
    }
}

#[test]
fn test_sequential_dimensions_cosine_f32_bytes() {
    let mut rng = DeterministicRng::with_seed(0x4567_89AB_CDEF_0123);

    for dim in 1..=257 {
        let a = rng.gen_f32_vec(dim, -5.0, 5.0);
        let b = rng.gen_f32_vec(dim, -5.0, 5.0);
        let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

        let scalar = cosine_distance_f32_bytes_scalar(&a, &b_bytes);
        let dispatch = cosine_distance_f32_bytes(&a, &b_bytes).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-4 * scalar.abs().max(dispatch.abs()).max(1.0);

        assert!(
            diff <= tol,
            "Cosine f32_bytes mismatch at dim {dim}: scalar={scalar}, dispatch={dispatch}, diff={diff}, tol={tol}"
        );
    }
}

#[test]
fn test_sequential_dimensions_euclidean_f32_bytes() {
    let mut rng = DeterministicRng::with_seed(0x5678_9ABC_DEF0_1234);

    for dim in 1..=257 {
        let a = rng.gen_f32_vec(dim, -10.0, 10.0);
        let b = rng.gen_f32_vec(dim, -10.0, 10.0);
        let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

        let scalar = euclidean_distance_f32_bytes_scalar(&a, &b_bytes);
        let dispatch = euclidean_distance_f32_bytes(&a, &b_bytes).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);

        assert!(
            diff <= tol,
            "Euclidean f32_bytes mismatch at dim {dim}: scalar={scalar}, dispatch={dispatch}, diff={diff}, tol={tol}"
        );
    }
}

#[test]
fn test_sequential_dimensions_dot_product_f32_bytes() {
    let mut rng = DeterministicRng::with_seed(0x6789_ABCD_EF01_2345);

    for dim in 1..=257 {
        let a = rng.gen_f32_vec(dim, -10.0, 10.0);
        let b = rng.gen_f32_vec(dim, -10.0, 10.0);
        let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

        let scalar = dot_product_f32_bytes_scalar(&a, &b_bytes);
        let dispatch = dot_product_distance_f32_bytes(&a, &b_bytes).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);

        assert!(
            diff <= tol,
            "Dot product f32_bytes mismatch at dim {dim}: scalar={scalar}, dispatch={dispatch}, diff={diff}, tol={tol}"
        );
    }
}

#[test]
fn test_sequential_dimensions_dot_product_u8() {
    let mut rng = DeterministicRng::with_seed(0x789A_BCDE_F012_3456);

    for dim in 1..=257 {
        let a = rng.gen_u8_vec(dim);
        let b = rng.gen_u8_vec(dim);

        let scalar = dot_product_u8_scalar(&a, &b);
        let dispatch = dot_product_u8(&a, &b).unwrap();

        assert_eq!(
            scalar, dispatch,
            "Dot product u8 mismatch at dim {dim}: scalar={scalar}, dispatch={dispatch}"
        );
    }
}

#[test]
fn test_sequential_dimensions_euclidean_sq_u8() {
    let mut rng = DeterministicRng::with_seed(0x89AB_CDEF_0123_4567);

    for dim in 1..=257 {
        let a = rng.gen_u8_vec(dim);
        let b = rng.gen_u8_vec(dim);

        let scalar = euclidean_distance_sq_u8_scalar(&a, &b);
        let dispatch = euclidean_distance_sq_u8(&a, &b).unwrap();

        assert_eq!(
            scalar, dispatch,
            "Euclidean sq u8 mismatch at dim {dim}: scalar={scalar}, dispatch={dispatch}"
        );
    }
}

#[test]
fn test_sequential_dimensions_cosine_similarity_parts_u8() {
    let mut rng = DeterministicRng::with_seed(0x9ABC_DEF0_1234_5678);

    for dim in 1..=257 {
        let a = rng.gen_u8_vec(dim);
        let b = rng.gen_u8_vec(dim);

        let scalar = cosine_similarity_parts_u8_scalar(&a, &b);
        let dispatch = cosine_similarity_parts_u8(&a, &b).unwrap();

        assert_eq!(
            scalar, dispatch,
            "Cosine similarity parts u8 mismatch at dim {dim}: scalar={scalar:?}, dispatch={dispatch:?}"
        );
    }
}

#[test]
fn test_sequential_dimensions_dot_product_f32_u8() {
    let mut rng = DeterministicRng::with_seed(0xABCD_EF01_2345_6789);

    for dim in 1..=257 {
        let a = rng.gen_f32_vec(dim, -5.0, 5.0);
        let b = rng.gen_u8_vec(dim);

        let scalar = dot_product_f32_u8(&a, &b);
        assert!(!scalar.is_nan());
    }
}

#[test]
fn test_sequential_dimensions_euclidean_sq_f32_u8() {
    let mut rng = DeterministicRng::with_seed(0xBCDE_F012_3456_789A);

    for dim in 1..=257 {
        let a = rng.gen_f32_vec(dim, -5.0, 5.0);
        let b = rng.gen_u8_vec(dim);
        let alphas = rng.gen_f32_vec(dim, 0.01, 0.1);
        let mins = rng.gen_f32_vec(dim, -1.0, 1.0);

        let res = euclidean_distance_sq_f32_u8(&a, &b, &alphas, &mins);
        assert!(res >= 0.0);
    }
}

#[test]
fn test_sequential_dimensions_cosine_similarity_parts_f32_u8() {
    let mut rng = DeterministicRng::with_seed(0xCDEF_0123_4567_89AB);

    for dim in 1..=257 {
        let a = rng.gen_f32_vec(dim, -5.0, 5.0);
        let b = rng.gen_u8_vec(dim);

        let parts = cosine_similarity_parts_f32_u8(&a, &b);
        assert!(parts.norm_a_sq >= 0.0);
        assert!(parts.norm_b_sq >= 0.0);
    }
}

#[test]
fn test_sequential_dimensions_normalize_inplace() {
    let mut rng = DeterministicRng::with_seed(0x1122_3344_5566_7788);

    for dim in 1..=257 {
        let mut v = rng.gen_f32_vec(dim, -10.0, 10.0);
        normalize_inplace(&mut v);

        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "Normalized vector L2 norm at dim {dim} should be ~1.0, got {norm}"
        );
    }
}

#[test]
fn test_multiple_prng_seeds_f32_metrics() {
    let seeds = [
        0x1111_2222_3333_4444,
        0x5555_6666_7777_8888,
        0x9999_AAAA_BBBB_CCCC,
        0xDDDD_EEEE_FFFF_0000,
    ];

    for &seed in &seeds {
        let mut rng = DeterministicRng::with_seed(seed);
        for dim in [1, 7, 15, 31, 63, 127, 255] {
            let a = rng.gen_f32_vec(dim, -10.0, 10.0);
            let b = rng.gen_f32_vec(dim, -10.0, 10.0);

            let cos_s = cosine_distance_scalar(&a, &b);
            let cos_d = cosine_distance(&a, &b).unwrap();
            assert!((cos_s - cos_d).abs() <= 1e-4 * cos_s.abs().max(cos_d.abs()).max(1.0));

            let euc_s = euclidean_distance_scalar(&a, &b);
            let euc_d = euclidean_distance(&a, &b).unwrap();
            assert!((euc_s - euc_d).abs() <= 1e-3 * euc_s.abs().max(euc_d.abs()).max(1.0));

            let dot_s = dot_product_scalar(&a, &b);
            let dot_d = dot_product_distance(&a, &b).unwrap();
            assert!((dot_s - dot_d).abs() <= 1e-3 * dot_s.abs().max(dot_d.abs()).max(1.0));
        }
    }
}

#[test]
fn test_multiple_prng_seeds_u8_metrics() {
    let seeds = [
        0xA1B2_C3D4_E5F6_0102,
        0x0304_0506_0708_090A,
        0x0B0C_0D0E_0F10_1112,
    ];

    for &seed in &seeds {
        let mut rng = DeterministicRng::with_seed(seed);
        for dim in [1, 8, 16, 32, 64, 128, 256] {
            let a = rng.gen_u8_vec(dim);
            let b = rng.gen_u8_vec(dim);

            assert_eq!(
                dot_product_u8_scalar(&a, &b),
                dot_product_u8(&a, &b).unwrap()
            );
            assert_eq!(
                euclidean_distance_sq_u8_scalar(&a, &b),
                euclidean_distance_sq_u8(&a, &b).unwrap()
            );
            assert_eq!(
                cosine_similarity_parts_u8_scalar(&a, &b),
                cosine_similarity_parts_u8(&a, &b).unwrap()
            );
        }
    }
}

#[test]
fn test_unaligned_slice_windows_f32() {
    let mut rng = DeterministicRng::with_seed(0xDEF0_1234_5678_9ABC);

    // Generate large buffer and test non-byte-aligned slice windows
    let buf_a = rng.gen_f32_vec(300, -5.0, 5.0);
    let buf_b = rng.gen_f32_vec(300, -5.0, 5.0);

    for offset in 1..=7 {
        for dim in [16, 32, 64, 128, 256] {
            let a = &buf_a[offset..offset + dim];
            let b = &buf_b[offset + 1..offset + 1 + dim];

            let cos_s = cosine_distance_scalar(a, b);
            let cos_d = cosine_distance(a, b).unwrap();
            let cos_tol = 1e-4 * cos_s.abs().max(cos_d.abs()).max(1.0);
            assert!(
                (cos_s - cos_d).abs() <= cos_tol,
                "Unaligned Cosine mismatch at offset {offset}, dim {dim}"
            );

            let euc_s = euclidean_distance_scalar(a, b);
            let euc_d = euclidean_distance(a, b).unwrap();
            let euc_tol = 1e-3 * euc_s.abs().max(euc_d.abs()).max(1.0);
            assert!(
                (euc_s - euc_d).abs() <= euc_tol,
                "Unaligned Euclidean mismatch at offset {offset}, dim {dim}"
            );

            let dot_s = dot_product_scalar(a, b);
            let dot_d = dot_product_distance(a, b).unwrap();
            let dot_tol = 1e-3 * dot_s.abs().max(dot_d.abs()).max(1.0);
            assert!(
                (dot_s - dot_d).abs() <= dot_tol,
                "Unaligned Dot Product mismatch at offset {offset}, dim {dim}"
            );
        }
    }
}

#[test]
fn test_unaligned_slice_windows_u8() {
    let mut rng = DeterministicRng::with_seed(0xEF01_2345_6789_ABCD);

    let buf_a = rng.gen_u8_vec(300);
    let buf_b = rng.gen_u8_vec(300);

    for offset in 1..=7 {
        for dim in [16, 32, 64, 128, 256] {
            let a = &buf_a[offset..offset + dim];
            let b = &buf_b[offset + 2..offset + 2 + dim];

            let dot_s = dot_product_u8_scalar(a, b);
            let dot_d = dot_product_u8(a, b).unwrap();
            assert_eq!(
                dot_s, dot_d,
                "Unaligned u8 dot product mismatch at offset {offset}, dim {dim}"
            );

            let euc_s = euclidean_distance_sq_u8_scalar(a, b);
            let euc_d = euclidean_distance_sq_u8(a, b).unwrap();
            assert_eq!(
                euc_s, euc_d,
                "Unaligned u8 euclidean sq mismatch at offset {offset}, dim {dim}"
            );

            let cos_s = cosine_similarity_parts_u8_scalar(a, b);
            let cos_d = cosine_similarity_parts_u8(a, b).unwrap();
            assert_eq!(
                cos_s, cos_d,
                "Unaligned u8 cosine parts mismatch at offset {offset}, dim {dim}"
            );
        }
    }
}

#[test]
fn test_unaligned_f32_bytes_slice_windows() {
    let mut rng = DeterministicRng::with_seed(0x0123_4567_89AB_CDEF);

    let buf_a = rng.gen_f32_vec(300, -5.0, 5.0);
    let buf_b = rng.gen_f32_vec(300, -5.0, 5.0);
    let buf_b_bytes: Vec<u8> = buf_b.iter().flat_map(|x| x.to_le_bytes()).collect();

    for offset in 1..=7 {
        for dim in [16, 32, 64, 128] {
            let a = &buf_a[offset..offset + dim];
            let b_bytes = &buf_b_bytes[offset * 4..(offset + dim) * 4];

            let cos_s = cosine_distance_f32_bytes_scalar(a, b_bytes);
            let cos_d = cosine_distance_f32_bytes(a, b_bytes).unwrap();
            assert!((cos_s - cos_d).abs() <= 1e-4 * cos_s.abs().max(cos_d.abs()).max(1.0));

            let euc_s = euclidean_distance_f32_bytes_scalar(a, b_bytes);
            let euc_d = euclidean_distance_f32_bytes(a, b_bytes).unwrap();
            assert!((euc_s - euc_d).abs() <= 1e-3 * euc_s.abs().max(euc_d.abs()).max(1.0));

            let dot_s = dot_product_f32_bytes_scalar(a, b_bytes);
            let dot_d = dot_product_distance_f32_bytes(a, b_bytes).unwrap();
            assert!((dot_s - dot_d).abs() <= 1e-3 * dot_s.abs().max(dot_d.abs()).max(1.0));
        }
    }
}
