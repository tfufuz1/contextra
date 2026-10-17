// FILE-CONTEXT
// ZWECK: AVX2 SIMD-Intrinsics für f32, f32_bytes und u8 Distanzberechnungen.
// INVARIANTEN: Target-Feature "avx2", "fma" garantiert vor Aufruf via Feature-Check.

#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unused_unsafe)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::kernels::scalar::CosineSimilarityPartsU8;

/// Computes the cosine distance between two `f32` slices using AVX2 and FMA instructions.
///
/// # Safety
///
/// - The target CPU must support the `avx2` and `fma` target features. The caller must verify this
///   at runtime (e.g. via `is_x86_feature_detected!("avx2")` and `is_x86_feature_detected!("fma")`).
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
/// - Memory pointers are read using unaligned vector loads (`_mm256_loadu_ps`), so 32-byte alignment
///   is not required.
/// - Slices with length 0 or remainder elements not divisible by 8 are handled safely by the
///   vector loop condition and scalar tail loop.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Zeroing AVX2 vector registers; target feature `avx2` is guaranteed by caller.
    let (mut dot_v, mut norm_a_v, mut norm_b_v) = unsafe {
        (
            _mm256_setzero_ps(),
            _mm256_setzero_ps(),
            _mm256_setzero_ps(),
        )
    };

    while i + 8 <= len {
        // SAFETY: The loop condition `i + 8 <= len` guarantees that reading 8 `f32` elements from
        // `a.as_ptr().add(i)` and `b.as_ptr().add(i)` stays within the valid bounds of `a` and `b`.
        // `_mm256_loadu_ps` supports unaligned loads. `_mm256_fmadd_ps` requires AVX2/FMA features,
        // guaranteed by the caller safety contract.
        unsafe {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b.as_ptr().add(i));

            dot_v = _mm256_fmadd_ps(va, vb, dot_v);
            norm_a_v = _mm256_fmadd_ps(va, va, norm_a_v);
            norm_b_v = _mm256_fmadd_ps(vb, vb, norm_b_v);
        }

        i += 8;
    }

    // SAFETY: `hsum256_ps_avx` requires the AVX2 target feature, guaranteed by the caller safety contract.
    let (mut dot, mut norm_a, mut norm_b) = unsafe {
        (
            hsum256_ps_avx(dot_v),
            hsum256_ps_avx(norm_a_v),
            hsum256_ps_avx(norm_b_v),
        )
    };

    while i < len {
        // SAFETY: The loop condition `i < len` guarantees that index `i` is strictly within the bounds
        // of both `a` and `b` (given `a.len() == b.len()`).
        unsafe {
            let x = *a.get_unchecked(i);
            let y = *b.get_unchecked(i);
            dot += x * y;
            norm_a += x * x;
            norm_b += y * y;
        }
        i += 1;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    let sim = (dot / denom).clamp(-1.0, 1.0);
    1.0 - sim
}

