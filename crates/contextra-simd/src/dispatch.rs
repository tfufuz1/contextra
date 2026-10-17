// FILE-CONTEXT
// ZWECK: Runtime Hardware Feature Detection & Dispatcher.
// INVARIANTEN: Zero-Panic, sicherer Fallback auf Skalar, wenn CPU-Features fehlen.

use crate::kernels::*;
use contextra_core::ContextraError;

/// Computes cosine distance between `a` and `b`, dispatching to SIMD hardware acceleration if available.
///
/// # Errors
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is `a.len()`.
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
            // SAFETY: `avx512f` feature is confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 18.
            return Ok(unsafe { avx512::cosine_distance_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: `avx2` and `fma` features are confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 18.
            return Ok(unsafe { avx2::cosine_distance_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: `neon` feature is confirmed via `std::arch::is_aarch64_feature_detected!`; `a.len() == b.len()` verified at line 18.
            return Ok(unsafe { neon::cosine_distance_neon(a, b) });
        }
    }

    Ok(scalar::cosine_distance_scalar(a, b))
}

/// Computes Euclidean distance between `a` and `b`, dispatching to SIMD hardware acceleration if available.
///
/// # Errors
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is `a.len()`.
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
            // SAFETY: `avx512f` feature is confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 55.
            return Ok(unsafe { avx512::euclidean_distance_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: `avx2` and `fma` features are confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 55.
            return Ok(unsafe { avx2::euclidean_distance_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: `neon` feature is confirmed via `std::arch::is_aarch64_feature_detected!`; `a.len() == b.len()` verified at line 55.
            return Ok(unsafe { neon::euclidean_distance_neon(a, b) });
        }
    }

    Ok(scalar::euclidean_distance_scalar(a, b))
}

/// Computes dot product distance between `a` and `b`, dispatching to SIMD hardware acceleration if available.
///
/// # Errors
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is `a.len()`.
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
            // SAFETY: `avx512f` feature is confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 92.
            return Ok(unsafe { avx512::dot_product_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: `avx2` and `fma` features are confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 92.
            return Ok(unsafe { avx2::dot_product_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: `neon` feature is confirmed via `std::arch::is_aarch64_feature_detected!`; `a.len() == b.len()` verified at line 92.
            return Ok(unsafe { neon::dot_product_neon(a, b) });
        }
    }

    Ok(scalar::dot_product_scalar(a, b))
}

/// Computes cosine distance between f32 slice `a` and raw byte slice `b_bytes`.
///
/// # Errors
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `b_bytes.len() < a.len() * 4`.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is `a.len()`.
#[inline]
#[allow(unsafe_code)]
pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: `avx512f` feature is confirmed via `is_x86_feature_detected!`; `b_bytes.len() >= a.len() * 4` verified at line 146.
            return Ok(unsafe { avx512::cosine_distance_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: `avx2` and `fma` features are confirmed via `is_x86_feature_detected!`; `b_bytes.len() >= a.len() * 4` verified at line 146.
            return Ok(unsafe { avx2::cosine_distance_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::cosine_distance_f32_bytes_scalar(a, b_bytes))
}

/// Computes Euclidean distance between f32 slice `a` and raw byte slice `b_bytes`.
///
/// # Errors
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `b_bytes.len() < a.len() * 4`.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is `a.len()`.
#[inline]
#[allow(unsafe_code)]
pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: `avx512f` feature is confirmed via `is_x86_feature_detected!`; `b_bytes.len() >= a.len() * 4` verified at line 180.
            return Ok(unsafe { avx512::euclidean_distance_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: `avx2` and `fma` features are confirmed via `is_x86_feature_detected!`; `b_bytes.len() >= a.len() * 4` verified at line 180.
            return Ok(unsafe { avx2::euclidean_distance_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::euclidean_distance_f32_bytes_scalar(a, b_bytes))
}

/// Computes dot product distance between f32 slice `a` and raw byte slice `b_bytes`.
///
/// # Errors
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `b_bytes.len() < a.len() * 4`.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is `a.len()`.
#[inline]
#[allow(unsafe_code)]
pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: `avx512f` feature is confirmed via `is_x86_feature_detected!`; `b_bytes.len() >= a.len() * 4` verified at line 214.
            return Ok(unsafe { avx512::dot_product_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: `avx2` and `fma` features are confirmed via `is_x86_feature_detected!`; `b_bytes.len() >= a.len() * 4` verified at line 214.
            return Ok(unsafe { avx2::dot_product_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::dot_product_f32_bytes_scalar(a, b_bytes))
}

/// Computes dot product of quantized u8 vectors `a` and `b`.
///
/// # Errors
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is `a.len()`.
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
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512bw")
            && is_x86_feature_detected!("avx512vnni")
        {
            // SAFETY: `avx512f`, `avx512bw`, and `avx512vnni` features are confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 248.
            return Ok(unsafe { avx512::dot_product_u8_avx512vnni(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: `avx2` feature is confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 248.
            return Ok(unsafe { avx2::dot_product_u8_avx2(a, b) });
        }
    }

    Ok(scalar::dot_product_u8_scalar(a, b))
}

/// Computes squared Euclidean distance of quantized u8 vectors `a` and `b`.
///
/// # Errors
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is `a.len()`.
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
            // SAFETY: `avx512f` and `avx512bw` features are confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 285.
            return Ok(unsafe { avx512::euclidean_distance_sq_u8_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: `avx2` feature is confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 285.
            return Ok(unsafe { avx2::euclidean_distance_sq_u8_avx2(a, b) });
        }
    }

    Ok(scalar::euclidean_distance_sq_u8_scalar(a, b))
}

/// Computes cosine similarity component parts (dot product, norm_a_sq, norm_b_sq) for u8 vectors.
///
/// # Errors
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
///
/// # Panics
/// Guarantees zero panic.
///
/// # Complexity
/// O(N) where N is `a.len()`.
#[inline]
#[allow(unsafe_code)]
pub fn cosine_similarity_parts_u8(
    a: &[u8],
    b: &[u8],
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
            // SAFETY: `avx512f`, `avx512bw`, and `avx512vnni` features are confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 322.
            return Ok(unsafe { avx512::cosine_similarity_parts_u8_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: `avx2` feature is confirmed via `is_x86_feature_detected!`; `a.len() == b.len()` verified at line 322.
            return Ok(unsafe { avx2::cosine_similarity_parts_u8_avx2(a, b) });
        }
    }

    Ok(scalar::cosine_similarity_parts_u8_scalar(a, b))
}
