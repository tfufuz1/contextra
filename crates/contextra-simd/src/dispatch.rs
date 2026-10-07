// FILE-CONTEXT
// ZWECK: Runtime Hardware Feature Detection & Dispatcher.
// INVARIANTEN: Zero-Panic, sicherer Fallback auf Skalar, wenn CPU-Features fehlen.

#![allow(unsafe_code)]

use crate::kernels::*;
use contextra_core::ContextraError;

/// Opaque CPU feature token required to execute SIMD kernels.
/// Private constructor enforces that instances can only be instantiated via [`CpuFeatures::detect`].
///
/// ```compile_fail
/// use contextra_simd::CpuFeatures;
/// let _f = CpuFeatures {}; // Compile error: CpuFeatures has private fields
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    _private: (),
}

impl CpuFeatures {
    /// Detects runtime CPU capabilities using architecture feature detection.
    #[inline]
    pub fn detect() -> Self {
        Self { _private: () }
    }
}

/// Standalone detection function for hardware CPU feature tokens.
#[inline]
pub fn detect() -> CpuFeatures {
    CpuFeatures::detect()
}

#[inline]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    cosine_distance_with_features(a, b, &CpuFeatures::detect())
}

#[inline]
pub fn cosine_distance_with_features(
    a: &[f32],
    b: &[f32],
    _features: &CpuFeatures,
) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: CPU feature "avx512f" verified above; a and b have equal lengths.
            return Ok(unsafe { avx512::cosine_distance_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: CPU features "avx2" and "fma" verified above; a and b have equal lengths.
            return Ok(unsafe { avx2::cosine_distance_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: CPU feature "neon" verified above; a and b have equal lengths.
            return Ok(unsafe { neon::cosine_distance_neon(a, b) });
        }
    }

    Ok(scalar::cosine_distance_scalar(a, b))
}

#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    euclidean_distance_with_features(a, b, &CpuFeatures::detect())
}

#[inline]
pub fn euclidean_distance_with_features(
    a: &[f32],
    b: &[f32],
    _features: &CpuFeatures,
) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: CPU feature "avx512f" verified above; a and b have equal lengths.
            return Ok(unsafe { avx512::euclidean_distance_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: CPU features "avx2" and "fma" verified above; a and b have equal lengths.
            return Ok(unsafe { avx2::euclidean_distance_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: CPU feature "neon" verified above; a and b have equal lengths.
            return Ok(unsafe { neon::euclidean_distance_neon(a, b) });
        }
    }

    Ok(scalar::euclidean_distance_scalar(a, b))
}

#[inline]
pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    dot_product_distance_with_features(a, b, &CpuFeatures::detect())
}

#[inline]
pub fn dot_product_distance_with_features(
    a: &[f32],
    b: &[f32],
    _features: &CpuFeatures,
) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: CPU feature "avx512f" verified above; a and b have equal lengths.
            return Ok(unsafe { avx512::dot_product_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: CPU features "avx2" and "fma" verified above; a and b have equal lengths.
            return Ok(unsafe { avx2::dot_product_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: CPU feature "neon" verified above; a and b have equal lengths.
            return Ok(unsafe { neon::dot_product_neon(a, b) });
        }
    }

    Ok(scalar::dot_product_scalar(a, b))
}

#[inline]
pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    cosine_distance_f32_bytes_with_features(a, b_bytes, &CpuFeatures::detect())
}

