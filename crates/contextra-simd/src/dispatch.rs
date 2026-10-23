// FILE-CONTEXT
// ZWECK: Runtime Hardware Feature Detection & Dispatcher.
// INVARIANTEN: Zero-Panic, sicherer Fallback auf Skalar, wenn CPU-Features fehlen.

use crate::kernels::*;
use contextra_types::ContextraError;

/// Token representing detected CPU hardware features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuFeatures {
    pub avx512: bool,
    pub avx2: bool,
    pub fma: bool,
    pub neon: bool,
}

impl CpuFeatures {
    /// Detects supported CPU hardware acceleration features at runtime.
    pub fn detect() -> Self {
        let mut f = Self::default();
        #[cfg(target_arch = "x86_64")]
        {
            f.avx512 = is_x86_feature_detected!("avx512f");
            f.avx2 = is_x86_feature_detected!("avx2");
            f.fma = is_x86_feature_detected!("fma");
        }
        #[cfg(target_arch = "aarch64")]
        {
            f.neon = std::arch::is_aarch64_feature_detected!("neon");
        }
        f
    }
}

/// Detects active hardware features.
pub fn detect() -> CpuFeatures {
    CpuFeatures::detect()
}

/// Computes cosine distance between `a` and `b`, dispatching to SIMD hardware acceleration if available.
#[inline]
#[allow(unsafe_code)]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: Target feature `avx512f` verified via runtime check immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx512::cosine_distance_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: Target features `avx2` and `fma` verified via runtime checks immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx2::cosine_distance_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: Target feature `neon` verified via runtime check immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { neon::cosine_distance_neon(a, b) });
        }
    }

    Ok(scalar::cosine_distance_scalar(a, b))
}

pub fn cosine_distance_with_features(
    a: &[f32],
    b: &[f32],
    _features: &CpuFeatures,
) -> Result<f32, ContextraError> {
    cosine_distance(a, b)
}

/// Computes Euclidean distance between `a` and `b`, dispatching to SIMD hardware acceleration if available.
#[inline]
#[allow(unsafe_code)]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: Target feature `avx512f` verified via runtime check immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx512::euclidean_distance_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: Target features `avx2` and `fma` verified via runtime checks immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx2::euclidean_distance_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: Target feature `neon` verified via runtime check immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { neon::euclidean_distance_neon(a, b) });
        }
    }

    Ok(scalar::euclidean_distance_scalar(a, b))
}

pub fn euclidean_distance_with_features(
    a: &[f32],
    b: &[f32],
    _features: &CpuFeatures,
) -> Result<f32, ContextraError> {
    euclidean_distance(a, b)
}

/// Computes dot product distance between `a` and `b`, dispatching to SIMD hardware acceleration if available.
#[inline]
#[allow(unsafe_code)]
pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: Target feature `avx512f` verified via runtime check immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx512::dot_product_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: Target features `avx2` and `fma` verified via runtime checks immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx2::dot_product_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: Target feature `neon` verified via runtime check immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { neon::dot_product_neon(a, b) });
        }
    }

    Ok(scalar::dot_product_scalar(a, b))
}

pub fn dot_product_distance_with_features(
    a: &[f32],
    b: &[f32],
    _features: &CpuFeatures,
) -> Result<f32, ContextraError> {
    dot_product_distance(a, b)
}

/// Computes cosine distance between f32 slice `a` and raw byte slice `b_bytes`.
#[inline]
#[allow(unsafe_code)]
pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len().saturating_mul(4) {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: Target feature `avx512f` verified via runtime check immediately prior; `b_bytes.len() >= a.len() * 4` is guaranteed by check above.
            return Ok(unsafe { avx512::cosine_distance_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: Target features `avx2` and `fma` verified via runtime checks immediately prior; `b_bytes.len() >= a.len() * 4` is guaranteed by check above.
            return Ok(unsafe { avx2::cosine_distance_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::cosine_distance_f32_bytes_scalar(a, b_bytes))
}

/// Computes Euclidean distance between f32 slice `a` and raw byte slice `b_bytes`.
#[inline]
#[allow(unsafe_code)]
pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len().saturating_mul(4) {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: Target feature `avx512f` verified via runtime check immediately prior; `b_bytes.len() >= a.len() * 4` is guaranteed by check above.
            return Ok(unsafe { avx512::euclidean_distance_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: Target features `avx2` and `fma` verified via runtime checks immediately prior; `b_bytes.len() >= a.len() * 4` is guaranteed by check above.
            return Ok(unsafe { avx2::euclidean_distance_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::euclidean_distance_f32_bytes_scalar(a, b_bytes))
}

/// Computes dot product distance between f32 slice `a` and raw byte slice `b_bytes`.
#[inline]
#[allow(unsafe_code)]
pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len().saturating_mul(4) {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: Target feature `avx512f` verified via runtime check immediately prior; `b_bytes.len() >= a.len() * 4` is guaranteed by check above.
            return Ok(unsafe { avx512::dot_product_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: Target features `avx2` and `fma` verified via runtime checks immediately prior; `b_bytes.len() >= a.len() * 4` is guaranteed by check above.
            return Ok(unsafe { avx2::dot_product_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::dot_product_f32_bytes_scalar(a, b_bytes))
}

/// Computes dot product of quantized u8 vectors `a` and `b`.
#[inline]
#[allow(unsafe_code)]
pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
            // SAFETY: Target features `avx512f` and `avx512bw` verified via runtime checks immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx512::dot_product_u8_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: Target feature `avx2` verified via runtime check immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx2::dot_product_u8_avx2(a, b) });
        }
    }

    Ok(scalar::dot_product_u8_scalar(a, b))
}

pub fn dot_product_u8_with_features(
    a: &[u8],
    b: &[u8],
    _features: &CpuFeatures,
) -> Result<u32, ContextraError> {
    dot_product_u8(a, b)
}

/// Computes squared Euclidean distance of quantized u8 vectors `a` and `b`.
#[inline]
#[allow(unsafe_code)]
pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
            // SAFETY: Target features `avx512f` and `avx512bw` verified via runtime checks immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx512::euclidean_distance_sq_u8_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: Target feature `avx2` verified via runtime check immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx2::euclidean_distance_sq_u8_avx2(a, b) });
        }
    }

    Ok(scalar::euclidean_distance_sq_u8_scalar(a, b))
}

/// Computes cosine similarity component parts (dot product, norm_a_sq, norm_b_sq) for u8 vectors.
#[inline]
#[allow(unsafe_code)]
pub fn cosine_similarity_parts_u8(
    a: &[u8],
    b: &[u8],
) -> Result<scalar::CosineSimilarityPartsU8, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
            // SAFETY: Target features `avx512f` and `avx512bw` verified via runtime checks immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx512::cosine_similarity_parts_u8_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: Target feature `avx2` verified via runtime check immediately prior; `a.len() == b.len()` is guaranteed by dimension check above.
            return Ok(unsafe { avx2::cosine_similarity_parts_u8_avx2(a, b) });
        }
    }

    Ok(scalar::cosine_similarity_parts_u8_scalar(a, b))
}

pub fn cosine_similarity_parts_u8_with_features(
    a: &[u8],
    b: &[u8],
    _features: &CpuFeatures,
) -> Result<scalar::CosineSimilarityPartsU8, ContextraError> {
    cosine_similarity_parts_u8(a, b)
}
