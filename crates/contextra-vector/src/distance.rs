// FILE-CONTEXT
// ZWECK: Distanzberechnungen und Vektor-Dequantisierung via contextra-simd.
// INVARIANTEN: Zero unsafe code in distance.rs; Delegierung an contextra-simd.
// HOTSPOTS: compute_distance, compute_distance_trusted, compute_distance_f32_bytes_trusted

//! # Distance Computation Module
//!
//! This module adapts distance metric computations and vector validation from `contextra_simd`.

pub use contextra_simd::kernels::scalar::{
    cosine_distance_scalar, dot_product_scalar, euclidean_distance_scalar,
};
pub use contextra_simd::{
    cosine_distance, cosine_similarity_parts_u8, dot_product_distance, dot_product_f32_u8,
    dot_product_u8, euclidean_distance, euclidean_distance_sq_f32_u8, euclidean_distance_sq_u8,
    normalize_inplace,
};

use contextra_types::{ContextraError, DistanceMetric, Result};

#[inline]
pub fn validate_vector(vec: &[f32]) -> Result<()> {
    contextra_simd::validate_vector(vec).map_err(|e| ContextraError::invalid_input(e.to_string()))
}

#[inline]
pub fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32> {
    contextra_simd::compute_distance(a, b, metric.into()).map_err(|e| ContextraError::invalid_input(e.to_string()))
}

#[inline]
pub fn compute_distance_trusted(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32> {
    contextra_simd::compute_distance_trusted(a, b, metric.into()).map_err(|e| ContextraError::invalid_input(e.to_string()))
}

#[inline]
pub fn compute_distance_f32_bytes_trusted(a: &[f32], b_bytes: &[u8], metric: DistanceMetric) -> Result<f32> {
    contextra_simd::compute_distance_f32_bytes_trusted(a, b_bytes, metric.into()).map_err(|e| ContextraError::invalid_input(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::DistanceMetric;

    #[test]
    fn test_compute_distance_f32_bytes_equivalence() {
        let dims = [1, 7, 16, 32, 64, 128, 768];
        let metrics = [
            DistanceMetric::Cosine,
            DistanceMetric::Euclidean,
            DistanceMetric::DotProduct,
        ];

        for &dim in &dims {
            let query: Vec<f32> = (0..dim).map(|i| (i as f32 * 0.17).sin()).collect();
            let target: Vec<f32> = (0..dim).map(|i| ((i + 3) as f32 * 0.23).cos()).collect();

            let mut target_bytes = Vec::with_capacity(dim * 4);
            for &val in &target {
                target_bytes.extend_from_slice(&val.to_le_bytes());
            }

            for &metric in &metrics {
                let dist_trusted = compute_distance_trusted(&query, &target, metric).unwrap();
                let dist_bytes =
                    compute_distance_f32_bytes_trusted(&query, &target_bytes, metric).unwrap();

                let diff = (dist_trusted - dist_bytes).abs();
                assert!(
                    diff < 1e-5,
                    "Unaligned byte kernel mismatch for {:?} at dim {dim}: trusted={dist_trusted}, bytes={dist_bytes}, diff={diff}",
                    metric
                );
            }
        }
    }

    #[test]
    fn test_public_compute_distance_api_sanity() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];

        let cos = compute_distance(&v1, &v2, DistanceMetric::Cosine).expect("cosine");
        assert!((cos - 1.0).abs() < 1e-5);

        let euc = compute_distance(&v1, &v2, DistanceMetric::Euclidean).expect("euclidean");
        assert!((euc - 2.0_f32.sqrt()).abs() < 1e-5);

        let dot = compute_distance(&v1, &v2, DistanceMetric::DotProduct).expect("dot");
        assert_eq!(dot, 0.0);
    }

    #[test]
    fn test_distances_match_scalar() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let b = vec![9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

        let dot_scalar = contextra_simd::kernels::scalar::dot_product_scalar(&a, &b);
        let d = compute_distance(&a, &b, DistanceMetric::DotProduct).expect("test");
        let dot_simd = d;
        assert!((dot_scalar - dot_simd).abs() < 1e-3);

        let euc_scalar = contextra_simd::kernels::scalar::euclidean_distance_scalar(&a, &b);
        let euc_simd = compute_distance(&a, &b, DistanceMetric::Euclidean).expect("test");
        assert!((euc_scalar - euc_simd).abs() < 1e-3);

        let cos_scalar = contextra_simd::kernels::scalar::cosine_distance_scalar(&a, &b);
        let cos_simd = compute_distance(&a, &b, DistanceMetric::Cosine).expect("test");
        assert!((cos_scalar - cos_simd).abs() < 1e-3);
    }
}
