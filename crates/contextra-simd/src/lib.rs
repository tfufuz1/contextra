// FILE-CONTEXT
// ZWECK: Layer-0 Ring-0 Unsafe-Insel für SIMD-Distanzkernel und Laufzeit-Dispatch.
// INVARIANTEN: Safe Public API, zero panic, bytemuck/slice alignments.
// HOTSPOTS: lib.rs, kernels/, dispatch.rs

//! Contextra SIMD — Ring 0 SIMD distance kernels and hardware runtime dispatch.

#![deny(unsafe_code)]

pub mod dispatch;
pub mod kernels;

pub use dispatch::*;
pub use kernels::scalar::{
    cosine_distance_f32_bytes_scalar, cosine_distance_scalar, cosine_similarity_parts_f32_u8,
    cosine_similarity_parts_u8_scalar, dot_product_f32_bytes_scalar, dot_product_f32_u8,
    dot_product_scalar, dot_product_u8_scalar, euclidean_distance_f32_bytes_scalar,
    euclidean_distance_scalar, euclidean_distance_sq_f32_u8, euclidean_distance_sq_u8_scalar,
    normalize_inplace, CosineSimilarityPartsF32U8, CosineSimilarityPartsU8,
};

use contextra_core::{ContextraError, DistanceMetric};

/// Validates that a vector contains no NaN or Infinite values.
///
/// # Errors
/// Returns [`ContextraError::InvalidInput`] if any element in `vec` is NaN or Infinity.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) linear scan over vector elements.
#[inline]
pub fn validate_vector(vec: &[f32]) -> contextra_core::Result<()> {
    if vec.iter().any(|v| !v.is_finite()) {
        return Err(ContextraError::invalid_input(
            "NaN or Infinity detected in vector",
        ));
    }
    Ok(())
}

/// Computes the distance between two vectors `a` and `b` according to `metric` after validating elements.
///
/// # Errors
/// Returns [`ContextraError::InvalidInput`] if `a` or `b` contain non-finite numbers, or if `metric` is unsupported.
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a` and `b` have different lengths.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is vector dimension.
#[inline]
pub fn compute_distance(
    a: &[f32],
    b: &[f32],
    metric: DistanceMetric,
) -> contextra_core::Result<f32> {
    validate_vector(a)?;
    validate_vector(b)?;
    compute_distance_trusted(a, b, metric)
}

/// Computes the distance between two pre-validated vectors `a` and `b` according to `metric`.
///
/// # Errors
/// Returns [`ContextraError::InvalidInput`] if `metric` is unsupported.
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a` and `b` have different lengths.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is vector dimension.
#[inline]
pub fn compute_distance_trusted(
    a: &[f32],
    b: &[f32],
    metric: DistanceMetric,
) -> contextra_core::Result<f32> {
    match metric {
        DistanceMetric::Cosine => cosine_distance(a, b),
        DistanceMetric::Euclidean => euclidean_distance(a, b),
        DistanceMetric::DotProduct => dot_product_distance(a, b),
        _ => Err(ContextraError::invalid_input("Unsupported distance metric")),
    }
}

/// Computes the distance between pre-validated f32 vector `a` and byte slice `b_bytes` according to `metric`.
///
/// # Errors
/// Returns [`ContextraError::InvalidInput`] if `metric` is unsupported.
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `b_bytes` has fewer bytes than `a.len() * 4`.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is vector dimension (`a.len()`).
#[inline]
pub fn compute_distance_f32_bytes_trusted(
    a: &[f32],
    b_bytes: &[u8],
    metric: DistanceMetric,
) -> contextra_core::Result<f32> {
    match metric {
        DistanceMetric::Cosine => cosine_distance_f32_bytes(a, b_bytes),
        DistanceMetric::Euclidean => euclidean_distance_f32_bytes(a, b_bytes),
        DistanceMetric::DotProduct => dot_product_distance_f32_bytes(a, b_bytes),
        _ => Err(ContextraError::invalid_input("Unsupported distance metric")),
    }
}
