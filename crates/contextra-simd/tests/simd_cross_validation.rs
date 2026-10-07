#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_simd::dispatch::{
    cosine_distance, cosine_distance_f32_bytes, cosine_similarity_parts_u8, dot_product_distance,
    dot_product_distance_f32_bytes, dot_product_u8, euclidean_distance,
    euclidean_distance_f32_bytes, euclidean_distance_sq_u8,
};

use contextra_simd::kernels::scalar::{
    cosine_distance_f32_bytes_scalar, cosine_distance_scalar, cosine_similarity_parts_u8_scalar,
    dot_product_f32_bytes_scalar, dot_product_scalar, dot_product_u8_scalar,
    euclidean_distance_f32_bytes_scalar, euclidean_distance_scalar,
    euclidean_distance_sq_u8_scalar,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn simd_vs_scalar_cosine_f32(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = cosine_distance_scalar(a, b);
        let dispatch = cosine_distance(a, b).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-4 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Cosine mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    #[test]
    fn simd_vs_scalar_euclidean_f32(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = euclidean_distance_scalar(a, b);
        let dispatch = euclidean_distance(a, b).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Euclidean mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    #[test]
    fn simd_vs_scalar_dot_product_f32(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = dot_product_scalar(a, b);
        let dispatch = dot_product_distance(a, b).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Dot product mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    #[test]
    fn simd_vs_scalar_cosine_f32_bytes(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b_bytes: Vec<u8> = b[..n].iter().flat_map(|x| x.to_le_bytes()).collect();

        let scalar = cosine_distance_f32_bytes_scalar(a, &b_bytes);
        let dispatch = cosine_distance_f32_bytes(a, &b_bytes).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-4 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Cosine f32_bytes mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    #[test]
    fn simd_vs_scalar_euclidean_f32_bytes(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b_bytes: Vec<u8> = b[..n].iter().flat_map(|x| x.to_le_bytes()).collect();

        let scalar = euclidean_distance_f32_bytes_scalar(a, &b_bytes);
        let dispatch = euclidean_distance_f32_bytes(a, &b_bytes).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Euclidean f32_bytes mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    #[test]
    fn simd_vs_scalar_dot_product_f32_bytes(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b_bytes: Vec<u8> = b[..n].iter().flat_map(|x| x.to_le_bytes()).collect();

        let scalar = dot_product_f32_bytes_scalar(a, &b_bytes);
        let dispatch = dot_product_distance_f32_bytes(a, &b_bytes).unwrap();
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Dot product f32_bytes mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    #[test]
    fn simd_vs_scalar_dot_product_u8(
        dim in 1..1024usize,
        a in prop::collection::vec(any::<u8>(), 1..1024),
        b in prop::collection::vec(any::<u8>(), 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = dot_product_u8_scalar(a, b);
        let dispatch = dot_product_u8(a, b).unwrap();
        prop_assert_eq!(
            scalar, dispatch,
            "Dot product u8 mismatch at dim {}: scalar={}, dispatch={}",
            n, scalar, dispatch
        );
    }

    #[test]
    fn simd_vs_scalar_euclidean_sq_u8(
        dim in 1..1024usize,
        a in prop::collection::vec(any::<u8>(), 1..1024),
        b in prop::collection::vec(any::<u8>(), 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = euclidean_distance_sq_u8_scalar(a, b);
        let dispatch = euclidean_distance_sq_u8(a, b).unwrap();
        prop_assert_eq!(
            scalar, dispatch,
            "Euclidean sq u8 mismatch at dim {}: scalar={}, dispatch={}",
            n, scalar, dispatch
        );
    }

    #[test]
    fn simd_vs_scalar_cosine_similarity_parts_u8(
        dim in 1..1024usize,
        a in prop::collection::vec(any::<u8>(), 1..1024),
        b in prop::collection::vec(any::<u8>(), 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = cosine_similarity_parts_u8_scalar(a, b);
        let dispatch = cosine_similarity_parts_u8(a, b).unwrap();
        prop_assert_eq!(
            scalar, dispatch,
            "Cosine similarity parts u8 mismatch at dim {}: scalar={:?}, dispatch={:?}",
            n, scalar, dispatch
        );
    }
}

#[test]
fn test_s4_1000_pairs_equivalence_threshold() {
    let dimensions = [128, 512, 1536, 4096];
    let num_pairs = 1000;

    for &dim in &dimensions {
        let mut max_cos_diff = 0.0f32;
        let mut max_euc_diff = 0.0f32;
        let mut max_dot_diff = 0.0f32;

        for pair_idx in 0..num_pairs {
            let a: Vec<f32> = (0..dim)
                .map(|i| (((i * 17 + pair_idx * 31) % 1000) as f32) / 100.0 - 5.0)
                .collect();
            let b: Vec<f32> = (0..dim)
                .map(|i| (((i * 23 + pair_idx * 37) % 1000) as f32) / 100.0 - 5.0)
                .collect();

            let cos_s = cosine_distance_scalar(&a, &b);
            let cos_d = cosine_distance(&a, &b).unwrap();
            let cos_diff = (cos_s - cos_d).abs();
            max_cos_diff = max_cos_diff.max(cos_diff);

            let euc_s = euclidean_distance_scalar(&a, &b);
            let euc_d = euclidean_distance(&a, &b).unwrap();
            let euc_diff = (euc_s - euc_d).abs();
            max_euc_diff = max_euc_diff.max(euc_diff);

            let dot_s = dot_product_scalar(&a, &b);
            let dot_d = dot_product_distance(&a, &b).unwrap();
            let dot_diff = (dot_s - dot_d).abs();
            max_dot_diff = max_dot_diff.max(dot_diff);
        }

        println!("Dim {dim}: max_cos_diff={max_cos_diff:.8e}, max_euc_diff={max_euc_diff:.8e}, max_dot_diff={max_dot_diff:.8e}");

        assert!(
            max_cos_diff < 1e-5,
            "Cosine max diff at dim {dim} exceeded 1e-5: {max_cos_diff}"
        );
        assert!(
            max_euc_diff < 1e-3,
            "Euclidean max diff at dim {dim}: {max_euc_diff}"
        );
        assert!(
            max_dot_diff < 5e-3,
            "Dot product max diff at dim {dim}: {max_dot_diff}"
        );
    }
}

#[test]
fn test_s1_zero_vector_and_l2_norm() {
    let zero = vec![0.0f32; 128];
    let v = vec![1.0f32; 128];

    assert_eq!(cosine_distance_scalar(&zero, &zero), 1.0);
    assert_eq!(cosine_distance(&zero, &zero).unwrap(), 1.0);
    assert_eq!(cosine_distance(&zero, &v).unwrap(), 1.0);
    assert_eq!(cosine_distance(&v, &zero).unwrap(), 1.0);
}

#[test]
fn test_s2_l2_distance_correctness() {
    let a = vec![3.0f32, 0.0];
    let b = vec![0.0f32, 4.0];
    assert_eq!(euclidean_distance_scalar(&a, &b), 5.0);
    assert_eq!(euclidean_distance(&a, &b).unwrap(), 5.0);
}

#[test]
fn test_s3_overflow_behavior() {
    let norm_a = vec![1.0f32 / (4096.0f32.sqrt()); 4096];
    let norm_b = vec![1.0f32 / (4096.0f32.sqrt()); 4096];
    let dot = dot_product_distance(&norm_a, &norm_b).unwrap();
    assert!(!dot.is_nan() && !dot.is_infinite());

    let huge_a = vec![1e20f32; 128];
    let huge_b = vec![1e20f32; 128];
    let huge_dot = dot_product_distance(&huge_a, &huge_b).unwrap();
    assert!(huge_dot.is_infinite());
}
