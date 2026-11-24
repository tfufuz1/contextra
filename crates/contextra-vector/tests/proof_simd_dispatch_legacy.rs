// FILE-CONTEXT
// ZWECK: Differenztest zwischen SIMD-Dispatch und Legacy-Skalar-Orakel für VEC-A 36-E (Wechsel auf contextra_simd::dispatch).
#![cfg(feature = "simd")]

mod legacy_reference;

use contextra_types::DistanceMetric;
use contextra_vector::distance::compute_distance;
use legacy_reference::{legacy_compute_distance, LegacyDistanceMetric};

#[test]
fn test_proof_simd_dispatch_matches_legacy_scalar_dims() {
    let dims = [4, 7, 8, 15, 16, 31, 32, 64, 128, 768];
    for &d in &dims {
        let v1: Vec<f32> = (0..d).map(|i| (i as f32 * 0.1).sin()).collect();
        let v2: Vec<f32> = (0..d).map(|i| (i as f32 * 0.2).cos()).collect();

        // Cosine
        let simd_cos = compute_distance(&v1, &v2, DistanceMetric::Cosine).unwrap();
        let leg_cos = legacy_compute_distance(&v1, &v2, LegacyDistanceMetric::Cosine).unwrap();
        assert!((simd_cos - leg_cos).abs() < 1e-4);

        // Euclidean
        let simd_euc = compute_distance(&v1, &v2, DistanceMetric::Euclidean).unwrap();
        let leg_euc = legacy_compute_distance(&v1, &v2, LegacyDistanceMetric::Euclidean).unwrap();
        assert!((simd_euc - leg_euc).abs() < 1e-4);

        // Dot
        let simd_dot = compute_distance(&v1, &v2, DistanceMetric::DotProduct).unwrap();
        let leg_dot = legacy_compute_distance(&v1, &v2, LegacyDistanceMetric::DotProduct).unwrap();
        assert!((simd_dot - leg_dot).abs() < 1e-3);
    }
}