/// Computes the Euclidean distance between two `f32` slices using AVX2 and FMA instructions.
///
/// # Safety
///
/// - The target CPU must support the `avx2` and `fma` target features. The caller must verify this
///   at runtime (e.g. via `is_x86_feature_detected!("avx2")` and `is_x86_feature_detected!("fma")`).
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
/// - Memory pointers are read using unaligned vector loads (`_mm256_loadu_ps`), so 32-byte alignment
///   is not required.
/// - Slices with length 0 or remainder elements not divisible by 8 are handled safely by the
///   vector loop condition and scalar tail loop.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Zeroing AVX2 vector register; target feature `avx2` is guaranteed by caller.
    let mut sum_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: The loop condition `i + 8 <= len` guarantees that reading 8 `f32` elements from
        // `a.as_ptr().add(i)` and `b.as_ptr().add(i)` stays within the valid bounds of `a` and `b`.
        // `_mm256_loadu_ps` supports unaligned loads. `_mm256_sub_ps` and `_mm256_fmadd_ps` require
        // AVX2/FMA features, guaranteed by the caller safety contract.
        unsafe {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b.as_ptr().add(i));
            let diff = _mm256_sub_ps(va, vb);

            sum_v = _mm256_fmadd_ps(diff, diff, sum_v);
        }

        i += 8;
    }

    // SAFETY: `hsum256_ps_avx` requires the AVX2 target feature, guaranteed by the caller safety contract.
    let mut sum = unsafe { hsum256_ps_avx(sum_v) };

    while i < len {
        // SAFETY: The loop condition `i < len` guarantees that index `i` is strictly within the bounds
        // of both `a` and `b` (given `a.len() == b.len()`).
        let diff = unsafe { *a.get_unchecked(i) - *b.get_unchecked(i) };
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Computes the negative dot product between two `f32` slices using AVX2 and FMA instructions.
///
/// # Safety
///
/// - The target CPU must support the `avx2` and `fma` target features. The caller must verify this
///   at runtime (e.g. via `is_x86_feature_detected!("avx2")` and `is_x86_feature_detected!("fma")`).
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
/// - Memory pointers are read using unaligned vector loads (`_mm256_loadu_ps`), so 32-byte alignment
///   is not required.
/// - Slices with length 0 or remainder elements not divisible by 8 are handled safely by the
///   vector loop condition and scalar tail loop.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Zeroing AVX2 vector register; target feature `avx2` is guaranteed by caller.
    let mut dot_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: The loop condition `i + 8 <= len` guarantees that reading 8 `f32` elements from
        // `a.as_ptr().add(i)` and `b.as_ptr().add(i)` stays within the valid bounds of `a` and `b`.
        // `_mm256_loadu_ps` supports unaligned loads. `_mm256_fmadd_ps` requires AVX2/FMA features,
        // guaranteed by the caller safety contract.
        unsafe {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b.as_ptr().add(i));

            dot_v = _mm256_fmadd_ps(va, vb, dot_v);
        }

        i += 8;
    }

    // SAFETY: `hsum256_ps_avx` requires the AVX2 target feature, guaranteed by the caller safety contract.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };

    while i < len {
        // SAFETY: The loop condition `i < len` guarantees that index `i` is strictly within the bounds
        // of both `a` and `b` (given `a.len() == b.len()`).
        dot += unsafe { *a.get_unchecked(i) * *b.get_unchecked(i) };
        i += 1;
    }

    -dot
}

/// Computes the cosine distance between an `f32` slice `a` and little-endian `u8` encoded `f32` slice `b_bytes` using AVX2 and FMA instructions.
///
/// # Safety
///
/// - The target CPU must support the `avx2` and `fma` target features. The caller must verify this
///   at runtime (e.g. via `is_x86_feature_detected!("avx2")` and `is_x86_feature_detected!("fma")`).
/// - Slice `b_bytes` must contain at least `a.len() * 4` bytes.
/// - Memory pointers are read using unaligned vector loads (`_mm256_loadu_ps`), so 32-byte alignment
///   is not required.
/// - Slices with length 0 or remainder elements not divisible by 8 are handled safely by the
///   vector loop condition and scalar tail loop.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn cosine_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Zeroing AVX2 vector registers; target feature `avx2` is guaranteed by caller.
    let (mut dot_v, mut norm_a_v, mut norm_b_v) = unsafe {
        (
            _mm256_setzero_ps(),
            _mm256_setzero_ps(),
            _mm256_setzero_ps(),
        )
    };

    while i + 8 <= len {
        // SAFETY: The loop condition `i + 8 <= len` guarantees `i * 4 + 31 < b_bytes.len()` (32 bytes = 8 f32s),
        // making reads from `a.as_ptr().add(i)` and `b_bytes.as_ptr().add(i * 4)` in-bounds.
        // `_mm256_loadu_ps` handles unaligned reads. `_mm256_fmadd_ps` requires AVX2/FMA features,
        // guaranteed by the caller safety contract.
        unsafe {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32);

            dot_v = _mm256_fmadd_ps(va, vb, dot_v);
            norm_a_v = _mm256_fmadd_ps(va, va, norm_a_v);
            norm_b_v = _mm256_fmadd_ps(vb, vb, norm_b_v);
        }

        i += 8;
    }

    // SAFETY: `hsum256_ps_avx` requires the AVX2 target feature, guaranteed by the caller safety contract.
    let (mut dot, mut norm_a, mut norm_b) = unsafe {
        (
            hsum256_ps_avx(dot_v),
            hsum256_ps_avx(norm_a_v),
            hsum256_ps_avx(norm_b_v),
        )
    };

    while i < len {
        // SAFETY: The loop condition `i < len` guarantees that `i` is within bounds of `a` and
        // `i * 4 + 3 < b_bytes.len()` (given `b_bytes.len() >= a.len() * 4`).
        unsafe {
            let x = *a.get_unchecked(i);
            let b_val = f32::from_le_bytes([
                *b_bytes.get_unchecked(i * 4),
                *b_bytes.get_unchecked(i * 4 + 1),
                *b_bytes.get_unchecked(i * 4 + 2),
                *b_bytes.get_unchecked(i * 4 + 3),
            ]);
            dot += x * b_val;
            norm_a += x * x;
            norm_b += b_val * b_val;
        }
        i += 1;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    let sim = (dot / denom).clamp(-1.0, 1.0);
    1.0 - sim
}

