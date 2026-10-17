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

#[test]
fn test_unaligned_and_zero_length_differential_all_metrics() {
    let unaligned_lengths = [0, 1, 2, 3, 5, 7, 9, 13, 15, 17, 31, 33, 63, 65, 127, 255];

    for &len in &unaligned_lengths {
        let a: Vec<f32> = (0..len).map(|i| (i as f32 * 0.37).sin() * 5.0).collect();
        let b: Vec<f32> = (0..len).map(|i| (i as f32 * 0.53).cos() * 5.0).collect();
        let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

        let a_u8: Vec<u8> = (0..len).map(|i| ((i * 37) % 256) as u8).collect();
        let b_u8: Vec<u8> = (0..len).map(|i| ((i * 53) % 256) as u8).collect();

        // 1. Cosine f32
        let cos_s = cosine_distance_scalar(&a, &b);
        let cos_d = cosine_distance(&a, &b).unwrap();
        let cos_diff = (cos_s - cos_d).abs();
        let cos_tol = 1e-4 * cos_s.abs().max(cos_d.abs()).max(1.0);
        assert!(
            cos_diff <= cos_tol,
            "Cosine f32 mismatch at len {len}: scalar={cos_s}, dispatch={cos_d}, diff={cos_diff}, tol={cos_tol}"
        );

        // 2. Euclidean f32
        let euc_s = euclidean_distance_scalar(&a, &b);
        let euc_d = euclidean_distance(&a, &b).unwrap();
        let euc_diff = (euc_s - euc_d).abs();
        let euc_tol = 1e-3 * euc_s.abs().max(euc_d.abs()).max(1.0);
        assert!(
            euc_diff <= euc_tol,
            "Euclidean f32 mismatch at len {len}: scalar={euc_s}, dispatch={euc_d}, diff={euc_diff}, tol={euc_tol}"
        );

        // 3. Dot Product f32
        let dot_s = dot_product_scalar(&a, &b);
        let dot_d = dot_product_distance(&a, &b).unwrap();
        let dot_diff = (dot_s - dot_d).abs();
        let dot_tol = 1e-3 * dot_s.abs().max(dot_d.abs()).max(1.0);
        assert!(
            dot_diff <= dot_tol,
            "Dot product f32 mismatch at len {len}: scalar={dot_s}, dispatch={dot_d}, diff={dot_diff}, tol={dot_tol}"
        );

        // 4. Cosine f32 bytes
        let cos_bytes_s = cosine_distance_f32_bytes_scalar(&a, &b_bytes);
        let cos_bytes_d = cosine_distance_f32_bytes(&a, &b_bytes).unwrap();
        let cos_bytes_diff = (cos_bytes_s - cos_bytes_d).abs();
        let cos_bytes_tol = 1e-4 * cos_bytes_s.abs().max(cos_bytes_d.abs()).max(1.0);
        assert!(
            cos_bytes_diff <= cos_bytes_tol,
            "Cosine f32_bytes mismatch at len {len}: scalar={cos_bytes_s}, dispatch={cos_bytes_d}, diff={cos_bytes_diff}, tol={cos_bytes_tol}"
        );

        // 5. Euclidean f32 bytes
        let euc_bytes_s = euclidean_distance_f32_bytes_scalar(&a, &b_bytes);
        let euc_bytes_d = euclidean_distance_f32_bytes(&a, &b_bytes).unwrap();
        let euc_bytes_diff = (euc_bytes_s - euc_bytes_d).abs();
        let euc_bytes_tol = 1e-3 * euc_bytes_s.abs().max(euc_bytes_d.abs()).max(1.0);
        assert!(
            euc_bytes_diff <= euc_bytes_tol,
            "Euclidean f32_bytes mismatch at len {len}: scalar={euc_bytes_s}, dispatch={euc_bytes_d}, diff={euc_bytes_diff}, tol={euc_bytes_tol}"
        );

        // 6. Dot Product f32 bytes
        let dot_bytes_s = dot_product_f32_bytes_scalar(&a, &b_bytes);
        let dot_bytes_d = dot_product_distance_f32_bytes(&a, &b_bytes).unwrap();
        let dot_bytes_diff = (dot_bytes_s - dot_bytes_d).abs();
        let dot_bytes_tol = 1e-3 * dot_bytes_s.abs().max(dot_bytes_d.abs()).max(1.0);
        assert!(
            dot_bytes_diff <= dot_bytes_tol,
            "Dot product f32_bytes mismatch at len {len}: scalar={dot_bytes_s}, dispatch={dot_bytes_d}, diff={dot_bytes_diff}, tol={dot_bytes_tol}"
        );

        // 7. Dot Product u8
        let dot_u8_s = dot_product_u8_scalar(&a_u8, &b_u8);
        let dot_u8_d = dot_product_u8(&a_u8, &b_u8).unwrap();
        assert_eq!(
            dot_u8_s, dot_u8_d,
            "Dot product u8 mismatch at len {len}: scalar={dot_u8_s}, dispatch={dot_u8_d}"
        );

        // 8. Euclidean sq u8
        let euc_u8_s = euclidean_distance_sq_u8_scalar(&a_u8, &b_u8);
        let euc_u8_d = euclidean_distance_sq_u8(&a_u8, &b_u8).unwrap();
        assert_eq!(
            euc_u8_s, euc_u8_d,
            "Euclidean sq u8 mismatch at len {len}: scalar={euc_u8_s}, dispatch={euc_u8_d}"
        );

        // 9. Cosine similarity parts u8
        let cos_parts_s = cosine_similarity_parts_u8_scalar(&a_u8, &b_u8);
        let cos_parts_d = cosine_similarity_parts_u8(&a_u8, &b_u8).unwrap();
        assert_eq!(
            cos_parts_s, cos_parts_d,
            "Cosine similarity parts u8 mismatch at len {len}: scalar={cos_parts_s:?}, dispatch={cos_parts_d:?}"
        );
    }
}
