// FILE-CONTEXT
// ZWECK: Differenztest zwischen aktueller öffentlicher Distanzberechnung und gefrorener Legacy-Kopie (proptest und Randfälle).

mod legacy_reference;

use contextra_types::DistanceMetric;
use contextra_vector::distance::{
    compute_distance, cosine_distance_scalar, dot_product_scalar, euclidean_distance_scalar,
    validate_vector,
};
use legacy_reference::{
    legacy_compute_distance, legacy_cosine_distance_scalar, legacy_dot_product_scalar,
    legacy_euclidean_distance_scalar, LegacyDistanceMetric,
};
use proptest::prelude::*;

#[test]
fn test_proof_distance_legacy_edge_cases() {
    let dims = [4, 17, 128, 768];
    for &d in &dims {
        let v1 = vec![1.0f32; d];
        let v2 = vec![2.0f32; d];
        let zero = vec![0.0f32; d];
        let mut denorm = vec![1.0e-38f32; d];
        denorm[0] = 1.0e-40f32;

        let cos_curr = cosine_distance_scalar(&v1, &v2);
        let cos_leg = legacy_cosine_distance_scalar(&v1, &v2);
        assert!((cos_curr - cos_leg).abs() < 1e-6);

        let cos_zero = cosine_distance_scalar(&v1, &zero);
        let cos_zero_leg = legacy_cosine_distance_scalar(&v1, &zero);
        assert_eq!(cos_zero, cos_zero_leg);

        let euc_curr = euclidean_distance_scalar(&v1, &v2);
        let euc_leg = legacy_euclidean_distance_scalar(&v1, &v2);
        assert!((euc_curr - euc_leg).abs() < 1e-5);

        let dot_curr = dot_product_scalar(&v1, &v2);
        let dot_leg = legacy_dot_product_scalar(&v1, &v2);
        assert!((dot_curr - dot_leg).abs() < 1e-5);

        let d_denorm = euclidean_distance_scalar(&v1, &denorm);
        let d_denorm_leg = legacy_euclidean_distance_scalar(&v1, &denorm);
        assert!((d_denorm - d_denorm_leg).abs() < 1e-5);
    }

    let nan_vec = vec![1.0, f32::NAN, 3.0];
    let inf_vec = vec![1.0, f32::INFINITY, 3.0];
    let normal_vec = vec![1.0, 2.0, 3.0];

    assert!(validate_vector(&nan_vec).is_err());
    assert!(validate_vector(&inf_vec).is_err());
    assert!(legacy_compute_distance(&nan_vec, &normal_vec, LegacyDistanceMetric::Cosine).is_err());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn test_proof_distance_legacy_proptest_cosine(
        vec_a in prop::collection::vec(-10.0f32..10.0f32, 16),
        vec_b in prop::collection::vec(-10.0f32..10.0f32, 16)
    ) {
        let curr = compute_distance(&vec_a, &vec_b, DistanceMetric::Cosine).unwrap();
        let leg = legacy_compute_distance(&vec_a, &vec_b, LegacyDistanceMetric::Cosine).unwrap();
        let diff = (curr - leg).abs();
        prop_assert!(diff < 1e-5 || diff / leg.abs().max(1e-5) < 1e-4);
    }

    #[test]
    fn test_proof_distance_legacy_proptest_euclidean(
        vec_a in prop::collection::vec(-10.0f32..10.0f32, 16),
        vec_b in prop::collection::vec(-10.0f32..10.0f32, 16)
    ) {
        let curr = compute_distance(&vec_a, &vec_b, DistanceMetric::Euclidean).unwrap();
        let leg = legacy_compute_distance(&vec_a, &vec_b, LegacyDistanceMetric::Euclidean).unwrap();
        let diff = (curr - leg).abs();
        prop_assert!(diff < 1e-5 || diff / leg.abs().max(1e-5) < 1e-4);
    }

    #[test]
    fn test_proof_distance_legacy_proptest_dot(
        vec_a in prop::collection::vec(-10.0f32..10.0f32, 16),
        vec_b in prop::collection::vec(-10.0f32..10.0f32, 16)
    ) {
        let curr = compute_distance(&vec_a, &vec_b, DistanceMetric::DotProduct).unwrap();
        let leg = legacy_compute_distance(&vec_a, &vec_b, LegacyDistanceMetric::DotProduct).unwrap();
        let diff = (curr - leg).abs();
        prop_assert!(diff < 1e-4 || diff / leg.abs().max(1e-4) < 1e-4);
    }
}
