// FILE-CONTEXT
// ZWECK: Runtime Hardware Feature Detection & Dispatcher.
// INVARIANTEN: Zero-Panic, sicherer Fallback auf Skalar, wenn CPU-Features fehlen.

use crate::kernels::*;
use memfuse_core::MemFuseError;

#[inline]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32, MemFuseError> {
    if a.len() != b.len() {
        return Err(MemFuseError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return Ok(unsafe { avx512::cosine_distance_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            return Ok(unsafe { avx2::cosine_distance_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return Ok(unsafe { neon::cosine_distance_neon(a, b) });
        }
    }

    Ok(scalar::cosine_distance_scalar(a, b))
}

#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32, MemFuseError> {
    if a.len() != b.len() {
        return Err(MemFuseError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return Ok(unsafe { avx512::euclidean_distance_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            return Ok(unsafe { avx2::euclidean_distance_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return Ok(unsafe { neon::euclidean_distance_neon(a, b) });
        }
    }

    Ok(scalar::euclidean_distance_scalar(a, b))
}

#[inline]
pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32, MemFuseError> {
    if a.len() != b.len() {
        return Err(MemFuseError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return Ok(unsafe { avx512::dot_product_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            return Ok(unsafe { avx2::dot_product_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return Ok(unsafe { neon::dot_product_neon(a, b) });
        }
    }

    Ok(scalar::dot_product_scalar(a, b))
}

#[inline]
pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, MemFuseError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(MemFuseError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return Ok(unsafe { avx512::cosine_distance_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            return Ok(unsafe { avx2::cosine_distance_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::cosine_distance_f32_bytes_scalar(a, b_bytes))
}

#[inline]
pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, MemFuseError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(MemFuseError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return Ok(unsafe { avx512::euclidean_distance_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            return Ok(unsafe { avx2::euclidean_distance_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::euclidean_distance_f32_bytes_scalar(a, b_bytes))
}

#[inline]
pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, MemFuseError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(MemFuseError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return Ok(unsafe { avx512::dot_product_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            return Ok(unsafe { avx2::dot_product_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::dot_product_f32_bytes_scalar(a, b_bytes))
}

#[inline]
pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32, MemFuseError> {
    if a.len() != b.len() {
        return Err(MemFuseError::EmbeddingDimensionMismatch {
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
            return Ok(unsafe { avx512::dot_product_u8_avx512vnni(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            return Ok(unsafe { avx2::dot_product_u8_avx2(a, b) });
        }
    }

    Ok(scalar::dot_product_u8_scalar(a, b))
}

#[inline]
pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32, MemFuseError> {
    if a.len() != b.len() {
        return Err(MemFuseError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
            return Ok(unsafe { avx512::euclidean_distance_sq_u8_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            return Ok(unsafe { avx2::euclidean_distance_sq_u8_avx2(a, b) });
        }
    }

    Ok(scalar::euclidean_distance_sq_u8_scalar(a, b))
}

#[inline]
pub fn cosine_similarity_parts_u8(a: &[u8], b: &[u8]) -> Result<CosineSimilarityPartsU8, MemFuseError> {
    if a.len() != b.len() {
        return Err(MemFuseError::EmbeddingDimensionMismatch {
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
            return Ok(unsafe { avx512::cosine_similarity_parts_u8_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            return Ok(unsafe { avx2::cosine_similarity_parts_u8_avx2(a, b) });
        }
    }

    Ok(scalar::cosine_similarity_parts_u8_scalar(a, b))
}