/// Computes the Euclidean distance between an `f32` slice `a` and little-endian `u8` encoded `f32` slice `b_bytes` using AVX2 and FMA instructions.
///
/// # Safety
///
/// - The target CPU must support the `avx2` and `fma` target features. The caller must verify this
///   at runtime (e.g. via `is_x86_feature_detected!("avx2")` and `is_x86_feature_detected!("fma")`).
/// - Slice `b_bytes` must contain at least `a.len() * 4` bytes.
/// - Memory pointers are read using unaligned vector loads (`_mm256_loadu_ps`), so 32-byte alignment
///   is not required.
/// - Slices with length 0 or remainder elements not divisible by 8 are handled safely by the
///   vector loop condition and scalar tail loop.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn euclidean_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Zeroing AVX2 vector register; target feature `avx2` is guaranteed by caller.
    let mut sum_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: The loop condition `i + 8 <= len` guarantees `i * 4 + 31 < b_bytes.len()` (32 bytes = 8 f32s),
        // making reads from `a.as_ptr().add(i)` and `b_bytes.as_ptr().add(i * 4)` in-bounds.
        // `_mm256_loadu_ps` handles unaligned reads. `_mm256_sub_ps` and `_mm256_fmadd_ps` require
        // AVX2/FMA features, guaranteed by the caller safety contract.
        unsafe {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32);
            let diff = _mm256_sub_ps(va, vb);

            sum_v = _mm256_fmadd_ps(diff, diff, sum_v);
        }

        i += 8;
    }

    // SAFETY: `hsum256_ps_avx` requires the AVX2 target feature, guaranteed by the caller safety contract.
    let mut sum = unsafe { hsum256_ps_avx(sum_v) };

    while i < len {
        // SAFETY: The loop condition `i < len` guarantees that `i` is within bounds of `a` and
        // `i * 4 + 3 < b_bytes.len()` (given `b_bytes.len() >= a.len() * 4`).
        unsafe {
            let b_val = f32::from_le_bytes([
                *b_bytes.get_unchecked(i * 4),
                *b_bytes.get_unchecked(i * 4 + 1),
                *b_bytes.get_unchecked(i * 4 + 2),
                *b_bytes.get_unchecked(i * 4 + 3),
            ]);
            let diff = *a.get_unchecked(i) - b_val;
            sum += diff * diff;
        }
        i += 1;
    }

    sum.sqrt()
}

