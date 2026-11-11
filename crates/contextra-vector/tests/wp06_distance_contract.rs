// FILE-CONTEXT
// ZWECK: Integrations- und Vertragstests für Distanzfunktionen (WP-VEC-06).
// INVARIANTEN: Längenprüfung, NaN/Inf-Abweisung, Dreiecksungleichung, Symmetrie, SIMD vs Skalar.
// STAND: 2026-11-12

use contextra_vector::distance::{
    compute_distance, cosine_distance, dot_product_distance, euclidean_distance, validate_vector,
};
use contextra_vector::error::VectorError;
use contextra_vector::types::DistanceMetric;
use proptest::prelude::*;

#[test]
fn test_length_mismatch_fails_with_invalid_input() {
    let a = [1.0f32, 2.0, 3.0];
    let b = [1.0f32, 2.0];

    let res_cos = compute_distance(&a, &b, DistanceMetric::Cosine);
    assert!(res_cos.is_err());
    assert!(matches!(res_cos.unwrap_err(), VectorError::InvalidInput(_)));

    let res_euc = compute_distance(&a, &b, DistanceMetric::Euclidean);
    assert!(res_euc.is_err());
    assert!(matches!(res_euc.unwrap_err(), VectorError::InvalidInput(_)));

    let res_dot = compute_distance(&a, &b, DistanceMetric::DotProduct);
    assert!(res_dot.is_err());
    assert!(matches!(res_dot.unwrap_err(), VectorError::InvalidInput(_)));
}

#[test]
fn test_nan_and_inf_rejected() {
    let valid = [1.0f32, 2.0, 3.0];
    let nan_vec = [1.0f32, f32::NAN, 3.0];
    let pos_inf_vec = [1.0f32, f32::INFINITY, 3.0];
    let neg_inf_vec = [1.0f32, f32::NEG_INFINITY, 3.0];

    assert!(validate_vector(&nan_vec).is_err());
    assert!(validate_vector(&pos_inf_vec).is_err());
    assert!(validate_vector(&neg_inf_vec).is_err());

    assert!(compute_distance(&valid, &nan_vec, DistanceMetric::Cosine).is_err());
    assert!(compute_distance(&nan_vec, &valid, DistanceMetric::Euclidean).is_err());
    assert!(compute_distance(&valid, &pos_inf_vec, DistanceMetric::DotProduct).is_err());
    assert!(compute_distance(&neg_inf_vec, &valid, DistanceMetric::Cosine).is_err());
}

#[test]
fn test_empty_vectors() {
    let empty_a: [f32; 0] = [];
    let empty_b: [f32; 0] = [];

    assert!(validate_vector(&empty_a).is_ok());

    let cos_res = compute_distance(&empty_a, &empty_b, DistanceMetric::Cosine).unwrap();
    assert_eq!(cos_res, 1.0);

    let euc_res = compute_distance(&empty_a, &empty_b, DistanceMetric::Euclidean).unwrap();
    assert_eq!(euc_res, 0.0);

    let dot_res = compute_distance(&empty_a, &empty_b, DistanceMetric::DotProduct).unwrap();
    assert_eq!(dot_res, -0.0);
}

#[test]
fn test_zero_norm_cosine() {
    let zero = [0.0f32, 0.0, 0.0];
    let v = [1.0f32, 2.0, 3.0];

    let cos_zero = compute_distance(&zero, &v, DistanceMetric::Cosine).unwrap();
    assert_eq!(cos_zero, 1.0);

    let cos_both_zero = compute_distance(&zero, &zero, DistanceMetric::Cosine).unwrap();
    assert_eq!(cos_both_zero, 1.0);
}

#[test]
fn test_symmetry_and_triangle_inequality() {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [4.0f32, 3.0, 2.0, 1.0];
    let c = [2.0f32, 2.0, 2.0, 2.0];

    let d_ab = euclidean_distance(&a, &b).unwrap();
    let d_ba = euclidean_distance(&b, &a).unwrap();
    assert!((d_ab - d_ba).abs() < 1e-6);

    let d_bc = euclidean_distance(&b, &c).unwrap();
    let d_ac = euclidean_distance(&a, &c).unwrap();
    assert!(d_ac <= d_ab + d_bc + 1e-6);

    let cos_ab = cosine_distance(&a, &b).unwrap();
    let cos_ba = cosine_distance(&b, &a).unwrap();
    assert!((cos_ab - cos_ba).abs() < 1e-6);

    let dot_ab = dot_product_distance(&a, &b).unwrap();
    let dot_ba = dot_product_distance(&b, &a).unwrap();
    assert!((dot_ab - dot_ba).abs() < 1e-6);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_distance_invariants(
        v1 in prop::collection::vec(-100.0..100.0f32, 16..64),
        v2 in prop::collection::vec(-100.0..100.0f32, 16..64)
    ) {
        let len = v1.len().min(v2.len());
        let a = &v1[..len];
        let b = &v2[..len];

        let euc = euclidean_distance(a, b).unwrap();
        assert!(euc >= 0.0);

        let cos = cosine_distance(a, b).unwrap();
        assert!((0.0..=2.0001).contains(&cos), "Cosine distance out of bounds: {cos}");

        let dot1 = dot_product_distance(a, b).unwrap();
        let dot2 = dot_product_distance(b, a).unwrap();
        assert!((dot1 - dot2).abs() < 1e-4 * (dot1.abs().max(1.0)));
    }
}