#[inline]
pub fn cosine_distance_f32_bytes_with_features(
    a: &[f32],
    b_bytes: &[u8],
    _features: &CpuFeatures,
) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: CPU feature "avx512f" verified above; b_bytes has length at least a.len() * 4 bytes.
            return Ok(unsafe { avx512::cosine_distance_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: CPU features "avx2" and "fma" verified above; b_bytes has length at least a.len() * 4 bytes.
            return Ok(unsafe { avx2::cosine_distance_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::cosine_distance_f32_bytes_scalar(a, b_bytes))
}

#[inline]
pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    euclidean_distance_f32_bytes_with_features(a, b_bytes, &CpuFeatures::detect())
}

#[inline]
pub fn euclidean_distance_f32_bytes_with_features(
    a: &[f32],
    b_bytes: &[u8],
    _features: &CpuFeatures,
) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: CPU feature "avx512f" verified above; b_bytes has length at least a.len() * 4 bytes.
            return Ok(unsafe { avx512::euclidean_distance_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: CPU features "avx2" and "fma" verified above; b_bytes has length at least a.len() * 4 bytes.
            return Ok(unsafe { avx2::euclidean_distance_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::euclidean_distance_f32_bytes_scalar(a, b_bytes))
}

#[inline]
pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    dot_product_distance_f32_bytes_with_features(a, b_bytes, &CpuFeatures::detect())
}

#[inline]
pub fn dot_product_distance_f32_bytes_with_features(
    a: &[f32],
    b_bytes: &[u8],
    _features: &CpuFeatures,
) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: CPU feature "avx512f" verified above; b_bytes has length at least a.len() * 4 bytes.
            return Ok(unsafe { avx512::dot_product_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: CPU features "avx2" and "fma" verified above; b_bytes has length at least a.len() * 4 bytes.
            return Ok(unsafe { avx2::dot_product_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::dot_product_f32_bytes_scalar(a, b_bytes))
}

#[inline]
pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError> {
    dot_product_u8_with_features(a, b, &CpuFeatures::detect())
}

#[inline]
pub fn dot_product_u8_with_features(
    a: &[u8],
    b: &[u8],
    _features: &CpuFeatures,
) -> Result<u32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512bw")
            && is_x86_feature_detected!("avx512vnni")
        {
            // SAFETY: CPU features "avx512f", "avx512bw", and "avx512vnni" verified above; a and b have equal lengths.
            return Ok(unsafe { avx512::dot_product_u8_avx512vnni(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: CPU feature "avx2" verified above; a and b have equal lengths.
            return Ok(unsafe { avx2::dot_product_u8_avx2(a, b) });
        }
    }

    Ok(scalar::dot_product_u8_scalar(a, b))
}

#[inline]
pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError> {
    euclidean_distance_sq_u8_with_features(a, b, &CpuFeatures::detect())
}

#[inline]
pub fn euclidean_distance_sq_u8_with_features(
    a: &[u8],
    b: &[u8],
    _features: &CpuFeatures,
) -> Result<u32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
            // SAFETY: CPU features "avx512f" and "avx512bw" verified above; a and b have equal lengths.
            return Ok(unsafe { avx512::euclidean_distance_sq_u8_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: CPU feature "avx2" verified above; a and b have equal lengths.
            return Ok(unsafe { avx2::euclidean_distance_sq_u8_avx2(a, b) });
        }
    }

    Ok(scalar::euclidean_distance_sq_u8_scalar(a, b))
}

#[inline]
pub fn cosine_similarity_parts_u8(
    a: &[u8],
    b: &[u8],
) -> Result<CosineSimilarityPartsU8, ContextraError> {
    cosine_similarity_parts_u8_with_features(a, b, &CpuFeatures::detect())
}

#[inline]
pub fn cosine_similarity_parts_u8_with_features(
    a: &[u8],
    b: &[u8],
    _features: &CpuFeatures,
) -> Result<CosineSimilarityPartsU8, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512bw")
            && is_x86_feature_detected!("avx512vnni")
        {
            // SAFETY: CPU features "avx512f", "avx512bw", and "avx512vnni" verified above; a and b have equal lengths.
            return Ok(unsafe { avx512::cosine_similarity_parts_u8_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: CPU feature "avx2" verified above; a and b have equal lengths.
            return Ok(unsafe { avx2::cosine_similarity_parts_u8_avx2(a, b) });
        }
    }

    Ok(scalar::cosine_similarity_parts_u8_scalar(a, b))
}