/// Computes the negative dot product between an `f32` slice `a` and little-endian `u8` encoded `f32` slice `b_bytes` using AVX2 and FMA instructions.
///
/// # Safety
///
/// - The target CPU must support the `avx2` and `fma` target features. The caller must verify this
///   at runtime (e.g. via `is_x86_feature_detected!("avx2")` and `is_x86_feature_detected!("fma")`).
/// - Slice `b_bytes` must contain at least `a.len() * 4` bytes.
/// - Memory pointers are read using unaligned vector loads (`_mm256_loadu_ps`), so 32-byte alignment
///   is not required.
/// - Slices with length 0 or remainder elements not divisible by 8 are handled safely by the
///   vector loop condition and scalar tail loop.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn dot_product_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Zeroing AVX2 vector register; target feature `avx2` is guaranteed by caller.
    let mut dot_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: The loop condition `i + 8 <= len` guarantees `i * 4 + 31 < b_bytes.len()` (32 bytes = 8 f32s),
        // making reads from `a.as_ptr().add(i)` and `b_bytes.as_ptr().add(i * 4)` in-bounds.
        // `_mm256_loadu_ps` handles unaligned reads. `_mm256_fmadd_ps` requires AVX2/FMA features,
        // guaranteed by the caller safety contract.
        unsafe {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32);

            dot_v = _mm256_fmadd_ps(va, vb, dot_v);
        }

        i += 8;
    }

    // SAFETY: `hsum256_ps_avx` requires the AVX2 target feature, guaranteed by the caller safety contract.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };

    while i < len {
        // SAFETY: The loop condition `i < len` guarantees that `i` is within bounds of `a` and
        // `i * 4 + 3 < b_bytes.len()` (given `b_bytes.len() >= a.len() * 4`).
        unsafe {
            let b_val = f32::from_le_bytes([
                *b_bytes.get_unchecked(i * 4),
                *b_bytes.get_unchecked(i * 4 + 1),
                *b_bytes.get_unchecked(i * 4 + 2),
                *b_bytes.get_unchecked(i * 4 + 3),
            ]);
            dot += *a.get_unchecked(i) * b_val;
        }
        i += 1;
    }

    -dot
}

/// Horizontally sums all 8 `f32` lanes of a 256-bit AVX register `v`.
///
/// # Safety
///
/// - The target CPU must support the `avx2` target feature. The caller must verify this at
///   runtime (e.g. via `is_x86_feature_detected!("avx2")`).
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub unsafe fn hsum256_ps_avx(v: __m256) -> f32 {
    // SAFETY: All operations use valid AVX2 register intrinsics on input `v`; target feature `avx2` verified by caller.
    unsafe {
        let vlow = _mm256_castps256_ps128(v);
        let vhigh = _mm256_extractf128_ps(v, 1);
        let v128 = _mm_add_ps(vlow, vhigh);
        let v64 = _mm_add_ps(v128, _mm_movehl_ps(v128, v128));
        let v32 = _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55));
        _mm_cvtss_f32(v32)
    }
}

/// Computes the dot product between two `u8` slices using AVX2 vector operations.
///
/// # Safety
///
/// - The target CPU must support the `avx2` target feature. The caller must verify this at
///   runtime (e.g. via `is_x86_feature_detected!("avx2")`).
/// - Slices `a` and `b` must have equal or valid lengths (iteration is bounded by `min(a.len(), b.len())`).
/// - Memory pointers are read using unaligned vector loads (`_mm256_loadu_si256`), so 32-byte alignment
///   is not required.
/// - Slices with length 0 or remainder elements not divisible by 32 are handled safely by the
///   vector loop condition and scalar tail loop.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub unsafe fn dot_product_u8_avx2(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Zeroing AVX2 vector registers; target feature `avx2` is guaranteed by caller.
    let (mut sum_v, zero) = unsafe { (_mm256_setzero_si256(), _mm256_setzero_si256()) };

    while i + 32 <= len {
        // SAFETY: The loop condition `i + 32 <= len` guarantees reading 32 `u8` elements from
        // `a.as_ptr().add(i)` and `b.as_ptr().add(i)` stays within bounds of `a` and `b`.
        // `_mm256_loadu_si256` supports unaligned loads. All vector instructions (`_mm256_unpacklo_epi8`,
        // `_mm256_madd_epi16`, `_mm256_unpackhi_epi8`, `_mm256_add_epi32`) require the AVX2 target feature,
        // guaranteed by the caller contract.
        unsafe {
            let va = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
            let vb = _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i);

            let va_lo = _mm256_unpacklo_epi8(va, zero);
            let vb_lo = _mm256_unpacklo_epi8(vb, zero);
            let prod_lo = _mm256_madd_epi16(va_lo, vb_lo);

            let va_hi = _mm256_unpackhi_epi8(va, zero);
            let vb_hi = _mm256_unpackhi_epi8(vb, zero);
            let prod_hi = _mm256_madd_epi16(va_hi, vb_hi);

            sum_v = _mm256_add_epi32(sum_v, prod_lo);
            sum_v = _mm256_add_epi32(sum_v, prod_hi);
        }

        i += 32;
    }

    // SAFETY: `hsum256_epi32_avx2` requires the AVX2 target feature, guaranteed by the caller safety contract.
    let mut sum = unsafe { hsum256_epi32_avx2(sum_v) } as u32;

    while i < len {
        // SAFETY: The loop condition `i < len` guarantees that index `i` is strictly within bounds of
        // both `a` and `b` (given `len = min(a.len(), b.len())`).
        sum += unsafe { (*a.get_unchecked(i) as u32) * (*b.get_unchecked(i) as u32) };
        i += 1;
    }

    sum
}

