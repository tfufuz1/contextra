// FILE-CONTEXT
// ZWECK: Layer-0 Ring-0 Unsafe-Insel für SIMD-Distanzkernel und Laufzeit-Dispatch.
// INVARIANTEN: Safe Public API, zero panic, bytemuck/slice alignments.
// HOTSPOTS: lib.rs, kernels/, dispatch.rs

//! MemFuse SIMD — Ring 0 SIMD distance kernels and hardware runtime dispatch.

#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

pub mod dispatch;
pub mod kernels;

pub use dispatch::*;
#[cfg(target_arch = "aarch64")]
pub use kernels::neon;
pub use kernels::scalar::{
    cosine_distance_f32_bytes_scalar, cosine_distance_scalar, cosine_similarity_parts_f32_u8,
    cosine_similarity_parts_u8_scalar, dot_product_f32_bytes_scalar, dot_product_f32_u8,
    dot_product_scalar, dot_product_u8_scalar, euclidean_distance_f32_bytes_scalar,
    euclidean_distance_scalar, euclidean_distance_sq_f32_u8, euclidean_distance_sq_u8_scalar,
    normalize_inplace, CosineSimilarityPartsF32U8, CosineSimilarityPartsU8,
};
#[cfg(target_arch = "x86_64")]
pub use kernels::{avx2, avx512};

use memfuse_core::{DistanceMetric, MemFuseError};

/// Validates that a vector contains no NaN or Infinite values.
#[inline]
pub fn validate_vector(vec: &[f32]) -> memfuse_core::Result<()> {
    if vec.iter().any(|v| !v.is_finite()) {
        return Err(MemFuseError::invalid_input(
            "NaN or Infinity detected in vector",
        ));
    }
    Ok(())
}

#[inline]
pub fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> memfuse_core::Result<f32> {
    validate_vector(a)?;
    validate_vector(b)?;
    compute_distance_trusted(a, b, metric)
}

#[inline]
pub fn compute_distance_trusted(
    a: &[f32],
    b: &[f32],
    metric: DistanceMetric,
) -> memfuse_core::Result<f32> {
    match metric {
        DistanceMetric::Cosine => cosine_distance(a, b),
        DistanceMetric::Euclidean => euclidean_distance(a, b),
        DistanceMetric::DotProduct => dot_product_distance(a, b),
        _ => Err(MemFuseError::invalid_input("Unsupported distance metric")),
    }
}

#[inline]
pub fn compute_distance_f32_bytes_trusted(
    a: &[f32],
    b_bytes: &[u8],
    metric: DistanceMetric,
) -> memfuse_core::Result<f32> {
    match metric {
        DistanceMetric::Cosine => cosine_distance_f32_bytes(a, b_bytes),
        DistanceMetric::Euclidean => euclidean_distance_f32_bytes(a, b_bytes),
        DistanceMetric::DotProduct => dot_product_distance_f32_bytes(a, b_bytes),
        _ => Err(MemFuseError::invalid_input("Unsupported distance metric")),
    }
}
