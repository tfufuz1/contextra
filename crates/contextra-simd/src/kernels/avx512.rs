// FILE-CONTEXT
// ZWECK: AVX-512 SIMD-Intrinsics für f32, f32_bytes und u8 VNNI Distanzberechnungen.
// INVARIANTEN: Target-Features guaranteed by caller or feature-checks.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::kernels::scalar::CosineSimilarityPartsU8;

/// Computes the cosine distance between two `f32` slices using AVX-512 instructions.
///
/// # Safety
///
/// - **CPU Feature Guarantee:** The caller must ensure that the target CPU supports `avx512f`.
/// - **Slice Lengths:** Slices `a` and `b` must have equal length (`a.len() == b.len()`).
///   Providing unequal slice lengths will result in out-of-bounds pointer reads.
/// - **Memory Alignment:** Input slice pointers do not require 64-byte alignment; unaligned SIMD
///   loads (`_mm512_loadu_ps`) are used.
/// - **Length / Tail Handling:** Valid for any slice length including 0. Tail elements (< 16) are processed safely via scalar fallbacks.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn cosine_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let (mut dot_v, mut norm_a_v, mut norm_b_v) = (
        _mm512_setzero_ps(),
        _mm512_setzero_ps(),
        _mm512_setzero_ps(),
    );

    while i + 16 <= len {
        // SAFETY: `i + 16 <= len` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)` dereference
        // 16 contiguous valid `f32` elements (64 bytes). `_mm512_loadu_ps` supports unaligned pointers.
        // Target feature `avx512f` is guaranteed by caller contract.
        unsafe {
            let va = _mm512_loadu_ps(a.as_ptr().add(i));
            let vb = _mm512_loadu_ps(b.as_ptr().add(i));

            dot_v = _mm512_fmadd_ps(va, vb, dot_v);
            norm_a_v = _mm512_fmadd_ps(va, va, norm_a_v);
            norm_b_v = _mm512_fmadd_ps(vb, vb, norm_b_v);
        }

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is guaranteed by caller contract. Operates purely on SIMD registers.
    let (mut dot, mut norm_a, mut norm_b) = unsafe {
        (
            hsum512_ps_avx(dot_v),
            hsum512_ps_avx(norm_a_v),
            hsum512_ps_avx(norm_b_v),
        )
    };

    while i < len {
        // SAFETY: `i < len` guarantees `i` is within bounds of slices `a` and `b` (given caller contract `a.len() == b.len()`).
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

/// Computes the Euclidean distance between two `f32` slices using AVX-512 instructions.
///
/// # Safety
///
/// - **CPU Feature Guarantee:** The caller must ensure that the target CPU supports `avx512f`.
/// - **Slice Lengths:** Slices `a` and `b` must have equal length (`a.len() == b.len()`).
///   Providing unequal slice lengths will result in out-of-bounds pointer reads.
/// - **Memory Alignment:** Input slice pointers do not require 64-byte alignment; unaligned SIMD
///   loads (`_mm512_loadu_ps`) are used.
/// - **Length / Tail Handling:** Valid for any slice length including 0. Tail elements (< 16) are processed safely via scalar fallbacks.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut sum_v = _mm512_setzero_ps();

    while i + 16 <= len {
        // SAFETY: `i + 16 <= len` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)` dereference
        // 16 contiguous valid `f32` elements (64 bytes). `_mm512_loadu_ps` supports unaligned pointers.
        // Target feature `avx512f` is guaranteed by caller contract.
        unsafe {
            let va = _mm512_loadu_ps(a.as_ptr().add(i));
            let vb = _mm512_loadu_ps(b.as_ptr().add(i));
            let diff = _mm512_sub_ps(va, vb);

            sum_v = _mm512_fmadd_ps(diff, diff, sum_v);
        }

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is guaranteed by caller contract. Operates purely on SIMD registers.
    let mut sum = unsafe { hsum512_ps_avx(sum_v) };

    while i < len {
        // SAFETY: `i < len` guarantees `i` is within bounds of slices `a` and `b` (given caller contract `a.len() == b.len()`).
        let diff = unsafe { *a.get_unchecked(i) - *b.get_unchecked(i) };
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Computes the dot product distance (negated dot product) between two `f32` slices using AVX-512 instructions.
///
/// # Safety
///
/// - **CPU Feature Guarantee:** The caller must ensure that the target CPU supports `avx512f`.
/// - **Slice Lengths:** Slices `a` and `b` must have equal length (`a.len() == b.len()`).
///   Providing unequal slice lengths will result in out-of-bounds pointer reads.
/// - **Memory Alignment:** Input slice pointers do not require 64-byte alignment; unaligned SIMD
///   loads (`_mm512_loadu_ps`) are used.
/// - **Length / Tail Handling:** Valid for any slice length including 0. Tail elements (< 16) are processed safely via scalar fallbacks.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn dot_product_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = _mm512_setzero_ps();

    while i + 16 <= len {
        // SAFETY: `i + 16 <= len` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)` dereference
        // 16 contiguous valid `f32` elements (64 bytes). `_mm512_loadu_ps` supports unaligned pointers.
        // Target feature `avx512f` is guaranteed by caller contract.
        unsafe {
            let va = _mm512_loadu_ps(a.as_ptr().add(i));
            let vb = _mm512_loadu_ps(b.as_ptr().add(i));

            dot_v = _mm512_fmadd_ps(va, vb, dot_v);
        }

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is guaranteed by caller contract. Operates purely on SIMD registers.
    let mut dot = unsafe { hsum512_ps_avx(dot_v) };

    while i < len {
        // SAFETY: `i < len` guarantees `i` is within bounds of slices `a` and `b` (given caller contract `a.len() == b.len()`).
        dot += unsafe { *a.get_unchecked(i) * *b.get_unchecked(i) };
        i += 1;
    }

    -dot
}

/// Computes the cosine distance between an `f32` slice and a little-endian `u8` byte slice representing `f32` values using AVX-512.
///
/// # Safety
///
/// - **CPU Feature Guarantee:** The caller must ensure that the target CPU supports `avx512f`.
/// - **Slice Lengths:** `b_bytes` must contain at least `a.len() * 4` bytes.
///   Providing a shorter byte slice will result in out-of-bounds pointer reads.
/// - **Memory Alignment:** Slice pointers do not require 64-byte alignment; unaligned SIMD
///   loads (`_mm512_loadu_ps`) are used for both slices.
/// - **Length / Tail Handling:** Valid for any length including 0. Tail elements (< 16) are processed safely via scalar fallbacks.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn cosine_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let (mut dot_v, mut norm_a_v, mut norm_b_v) = (
        _mm512_setzero_ps(),
        _mm512_setzero_ps(),
        _mm512_setzero_ps(),
    );

    while i + 16 <= len {
        // SAFETY: `i + 16 <= len` guarantees `a.as_ptr().add(i)` dereferences 16 valid `f32` elements (64 bytes)
        // and `b_bytes.as_ptr().add(i * 4)` dereferences 64 valid bytes (16 `f32`s).
        // `_mm512_loadu_ps` permits unaligned pointers. Target feature `avx512f` is guaranteed by caller contract.
        unsafe {
            let va = _mm512_loadu_ps(a.as_ptr().add(i));
            let vb = _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32);

            dot_v = _mm512_fmadd_ps(va, vb, dot_v);
            norm_a_v = _mm512_fmadd_ps(va, va, norm_a_v);
            norm_b_v = _mm512_fmadd_ps(vb, vb, norm_b_v);
        }

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is guaranteed by caller contract. Operates purely on SIMD registers.
    let (mut dot, mut norm_a, mut norm_b) = unsafe {
        (
            hsum512_ps_avx(dot_v),
            hsum512_ps_avx(norm_a_v),
            hsum512_ps_avx(norm_b_v),
        )
    };

    while i < len {
        // SAFETY: `i < len` guarantees `i` is in bounds for `a` and `i * 4 + 3 < b_bytes.len()` (given caller contract `b_bytes.len() >= a.len() * 4`).
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

/// Computes the Euclidean distance between an `f32` slice and a little-endian `u8` byte slice representing `f32` values using AVX-512.
///
/// # Safety
///
/// - **CPU Feature Guarantee:** The caller must ensure that the target CPU supports `avx512f`.
/// - **Slice Lengths:** `b_bytes` must contain at least `a.len() * 4` bytes.
///   Providing a shorter byte slice will result in out-of-bounds pointer reads.
/// - **Memory Alignment:** Slice pointers do not require 64-byte alignment; unaligned SIMD
///   loads (`_mm512_loadu_ps`) are used for both slices.
/// - **Length / Tail Handling:** Valid for any length including 0. Tail elements (< 16) are processed safely via scalar fallbacks.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn euclidean_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut sum_v = _mm512_setzero_ps();

    while i + 16 <= len {
        // SAFETY: `i + 16 <= len` guarantees `a.as_ptr().add(i)` dereferences 16 valid `f32` elements (64 bytes)
        // and `b_bytes.as_ptr().add(i * 4)` dereferences 64 valid bytes (16 `f32`s).
        // `_mm512_loadu_ps` permits unaligned pointers. Target feature `avx512f` is guaranteed by caller contract.
        unsafe {
            let va = _mm512_loadu_ps(a.as_ptr().add(i));
            let vb = _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32);
            let diff = _mm512_sub_ps(va, vb);

            sum_v = _mm512_fmadd_ps(diff, diff, sum_v);
        }

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is guaranteed by caller contract. Operates purely on SIMD registers.
    let mut sum = unsafe { hsum512_ps_avx(sum_v) };

    while i < len {
        // SAFETY: `i < len` guarantees `i` is in bounds for `a` and `i * 4 + 3 < b_bytes.len()` (given caller contract `b_bytes.len() >= a.len() * 4`).
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

/// Computes the dot product distance (negated dot product) between an `f32` slice and a little-endian `u8` byte slice representing `f32` values using AVX-512.
///
/// # Safety
///
/// - **CPU Feature Guarantee:** The caller must ensure that the target CPU supports `avx512f`.
/// - **Slice Lengths:** `b_bytes` must contain at least `a.len() * 4` bytes.
///   Providing a shorter byte slice will result in out-of-bounds pointer reads.
/// - **Memory Alignment:** Slice pointers do not require 64-byte alignment; unaligned SIMD
///   loads (`_mm512_loadu_ps`) are used for both slices.
/// - **Length / Tail Handling:** Valid for any length including 0. Tail elements (< 16) are processed safely via scalar fallbacks.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn dot_product_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = _mm512_setzero_ps();

    while i + 16 <= len {
        // SAFETY: `i + 16 <= len` guarantees `a.as_ptr().add(i)` dereferences 16 valid `f32` elements (64 bytes)
        // and `b_bytes.as_ptr().add(i * 4)` dereferences 64 valid bytes (16 `f32`s).
        // `_mm512_loadu_ps` permits unaligned pointers. Target feature `avx512f` is guaranteed by caller contract.
        unsafe {
            let va = _mm512_loadu_ps(a.as_ptr().add(i));
            let vb = _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32);

            dot_v = _mm512_fmadd_ps(va, vb, dot_v);
        }

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is guaranteed by caller contract. Operates purely on SIMD registers.
    let mut dot = unsafe { hsum512_ps_avx(dot_v) };

    while i < len {
        // SAFETY: `i < len` guarantees `i` is in bounds for `a` and `i * 4 + 3 < b_bytes.len()` (given caller contract `b_bytes.len() >= a.len() * 4`).
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

/// Horizontally sums all 16 `f32` lane elements of an AVX-512 register (`__m512`) into a single `f32` scalar.
///
/// # Safety
///
/// - **CPU Feature Guarantee:** The caller must ensure that the target CPU supports `avx512f`.
/// - **No Pointer Operations:** Operates entirely on SIMD registers; does not dereference memory.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn hsum512_ps_avx(v: __m512) -> f32 {
    // SAFETY: Target feature `avx512f` is guaranteed by caller contract. Operates on SIMD register values without memory access.
    unsafe {
        let v256 = _mm256_add_ps(_mm512_extractf32x8_ps(v, 0), _mm512_extractf32x8_ps(v, 1));
        let vlow = _mm256_castps256_ps128(v256);
        let vhigh = _mm256_extractf128_ps(v256, 1);
        let v128 = _mm_add_ps(vlow, vhigh);
        let v64 = _mm_add_ps(v128, _mm_movehl_ps(v128, v128));
        let v32 = _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55));
        _mm_cvtss_f32(v32)
    }
}

/// Computes the dot product of two `u8` byte slices using AVX-512 VNNI (`_mm512_dpbusd_epi32`).
///
/// # Safety
///
/// - **CPU Feature Guarantee:** The caller must ensure that the target CPU supports `avx512f`, `avx512bw`, and `avx512vnni`.
/// - **Slice Lengths:** Slices `a` and `b` should have equal length. The kernel computes over `min(a.len(), b.len())`.
/// - **Memory Alignment:** Slice pointers do not require 64-byte alignment; unaligned SIMD
///   loads (`_mm512_loadu_si512`) are used.
/// - **Length / Tail Handling:** Valid for any slice length including 0. Tail elements (< 64) are processed safely via scalar fallbacks.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f", enable = "avx512bw", enable = "avx512vnni")]
pub unsafe fn dot_product_u8_avx512vnni(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    let mut sum_v = _mm512_setzero_si512();

    while i + 64 <= len {
        // SAFETY: `i + 64 <= len` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)` dereference
        // 64 valid contiguous bytes. `_mm512_loadu_si512` permits unaligned pointers.
        // Target features `avx512f`, `avx512bw`, and `avx512vnni` are guaranteed by caller contract.
        unsafe {
            let va = _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i);
            let vb = _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i);

            sum_v = _mm512_dpbusd_epi32(sum_v, va, vb);
        }

        i += 64;
    }

    // SAFETY: Target features `avx512f`, `avx512bw`, and `avx512vnni` are guaranteed by caller contract.
    let mut sum = unsafe { hsum512_epi32_avx512(sum_v) } as u32;

    while i < len {
        // SAFETY: `i < len` guarantees `i < a.len()` and `i < b.len()` since `len = min(a.len(), b.len())`.
        sum += unsafe { (*a.get_unchecked(i) as u32) * (*b.get_unchecked(i) as u32) };
        i += 1;
    }

    sum
}

/// Computes the squared Euclidean distance between two `u8` byte slices using AVX-512 BW instructions.
///
/// # Safety
///
/// - **CPU Feature Guarantee:** The caller must ensure that the target CPU supports `avx512f` and `avx512bw`.
/// - **Slice Lengths:** Slices `a` and `b` should have equal length. The kernel computes over `min(a.len(), b.len())`.
/// - **Memory Alignment:** Slice pointers do not require 64-byte alignment; unaligned SIMD
///   loads (`_mm512_loadu_si512`) are used.
/// - **Length / Tail Handling:** Valid for any slice length including 0. Tail elements (< 64) are processed safely via scalar fallbacks.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f", enable = "avx512bw")]
pub unsafe fn euclidean_distance_sq_u8_avx512(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    let (mut sum_v, zero) = (_mm512_setzero_si512(), _mm512_setzero_si512());

    while i + 64 <= len {
        // SAFETY: `i + 64 <= len` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)` dereference
        // 64 valid contiguous bytes. `_mm512_loadu_si512` permits unaligned pointers.
        // Target features `avx512f` and `avx512bw` are guaranteed by caller contract.
        unsafe {
            let va = _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i);
            let vb = _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i);

            let va_lo = _mm512_unpacklo_epi8(va, zero);
            let vb_lo = _mm512_unpacklo_epi8(vb, zero);
            let diff_lo = _mm512_sub_epi16(va_lo, vb_lo);
            let prod_lo = _mm512_madd_epi16(diff_lo, diff_lo);

            let va_hi = _mm512_unpackhi_epi8(va, zero);
            let vb_hi = _mm512_unpackhi_epi8(vb, zero);
            let diff_hi = _mm512_sub_epi16(va_hi, vb_hi);
            let prod_hi = _mm512_madd_epi16(diff_hi, diff_hi);

            sum_v = _mm512_add_epi32(sum_v, prod_lo);
            sum_v = _mm512_add_epi32(sum_v, prod_hi);
        }

        i += 64;
    }

    // SAFETY: Target features `avx512f` and `avx512bw` are guaranteed by caller contract.
    let mut sum = unsafe { hsum512_epi32_avx512(sum_v) } as u32;

    while i < len {
        // SAFETY: `i < len` guarantees `i < a.len()` and `i < b.len()` since `len = min(a.len(), b.len())`.
        unsafe {
            let diff = (*a.get_unchecked(i) as i32) - (*b.get_unchecked(i) as i32);
            sum += (diff * diff) as u32;
        }
        i += 1;
    }

    sum
}

