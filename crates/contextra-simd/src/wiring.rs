//! Facade wiring entry point for contextra-simd (Ring 0) per Rule 7.

use crate::dispatch::{
    cosine_distance, cosine_similarity_parts_u8, dot_product_distance, dot_product_u8,
    euclidean_distance,
};
use crate::kernels::scalar::CosineSimilarityPartsU8;
use contextra_types::ContextraError;

/// Wiring container for SIMD distance kernels and runtime feature dispatch.
#[derive(Debug, Clone, Default)]
pub struct SimdWiring;

impl SimdWiring {
    /// Builds the `SimdWiring` instance.
    pub fn build(_deps: ()) -> Result<Self, ContextraError> {
        Ok(Self)
    }

    /// Computes cosine distance between two `f32` slices using active hardware features.
    #[inline]
    pub fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
        cosine_distance(a, b)
    }

    /// Computes Euclidean distance between two `f32` slices using active hardware features.
    #[inline]
    pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
        euclidean_distance(a, b)
    }

    /// Computes dot product distance between two `f32` slices using active hardware features.
    #[inline]
    pub fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
        dot_product_distance(a, b)
    }

    /// Computes dot product between two `u8` quantized vector slices using active hardware features.
    #[inline]
    pub fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> Result<u32, ContextraError> {
        dot_product_u8(a, b)
    }

    /// Computes cosine similarity component parts between two `u8` slices using active hardware features.
    #[inline]
    pub fn cosine_similarity_parts_u8(
        &self,
        a: &[u8],
        b: &[u8],
    ) -> Result<CosineSimilarityPartsU8, ContextraError> {
        cosine_similarity_parts_u8(a, b)
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
}
