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
use contextra_simd::{compute_distance, validate_vector};
use contextra_types::DistanceMetric;
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
fn test_unaligned_slice_offsets_all_metrics() {
    let base_a: Vec<f32> = (0..1024).map(|i| (i as f32 * 0.1).sin() * 2.0).collect();
    let base_b: Vec<f32> = (0..1024).map(|i| (i as f32 * 0.15).cos() * 2.0).collect();
    let base_a_u8: Vec<u8> = (0..1024).map(|i| (i * 13 % 256) as u8).collect();
    let base_b_u8: Vec<u8> = (0..1024).map(|i| (i * 17 % 256) as u8).collect();

    for offset_a in 0..4 {
        for offset_b in 0..4 {
            let len = 256;
            let slice_a = &base_a[offset_a..offset_a + len];
            let slice_b = &base_b[offset_b..offset_b + len];
            let b_bytes: Vec<u8> = slice_b.iter().flat_map(|x| x.to_le_bytes()).collect();

            let slice_a_u8 = &base_a_u8[offset_a..offset_a + len];
            let slice_b_u8 = &base_b_u8[offset_b..offset_b + len];

            // f32 cosine
            let cos_s = cosine_distance_scalar(slice_a, slice_b);
            let cos_d = cosine_distance(slice_a, slice_b).unwrap();
            assert!((cos_s - cos_d).abs() <= 1e-4);

            // f32 euclidean
            let euc_s = euclidean_distance_scalar(slice_a, slice_b);
            let euc_d = euclidean_distance(slice_a, slice_b).unwrap();
            assert!((euc_s - euc_d).abs() <= 1e-3);

            // f32 dot
            let dot_s = dot_product_scalar(slice_a, slice_b);
            let dot_d = dot_product_distance(slice_a, slice_b).unwrap();
            assert!((dot_s - dot_d).abs() <= 1e-3);

            // f32 bytes
            let cos_b_s = cosine_distance_f32_bytes_scalar(slice_a, &b_bytes);
            let cos_b_d = cosine_distance_f32_bytes(slice_a, &b_bytes).unwrap();
            assert!((cos_b_s - cos_b_d).abs() <= 1e-4);

            // u8 dot
            let dot_u8_s = dot_product_u8_scalar(slice_a_u8, slice_b_u8);
            let dot_u8_d = dot_product_u8(slice_a_u8, slice_b_u8).unwrap();
            assert_eq!(dot_u8_s, dot_u8_d);

            // u8 euclidean sq
            let euc_u8_s = euclidean_distance_sq_u8_scalar(slice_a_u8, slice_b_u8);
            let euc_u8_d = euclidean_distance_sq_u8(slice_a_u8, slice_b_u8).unwrap();
            assert_eq!(euc_u8_s, euc_u8_d);

            // u8 cosine parts
            let parts_s = cosine_similarity_parts_u8_scalar(slice_a_u8, slice_b_u8);
            let parts_d = cosine_similarity_parts_u8(slice_a_u8, slice_b_u8).unwrap();
            assert_eq!(parts_s, parts_d);
        }
    }
}

#[test]
fn test_dimension_mismatch_returns_error() {
    let a = vec![1.0f32; 10];
    let b = vec![1.0f32; 12];
    let a_u8 = vec![1u8; 10];
    let b_u8 = vec![1u8; 12];
    let b_bytes = vec![0u8; 32]; // 32 bytes = 8 floats, expected 10 floats = 40 bytes

    assert!(cosine_distance(&a, &b).is_err());
    assert!(euclidean_distance(&a, &b).is_err());
    assert!(dot_product_distance(&a, &b).is_err());
    assert!(cosine_distance_f32_bytes(&a, &b_bytes).is_err());
    assert!(euclidean_distance_f32_bytes(&a, &b_bytes).is_err());
    assert!(dot_product_distance_f32_bytes(&a, &b_bytes).is_err());
    assert!(dot_product_u8(&a_u8, &b_u8).is_err());
    assert!(euclidean_distance_sq_u8(&a_u8, &b_u8).is_err());
    assert!(cosine_similarity_parts_u8(&a_u8, &b_u8).is_err());
}

#[test]
fn test_special_float_values_validation_and_trusted() {
    let nan_vec = vec![1.0f32, f32::NAN, 3.0];
    let inf_vec = vec![1.0f32, f32::INFINITY, 3.0];
    let neg_inf_vec = vec![1.0f32, f32::NEG_INFINITY, 3.0];
    let valid_vec = vec![1.0f32, 2.0, 3.0];

    assert!(validate_vector(&nan_vec).is_err());
    assert!(validate_vector(&inf_vec).is_err());
    assert!(validate_vector(&neg_inf_vec).is_err());
    assert!(validate_vector(&valid_vec).is_ok());

    assert!(compute_distance(&nan_vec, &valid_vec, DistanceMetric::Cosine).is_err());
    assert!(compute_distance(&valid_vec, &nan_vec, DistanceMetric::Euclidean).is_err());

    // Signed zero & denormals
    let neg_zero = vec![-0.0f32, 0.0, -0.0];
    let pos_zero = vec![0.0f32, -0.0, 0.0];
    let denorm = vec![f32::MIN_POSITIVE * 0.5, f32::MIN_POSITIVE * 0.2, 0.0];

    let cos_s = cosine_distance_scalar(&neg_zero, &pos_zero);
    let cos_d = cosine_distance(&neg_zero, &pos_zero).unwrap();
    assert_eq!(cos_s, cos_d);

    let denorm_s = dot_product_scalar(&denorm, &denorm);
    let denorm_d = dot_product_distance(&denorm, &denorm).unwrap();
    assert!((denorm_s - denorm_d).abs() <= 1e-10);
}

#[test]
fn test_parallel_simd_concurrency() {
    use std::sync::Arc;
    use std::thread;

    let dim = 1024;
    let a = Arc::new(
        (0..dim)
            .map(|i| (i as f32 * 0.01).sin())
            .collect::<Vec<f32>>(),
    );
    let b = Arc::new(
        (0..dim)
            .map(|i| (i as f32 * 0.02).cos())
            .collect::<Vec<f32>>(),
    );

    let mut handles = vec![];
    for seed in 0..16 {
        let a_cloned = Arc::clone(&a);
        let b_cloned = Arc::clone(&b);
        handles.push(thread::spawn(move || {
            let expected_cos = cosine_distance(&a_cloned, &b_cloned).unwrap();
            let expected_euc = euclidean_distance(&a_cloned, &b_cloned).unwrap();
            let expected_dot = dot_product_distance(&a_cloned, &b_cloned).unwrap();

            for iter in 0..100 {
                let cos = cosine_distance(&a_cloned, &b_cloned).unwrap();
                let euc = euclidean_distance(&a_cloned, &b_cloned).unwrap();
                let dot = dot_product_distance(&a_cloned, &b_cloned).unwrap();

                assert_eq!(cos, expected_cos, "Thread {seed} iter {iter} cos mismatch");
                assert_eq!(euc, expected_euc, "Thread {seed} iter {iter} euc mismatch");
                assert_eq!(dot, expected_dot, "Thread {seed} iter {iter} dot mismatch");
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
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
