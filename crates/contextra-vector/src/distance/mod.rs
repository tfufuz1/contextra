// FILE-CONTEXT
// ZWECK: Crate-lokale Distanz-Abstraktion und skalare Referenzimplementierung ohne contextra-simd (WP-VEC-06).
// INVARIANTEN: Zero unsafe code in distance (#![forbid(unsafe_code)]); INV-SIMD-FINITE-1; INV-SIMD-ALIGN-1.
// HOTSPOTS: compute_distance, compute_distance_trusted, compute_distance_f32_bytes_trusted, ScalarKernel, ScalarDistance
// STAND: 2026-11-12

//! # Distance Computation Module
//!
//! Dieses Modul definiert den Crate-lokalen [`DistanceKernel`] Trait, den Vertragsadapter [`ScalarDistance`],
//! sowie die sichere, allokationsfreie skalare Referenzimplementierung [`ScalarKernel`].

pub use crate::contract;

mod dispatch;
mod inject;
mod kernel;
mod scalar;
mod u8_ops;
mod validate;

#[doc(inline)]
pub use dispatch::{
    compute_distance, compute_distance_f32_bytes_trusted, compute_distance_trusted,
    cosine_distance, dot_product_distance, euclidean_distance,
};
#[doc(inline)]
pub use inject::InjectedDistanceKernel;
#[doc(inline)]
pub use kernel::{CosineSimilarityPartsU8, DistanceKernel};
#[doc(inline)]
pub use scalar::{
    cosine_distance_scalar, dot_product_scalar, euclidean_distance_scalar, ScalarDistance,
    ScalarKernel,
};
#[doc(inline)]
pub use u8_ops::{
    cosine_similarity_parts_u8, dot_product_f32_u8, dot_product_u8, euclidean_distance_sq_f32_u8,
    euclidean_distance_sq_u8, normalize_inplace,
};
#[doc(inline)]
pub use validate::validate_vector;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Metric as ContractMetric;

    #[test]
    fn test_scalar_distance_contract() {
        let sd = ScalarDistance::new(ContractMetric::Euclidean);
        let a = [0.0, 3.0];
        let b = [4.0, 0.0];
        assert_eq!(sd.compute(&a, &b).unwrap(), 5.0);

        let sd_cos = ScalarDistance::new(ContractMetric::Cosine);
        let c1 = [1.0, 0.0];
        let c2 = [0.0, 1.0];
        assert!((sd_cos.compute(&c1, &c2).unwrap() - 1.0).abs() < 1e-6);

        let sd_dot = ScalarDistance::new(ContractMetric::DotProduct);
        assert_eq!(sd_dot.compute(&[2.0, 3.0], &[4.0, 5.0]).unwrap(), -23.0);
    }

    #[test]
    fn test_scalar_distance_zero_vector_cosine() {
        let sd = ScalarDistance::new(ContractMetric::Cosine);
        let z = [0.0, 0.0];
        let v = [1.0, 2.0];
        assert_eq!(sd.compute(&z, &v).unwrap(), 1.0);
    }

    #[test]
    fn test_scalar_distance_dimension_mismatch() {
        let sd = ScalarDistance::new(ContractMetric::Euclidean);
        let a = [1.0, 2.0];
        let b = [1.0, 2.0, 3.0];
        assert!(sd.compute(&a, &b).is_err());
    }

    #[test]
    fn test_scalar_distance_non_finite() {
        let sd = ScalarDistance::new(ContractMetric::Euclidean);
        let a = [1.0, f32::NAN];
        let b = [1.0, 2.0];
        assert!(sd.compute(&a, &b).is_err());

        let inf = [1.0, f32::INFINITY];
        assert!(sd.compute(&a, &inf).is_err());
    }

    #[test]
    fn test_u8_ops_consistency() {
        let a = [10u8, 20u8, 30u8];
        let b = [12u8, 25u8, 30u8];
        let sq = euclidean_distance_sq_u8(&a, &b);
        assert_eq!(sq, 4 + 25 + 0);

        let dot = dot_product_u8(&a, &b);
        assert_eq!(dot, 10 * 12 + 20 * 25 + 30 * 30);

        let parts = cosine_similarity_parts_u8(&a, &b);
        assert_eq!(parts.dot, dot);
        assert_eq!(parts.norm_a_sq, 100 + 400 + 900);
        assert_eq!(parts.norm_b_sq, 144 + 625 + 900);
    }

    #[test]
    fn test_normalize_inplace() {
        let mut v = [3.0f32, 4.0f32];
        normalize_inplace(&mut v);
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);
    }
}