/// Computes the squared Euclidean distance between two `u8` slices using AVX2 vector operations.
///
/// # Safety
///
/// - The target CPU must support the `avx2` target feature. The caller must verify this at
///   runtime (e.g. via `is_x86_feature_detected!("avx2")`).
/// - Slices `a` and `b` must have equal or valid lengths (iteration is bounded by `min(a.len(), b.len())`).
/// - Memory pointers are read using unaligned vector loads (`_mm256_loadu_si256`), so 32-byte alignment
///   is not required.
/// - Slices with length 0 or remainder elements not divisible by 32 are handled safely by the
///   vector loop condition and scalar tail loop.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub unsafe fn euclidean_distance_sq_u8_avx2(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Zeroing AVX2 vector registers; target feature `avx2` is guaranteed by caller.
    let (mut sum_v, zero) = unsafe { (_mm256_setzero_si256(), _mm256_setzero_si256()) };

    while i + 32 <= len {
        // SAFETY: The loop condition `i + 32 <= len` guarantees reading 32 `u8` elements from
        // `a.as_ptr().add(i)` and `b.as_ptr().add(i)` stays within bounds of `a` and `b`.
        // `_mm256_loadu_si256` supports unaligned loads. All vector instructions (`_mm256_unpacklo_epi8`,
        // `_mm256_sub_epi16`, `_mm256_madd_epi16`, `_mm256_unpackhi_epi8`, `_mm256_add_epi32`) require the
        // AVX2 target feature, guaranteed by the caller contract.
        unsafe {
            let va = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
            let vb = _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i);

            let va_lo = _mm256_unpacklo_epi8(va, zero);
            let vb_lo = _mm256_unpacklo_epi8(vb, zero);
            let diff_lo = _mm256_sub_epi16(va_lo, vb_lo);
            let prod_lo = _mm256_madd_epi16(diff_lo, diff_lo);

            let va_hi = _mm256_unpackhi_epi8(va, zero);
            let vb_hi = _mm256_unpackhi_epi8(vb, zero);
            let diff_hi = _mm256_sub_epi16(va_hi, vb_hi);
            let prod_hi = _mm256_madd_epi16(diff_hi, diff_hi);

            sum_v = _mm256_add_epi32(sum_v, prod_lo);
            sum_v = _mm256_add_epi32(sum_v, prod_hi);
        }

        i += 32;
    }

    // SAFETY: `hsum256_epi32_avx2` requires the AVX2 target feature, guaranteed by the caller safety contract.
    let mut sum = unsafe { hsum256_epi32_avx2(sum_v) } as u32;

    while i < len {
        // SAFETY: The loop condition `i < len` guarantees that index `i` is strictly within bounds of
        // both `a` and `b` (given `len = min(a.len(), b.len())`).
        let diff = unsafe { (*a.get_unchecked(i) as i32) - (*b.get_unchecked(i) as i32) };
        sum += (diff * diff) as u32;
        i += 1;
    }

    sum
}

