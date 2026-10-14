//! Facade wiring entry point for contextra-simd (Ring 0) per Rule 7.

use crate::dispatch::{
    cosine_distance_with_features, cosine_similarity_parts_u8_with_features,
    dot_product_distance_with_features, dot_product_u8_with_features,
    euclidean_distance_with_features, CpuFeatures,
};
use crate::kernels::scalar::CosineSimilarityPartsU8;
use contextra_core::ContextraError;

/// Wiring container for SIMD distance kernels and runtime feature dispatch.
#[derive(Debug, Clone)]
pub struct SimdWiring {
    features: CpuFeatures,
}

impl SimdWiring {
    /// Builds the `SimdWiring` instance with auto-detected CPU features.
    pub fn build(_deps: ()) -> Result<Self, ContextraError> {
        Ok(Self {
            features: CpuFeatures::detect(),
        })
    }

    /// Constructs `SimdWiring` with explicitly provided CPU feature token.
    pub fn with_features(features: CpuFeatures) -> Self {
        Self { features }
    }

    /// Returns a reference to the active CPU feature token.
    #[inline]
    pub fn features(&self) -> &CpuFeatures {
        &self.features
    }

    /// Computes cosine distance between two `f32` slices using active hardware features.
    #[inline]
    pub fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
        cosine_distance_with_features(a, b, &self.features)
    }

    /// Computes Euclidean distance between two `f32` slices using active hardware features.
    #[inline]
    pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
        euclidean_distance_with_features(a, b, &self.features)
    }

    /// Computes dot product distance between two `f32` slices using active hardware features.
    #[inline]
    pub fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
        dot_product_distance_with_features(a, b, &self.features)
    }

    /// Computes dot product between two `u8` quantized vector slices using active hardware features.
    #[inline]
    pub fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> Result<u32, ContextraError> {
        dot_product_u8_with_features(a, b, &self.features)
    }

    /// Computes cosine similarity component parts between two `u8` slices using active hardware features.
    #[inline]
    pub fn cosine_similarity_parts_u8(
        &self,
        a: &[u8],
        b: &[u8],
    ) -> Result<CosineSimilarityPartsU8, ContextraError> {
        cosine_similarity_parts_u8_with_features(a, b, &self.features)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_wiring_build() {
        let wiring = SimdWiring::build(()).expect("SimdWiring build failed");
        let a = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = vec![4.0f32, 3.0, 2.0, 1.0];

        let dist = wiring
            .cosine_distance(&a, &b)
            .expect("cosine_distance failed");
        assert!(dist > 0.0);
    }

    #[test]
    fn test_forced_scalar_parity() {
        let hw_wiring = SimdWiring::build(()).expect("build failed");

        // Force scalar detection token
        let scalar_features = {
            std::env::set_var("CONTEXTRA_SIMD_FORCE", "scalar");
            let sf = CpuFeatures::detect();
            std::env::remove_var("CONTEXTRA_SIMD_FORCE");
            sf
        };

        let scalar_wiring = SimdWiring::with_features(scalar_features);

        let a = vec![0.5f32, -0.2, 0.8, 1.2, 0.1, -0.9, 0.4, 0.3];
        let b = vec![0.1f32, 0.7, -0.3, 0.4, 0.9, 0.2, -0.1, 0.8];

        let cos_hw = hw_wiring.cosine_distance(&a, &b).unwrap();
        let cos_scalar = scalar_wiring.cosine_distance(&a, &b).unwrap();
        assert!(
            (cos_hw - cos_scalar).abs() < 1e-6,
            "Cosine distance mismatch: hw={cos_hw}, scalar={cos_scalar}"
        );

        let euc_hw = hw_wiring.euclidean_distance(&a, &b).unwrap();
        let euc_scalar = scalar_wiring.euclidean_distance(&a, &b).unwrap();
        assert!(
            (euc_hw - euc_scalar).abs() < 1e-6,
            "Euclidean distance mismatch: hw={euc_hw}, scalar={euc_scalar}"
        );

        let dot_hw = hw_wiring.dot_product_distance(&a, &b).unwrap();
        let dot_scalar = scalar_wiring.dot_product_distance(&a, &b).unwrap();
        assert!(
            (dot_hw - dot_scalar).abs() < 1e-6,
            "Dot product mismatch: hw={dot_hw}, scalar={dot_scalar}"
        );
    }
}
