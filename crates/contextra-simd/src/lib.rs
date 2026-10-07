// FILE-CONTEXT
// ZWECK: Layer-0 Ring-0 Unsafe-Insel für SIMD-Distanzkernel und Laufzeit-Dispatch.
// INVARIANTEN: Safe Public API, zero panic, bytemuck/slice alignments.
// HOTSPOTS: lib.rs, kernels/, dispatch.rs

//! Contextra SIMD — Ring 0 SIMD distance kernels and hardware runtime dispatch.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod dispatch;
pub mod kernels;

pub use dispatch::{
    cosine_distance, cosine_distance_f32_bytes, cosine_similarity_parts_u8, detect,
    dot_product_distance, dot_product_distance_f32_bytes, dot_product_u8, euclidean_distance,
    euclidean_distance_f32_bytes, euclidean_distance_sq_u8, CpuFeatures,
};
pub use kernels::scalar::{
    dot_product_f32_u8, euclidean_distance_sq_f32_u8, normalize_inplace,
    CosineSimilarityPartsF32U8, CosineSimilarityPartsU8,
};

use contextra_core::{ContextraError, DistanceMetric};

/// Validates that a vector contains no NaN or Infinite values.
#[inline]
pub fn validate_vector(vec: &[f32]) -> contextra_core::Result<()> {
    if vec.iter().any(|v| !v.is_finite()) {
        return Err(ContextraError::invalid_input(
            "NaN or Infinity detected in vector",
        ));
    }
    Ok(())
}

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