/// Computes dot product and squared norms for two `u8` slices using AVX2 vector operations.
///
/// # Safety
///
/// - The target CPU must support the `avx2` target feature. The caller must verify this at
///   runtime (e.g. via `is_x86_feature_detected!("avx2")`).
/// - Slices `a` and `b` must have equal or valid lengths (iteration is bounded by `min(a.len(), b.len())`).
/// - Memory pointers are read using unaligned vector loads (`_mm256_loadu_si256`), so 32-byte alignment
///   is not required.
/// - Slices with length 0 or remainder elements not divisible by 32 are handled safely by the
///   vector loop condition and scalar tail loop.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub unsafe fn cosine_similarity_parts_u8_avx2(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Zeroing AVX2 vector registers; target feature `avx2` is guaranteed by caller.
    let (mut dot_v, mut norm_a_v, mut norm_b_v, zero) = unsafe {
        (
            _mm256_setzero_si256(),
            _mm256_setzero_si256(),
            _mm256_setzero_si256(),
            _mm256_setzero_si256(),
        )
    };

    while i + 32 <= len {
        // SAFETY: The loop condition `i + 32 <= len` guarantees reading 32 `u8` elements from
        // `a.as_ptr().add(i)` and `b.as_ptr().add(i)` stays within bounds of `a` and `b`.
        // `_mm256_loadu_si256` supports unaligned loads. All vector instructions (`_mm256_unpacklo_epi8`,
        // `_mm256_madd_epi16`, `_mm256_add_epi32`, `_mm256_unpackhi_epi8`) require the AVX2 target feature,
        // guaranteed by the caller contract.
        unsafe {
            let va = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
            let vb = _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i);

            let va_lo = _mm256_unpacklo_epi8(va, zero);
            let vb_lo = _mm256_unpacklo_epi8(vb, zero);
            dot_v = _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_lo, vb_lo));
            norm_a_v = _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_lo, va_lo));
            norm_b_v = _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_lo, vb_lo));

            let va_hi = _mm256_unpackhi_epi8(va, zero);
            let vb_hi = _mm256_unpackhi_epi8(vb, zero);
            dot_v = _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_hi, vb_hi));
            norm_a_v = _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_hi, va_hi));
            norm_b_v = _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_hi, vb_hi));
        }

        i += 32;
    }

    // SAFETY: `hsum256_epi32_avx2` requires the AVX2 target feature, guaranteed by the caller safety contract.
    let (mut dot, mut norm_a_sq, mut norm_b_sq) = unsafe {
        (
            hsum256_epi32_avx2(dot_v) as u32,
            hsum256_epi32_avx2(norm_a_v) as u32,
            hsum256_epi32_avx2(norm_b_v) as u32,
        )
    };

    while i < len {
        // SAFETY: The loop condition `i < len` guarantees that index `i` is strictly within bounds of
        // both `a` and `b` (given `len = min(a.len(), b.len())`).
        unsafe {
            let x = *a.get_unchecked(i) as u32;
            let y = *b.get_unchecked(i) as u32;
            dot += x * y;
            norm_a_sq += x * x;
            norm_b_sq += y * y;
        }
        i += 1;
    }

    CosineSimilarityPartsU8 {
        dot,
        norm_a_sq,
        norm_b_sq,
    }
}

/// Horizontally sums all 8 `i32` lanes of a 256-bit AVX2 register `v`.
///
/// # Safety
///
/// - The target CPU must support the `avx2` target feature. The caller must verify this at
///   runtime (e.g. via `is_x86_feature_detected!("avx2")`).
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub unsafe fn hsum256_epi32_avx2(v: __m256i) -> i32 {
    // SAFETY: All operations use valid AVX2 register intrinsics on input `v`; target feature `avx2` verified by caller.
    unsafe {
        let vlow = _mm256_castsi256_si128(v);
        let vhigh = _mm256_extracti128_si256(v, 1);
        let v128 = _mm_add_epi32(vlow, vhigh);
        let v64 = _mm_add_epi32(v128, _mm_srli_si128(v128, 8));
        let v32 = _mm_add_epi32(v64, _mm_srli_si128(v64, 4));
        _mm_cvtsi128_si32(v32)
    }
}