/// Computes dot product and squared norms for two `u8` byte slices using AVX-512 VNNI instructions.
///
/// # Safety
///
/// - **CPU Feature Guarantee:** The caller must ensure that the target CPU supports `avx512f`, `avx512bw`, and `avx512vnni`.
/// - **Slice Lengths:** Slices `a` and `b` should have equal length. The kernel computes over `min(a.len(), b.len())`.
/// - **Memory Alignment:** Slice pointers do not require 64-byte alignment; unaligned SIMD
///   loads (`_mm512_loadu_si512`) are used.
/// - **Length / Tail Handling:** Valid for any slice length including 0. Tail elements (< 64) are processed safely via scalar fallbacks.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f", enable = "avx512bw", enable = "avx512vnni")]
pub unsafe fn cosine_similarity_parts_u8_avx512(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
    let len = a.len().min(b.len());
    let mut i = 0;

    let (mut dot_v, mut norm_a_v, mut norm_b_v) = (
        _mm512_setzero_si512(),
        _mm512_setzero_si512(),
        _mm512_setzero_si512(),
    );

    while i + 64 <= len {
        // SAFETY: `i + 64 <= len` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)` dereference
        // 64 valid contiguous bytes. `_mm512_loadu_si512` permits unaligned pointers.
        // Target features `avx512f`, `avx512bw`, and `avx512vnni` are guaranteed by caller contract.
        unsafe {
            let va = _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i);
            let vb = _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i);

            dot_v = _mm512_dpbusd_epi32(dot_v, va, vb);
            norm_a_v = _mm512_dpbusd_epi32(norm_a_v, va, va);
            norm_b_v = _mm512_dpbusd_epi32(norm_b_v, vb, vb);
        }

        i += 64;
    }

    // SAFETY: Target features `avx512f`, `avx512bw`, and `avx512vnni` are guaranteed by caller contract.
    let (mut dot, mut norm_a_sq, mut norm_b_sq) = unsafe {
        (
            hsum512_epi32_avx512(dot_v) as u32,
            hsum512_epi32_avx512(norm_a_v) as u32,
            hsum512_epi32_avx512(norm_b_v) as u32,
        )
    };

    while i < len {
        // SAFETY: `i < len` guarantees `i < a.len()` and `i < b.len()` since `len = min(a.len(), b.len())`.
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

/// Horizontally sums all 16 32-bit integer lane elements of an AVX-512 register (`__m512i`) into a single `i32` scalar.
///
/// # Safety
///
/// - **CPU Feature Guarantee:** The caller must ensure that the target CPU supports `avx512f`.
/// - **No Pointer Operations:** Operates entirely on SIMD registers; does not dereference memory.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn hsum512_epi32_avx512(v: __m512i) -> i32 {
    // SAFETY: Target feature `avx512f` is guaranteed by caller contract. Operates on SIMD register values without memory access.
    unsafe {
        let v256 = _mm256_add_epi32(_mm512_castsi512_si256(v), _mm512_extracti32x8_epi32(v, 1));
        let vlow = _mm256_castsi256_si128(v256);
        let vhigh = _mm256_extracti128_si256(v256, 1);
        let v128 = _mm_add_epi32(vlow, vhigh);
        let v64 = _mm_add_epi32(v128, _mm_srli_si128(v128, 8));
        let v32 = _mm_add_epi32(v64, _mm_srli_si128(v64, 4));
        _mm_cvtsi128_si32(v32)
    }
}
