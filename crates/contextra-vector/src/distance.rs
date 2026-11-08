// FILE-CONTEXT
// ZWECK: Crate-lokale Distanz-Abstraktion und skalare Referenzimplementierung ohne contextra-simd (WP-VEC-3).
// INVARIANTEN: Zero unsafe code in distance.rs (#![forbid(unsafe_code)]); INV-SIMD-FINITE-1; INV-SIMD-ALIGN-1.
// HOTSPOTS: compute_distance, compute_distance_trusted, compute_distance_f32_bytes_trusted, ScalarKernel

//! # Distance Computation Module
//!
//! This module defines the crate-local [`DistanceKernel`] trait and the safe, allocation-free
//! scalar reference implementation [`ScalarKernel`] according to architecture principles §3.1 and §3.3.

use contextra_types::{ContextraError, DistanceMetric, Result};

/// Raw integer sum components for u8 cosine similarity calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CosineSimilarityPartsU8 {
    /// Accumulated dot product $\sum a_i b_i$.
    pub dot: u32,
    /// Accumulated squared L2 norm $\sum a_i^2$.
    pub norm_a_sq: u32,
    /// Accumulated squared L2 norm $\sum b_i^2$.
    pub norm_b_sq: u32,
}

/// Distance kernel interface defining vector metrics and quantized distance operations.
pub trait DistanceKernel: Send + Sync + 'static {
    /// Computes distance between two slices after validating finiteness.
    fn distance(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes distance between an f32 query and little-endian f32 byte slice after validating finiteness and alignment.
    fn distance_f32_bytes(&self, metric: DistanceMetric, a: &[f32], b_bytes: &[u8]) -> Result<f32>;

    /// Computes distance between pre-validated f32 slices.
    fn distance_trusted(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes distance between pre-validated f32 slice and LE f32 byte slice.
    fn distance_f32_bytes_trusted(
        &self,
        metric: DistanceMetric,
        a: &[f32],
        b_bytes: &[u8],
    ) -> Result<f32>;

    /// Computes cosine distance between two f32 slices.
    fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes dot product distance (negative dot product) between two f32 slices.
    fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes Euclidean distance between two f32 slices.
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes squared Euclidean distance between two u8 slices.
    fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> u32;

    /// Computes mixed f32/u8 squared Euclidean distance with dequantization.
    fn euclidean_distance_sq_f32_u8(
        &self,
        a: &[f32],
        b: &[u8],
        alphas: &[f32],
        mins: &[f32],
    ) -> f32;

    /// Computes unsigned integer dot product between two u8 byte slices.
    fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> u32;

    /// Computes mixed f32/u8 dot product.
    fn dot_product_f32_u8(&self, a: &[f32], b: &[u8]) -> f32;

    /// Computes raw component parts for u8 cosine similarity.
    fn cosine_similarity_parts_u8(&self, a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8;

    /// Normalizes a vector in-place to unit length.
    fn normalize_inplace(&self, v: &mut [f32]);
}

/// Complete, safe, scalar reference implementation using safe iterators and chunking.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScalarKernel;

impl ScalarKernel {
    #[inline]
    fn validate_slices(a: &[f32], b: &[f32]) -> Result<()> {
        if a.len() != b.len() {
            return Err(ContextraError::invalid_input(format!(
                "Vector dimensions must match: {} != {}",
                a.len(),
                b.len()
            )));
        }
        validate_vector(a)?;
        validate_vector(b)?;
        Ok(())
    }

    #[inline]
    fn validate_bytes(a: &[f32], b_bytes: &[u8]) -> Result<()> {
        if b_bytes.len() % 4 != 0 {
            return Err(ContextraError::invalid_input(format!(
                "Byte slice length {} is not aligned to 4 bytes",
                b_bytes.len()
            )));
        }
        if b_bytes.len() / 4 != a.len() {
            return Err(ContextraError::invalid_input(format!(
                "Dimension mismatch: f32 slice len {} != byte float count {}",
                a.len(),
                b_bytes.len() / 4
            )));
        }
        validate_vector(a)?;
        for chunk in b_bytes.chunks_exact(4) {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(chunk);
            let val = f32::from_le_bytes(arr);
            if !val.is_finite() {
                return Err(ContextraError::invalid_input(
                    "NaN or Infinity detected in vector bytes",
                ));
            }
        }
        Ok(())
    }
}

impl DistanceKernel for ScalarKernel {
    #[inline]
    fn distance(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32> {
        Self::validate_slices(a, b)?;
        self.distance_trusted(metric, a, b)
    }

    #[inline]
    fn distance_f32_bytes(&self, metric: DistanceMetric, a: &[f32], b_bytes: &[u8]) -> Result<f32> {
        Self::validate_bytes(a, b_bytes)?;
        self.distance_f32_bytes_trusted(metric, a, b_bytes)
    }

    #[inline]
    fn distance_trusted(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(ContextraError::invalid_input(format!(
                "Vector dimensions must match: {} != {}",
                a.len(),
                b.len()
            )));
        }
        match metric {
            DistanceMetric::Cosine => self.cosine_distance(a, b),
            DistanceMetric::Euclidean => self.euclidean_distance(a, b),
            DistanceMetric::DotProduct => self.dot_product_distance(a, b),
            _ => Err(ContextraError::invalid_input("Unsupported distance metric")),
        }
    }

    #[inline]
    fn distance_f32_bytes_trusted(
        &self,
        metric: DistanceMetric,
        a: &[f32],
        b_bytes: &[u8],
    ) -> Result<f32> {
        if b_bytes.len() % 4 != 0 || b_bytes.len() / 4 != a.len() {
            return Err(ContextraError::invalid_input(format!(
                "Byte alignment or length mismatch: b_bytes len {}, a len {}",
                b_bytes.len(),
                a.len()
            )));
        }
        match metric {
            DistanceMetric::Cosine => {
                let mut dot = 0.0f32;
                let mut norm_a = 0.0f32;
                let mut norm_b = 0.0f32;

                for (&x, chunk) in a.iter().zip(b_bytes.chunks_exact(4)) {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(chunk);
                    let b_val = f32::from_le_bytes(arr);
                    dot += x * b_val;
                    norm_a += x * x;
                    norm_b += b_val * b_val;
                }

                if norm_a == 0.0 || norm_b == 0.0 {
                    Ok(1.0)
                } else {
                    let denom = norm_a.sqrt() * norm_b.sqrt();
                    let sim = (dot / denom).clamp(-1.0, 1.0);
                    Ok(1.0 - sim)
                }
            }
            DistanceMetric::Euclidean => {
                let mut sum = 0.0f32;
                for (&x, chunk) in a.iter().zip(b_bytes.chunks_exact(4)) {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(chunk);
                    let b_val = f32::from_le_bytes(arr);
                    let diff = x - b_val;
                    sum += diff * diff;
                }
                Ok(sum.sqrt())
            }
            DistanceMetric::DotProduct => {
                let mut dot = 0.0f32;
                for (&x, chunk) in a.iter().zip(b_bytes.chunks_exact(4)) {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(chunk);
                    let b_val = f32::from_le_bytes(arr);
                    dot += x * b_val;
                }
                Ok(-dot)
            }
            _ => Err(ContextraError::invalid_input("Unsupported distance metric")),
        }
    }

    #[inline]
    fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(ContextraError::invalid_input(
                "Vector dimensions must match",
            ));
        }
        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;

        for (&x, &y) in a.iter().zip(b.iter()) {
            dot += x * y;
            norm_a += x * x;
            norm_b += y * y;
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(1.0);
        }

        let denom = norm_a.sqrt() * norm_b.sqrt();
        let sim = (dot / denom).clamp(-1.0, 1.0);
        Ok(1.0 - sim)
    }

    #[inline]
    fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(ContextraError::invalid_input(
                "Vector dimensions must match",
            ));
        }
        let mut dot = 0.0f32;
        for (&x, &y) in a.iter().zip(b.iter()) {
            dot += x * y;
        }
        Ok(-dot)
    }

    #[inline]
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(ContextraError::invalid_input(
                "Vector dimensions must match",
            ));
        }
        let mut sum = 0.0f32;
        for (&x, &y) in a.iter().zip(b.iter()) {
            let diff = x - y;
            sum += diff * diff;
        }
        Ok(sum.sqrt())
    }

    #[inline]
    fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> u32 {
        let mut sum = 0u32;
        for (&x, &y) in a.iter().zip(b.iter()) {
            let diff = (x as i32) - (y as i32);
            let abs_diff = diff.unsigned_abs();
            sum += abs_diff * abs_diff;
        }
        sum
    }

    #[inline]
    fn euclidean_distance_sq_f32_u8(
        &self,
        a: &[f32],
        b: &[u8],
        alphas: &[f32],
        mins: &[f32],
    ) -> f32 {
        let mut sum = 0.0f32;
        for (((&x, &y_u8), &alpha), &min_val) in
            a.iter().zip(b.iter()).zip(alphas.iter()).zip(mins.iter())
        {
            let y_f32 = (y_u8 as f32) * alpha + min_val;
            let diff = x - y_f32;
            sum += diff * diff;
        }
        sum
    }

    #[inline]
    fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> u32 {
        let mut dot = 0u32;
        for (&x, &y) in a.iter().zip(b.iter()) {
            dot += (x as u32) * (y as u32);
        }
        dot
    }

    #[inline]
    fn dot_product_f32_u8(&self, a: &[f32], b: &[u8]) -> f32 {
        let mut dot = 0.0f32;
        for (&x, &y) in a.iter().zip(b.iter()) {
            dot += x * (y as f32);
        }
        dot
    }

    #[inline]
    fn cosine_similarity_parts_u8(&self, a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
        let mut dot = 0u32;
        let mut norm_a_sq = 0u32;
        let mut norm_b_sq = 0u32;

        for (&x_u8, &y_u8) in a.iter().zip(b.iter()) {
            let x = x_u8 as u32;
            let y = y_u8 as u32;
            dot += x * y;
            norm_a_sq += x * x;
            norm_b_sq += y * y;
        }

        CosineSimilarityPartsU8 {
            dot,
            norm_a_sq,
            norm_b_sq,
        }
    }

    #[inline]
    fn normalize_inplace(&self, v: &mut [f32]) {
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            let inv_norm = 1.0 / norm;
            for x in v.iter_mut() {
                *x *= inv_norm;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Standalone Functions & Backward Compatibility Helper
// ---------------------------------------------------------------------------

/// Validates that a vector does not contain NaN or infinite values (INV-SIMD-FINITE-1).
#[inline]
pub fn validate_vector(vec: &[f32]) -> Result<()> {
    if vec.iter().any(|v| !v.is_finite()) {
        return Err(ContextraError::invalid_input(
            "NaN or Infinity detected in vector",
        ));
    }
    Ok(())
}

#[inline]
pub fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32> {
    ScalarKernel.distance(metric, a, b)
}

#[inline]
pub fn compute_distance_trusted(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32> {
    ScalarKernel.distance_trusted(metric, a, b)
}

#[inline]
pub fn compute_distance_f32_bytes_trusted(
    a: &[f32],
    b_bytes: &[u8],
    metric: DistanceMetric,
) -> Result<f32> {
    ScalarKernel.distance_f32_bytes_trusted(metric, a, b_bytes)
}

#[inline]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32> {
    ScalarKernel.cosine_distance(a, b)
}

#[inline]
pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32> {
    ScalarKernel.dot_product_distance(a, b)
}

#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32> {
    ScalarKernel.euclidean_distance(a, b)
}

#[inline]
pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> u32 {
    ScalarKernel.euclidean_distance_sq_u8(a, b)
}

#[inline]
pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32 {
    ScalarKernel.euclidean_distance_sq_f32_u8(a, b, alphas, mins)
}

#[inline]
pub fn dot_product_u8(a: &[u8], b: &[u8]) -> u32 {
    ScalarKernel.dot_product_u8(a, b)
}

#[inline]
pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32 {
    ScalarKernel.dot_product_f32_u8(a, b)
}

#[inline]
pub fn cosine_similarity_parts_u8(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
    ScalarKernel.cosine_similarity_parts_u8(a, b)
}

#[inline]
pub fn normalize_inplace(v: &mut [f32]) {
    ScalarKernel.normalize_inplace(v);
}

#[inline]
pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    ScalarKernel.cosine_distance(a, b).unwrap_or(1.0)
}

#[inline]
pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    ScalarKernel.dot_product_distance(a, b).unwrap_or(0.0)
}

#[inline]
pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    ScalarKernel.euclidean_distance(a, b).unwrap_or(0.0)
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

        let dot_scalar = dot_product_scalar(&a, &b);
        let d = compute_distance(&a, &b, DistanceMetric::DotProduct).expect("test");
        assert!((dot_scalar - d).abs() < 1e-6);

        let euc_scalar = euclidean_distance_scalar(&a, &b);
        let euc_simd = compute_distance(&a, &b, DistanceMetric::Euclidean).expect("test");
        assert!((euc_scalar - euc_simd).abs() < 1e-6);

        let cos_scalar = cosine_distance_scalar(&a, &b);
        let cos_simd = compute_distance(&a, &b, DistanceMetric::Cosine).expect("test");
        assert!((cos_scalar - cos_simd).abs() < 1e-6);
    }
}
