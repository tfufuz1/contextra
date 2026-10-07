// FILE-CONTEXT
// ZWECK: AVX2 SIMD-Intrinsics für f32, f32_bytes und u8 Distanzberechnungen.
// INVARIANTEN: Target-Feature "avx2", "fma" garantiert vor Aufruf via Feature-Check.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::kernels::scalar::CosineSimilarityPartsU8;

/// Calculates cosine distance using AVX2 and FMA SIMD intrinsics.
///
/// # Safety
/// Caller must ensure that:
/// - The target CPU supports `avx2` and `fma` target features.
/// - `a` and `b` have equal lengths or sufficient length to prevent out-of-bounds reads.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
// SAFETY: Target features "avx2" and "fma" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub(crate) unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Valid AVX2 zero vector initialization.
    let mut dot_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: Valid AVX2 zero vector initialization.
    let mut norm_a_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: Valid AVX2 zero vector initialization.
    let mut norm_b_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: Loop condition i + 8 <= len guarantees a pointer offset i..i+8 is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 8 <= len guarantees b pointer offset i..i+8 is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(i)) };

        // SAFETY: FMA target feature enabled and inputs are valid 256-bit float registers.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };
        // SAFETY: FMA target feature enabled and inputs are valid 256-bit float registers.
        norm_a_v = unsafe { _mm256_fmadd_ps(va, va, norm_a_v) };
        // SAFETY: FMA target feature enabled and inputs are valid 256-bit float registers.
        norm_b_v = unsafe { _mm256_fmadd_ps(vb, vb, norm_b_v) };

        i += 8;
    }

    // SAFETY: hsum256_ps_avx requires target_feature avx2 and valid 256-bit register.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };
    // SAFETY: hsum256_ps_avx requires target_feature avx2 and valid 256-bit register.
    let mut norm_a = unsafe { hsum256_ps_avx(norm_a_v) };
    // SAFETY: hsum256_ps_avx requires target_feature avx2 and valid 256-bit register.
    let mut norm_b = unsafe { hsum256_ps_avx(norm_b_v) };

    while i < len {
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into a.
        let x = unsafe { *a.get_unchecked(i) };
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into b.
        let y = unsafe { *b.get_unchecked(i) };
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
        i += 1;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    let sim = (dot / denom).clamp(-1.0, 1.0);
    1.0 - sim
}

/// Calculates Euclidean distance using AVX2 and FMA SIMD intrinsics.
///
/// # Safety
/// Caller must ensure that:
/// - The target CPU supports `avx2` and `fma` target features.
/// - `a` and `b` have equal lengths or sufficient length to prevent out-of-bounds reads.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
// SAFETY: Target features "avx2" and "fma" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub(crate) unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Valid AVX2 zero vector initialization.
    let mut sum_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: Loop condition i + 8 <= len guarantees a pointer offset i..i+8 is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 8 <= len guarantees b pointer offset i..i+8 is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(i)) };
        // SAFETY: Valid AVX2 vector subtraction.
        let diff = unsafe { _mm256_sub_ps(va, vb) };

        // SAFETY: FMA target feature enabled and inputs are valid 256-bit float registers.
        sum_v = unsafe { _mm256_fmadd_ps(diff, diff, sum_v) };

        i += 8;
    }

    // SAFETY: hsum256_ps_avx requires target_feature avx2 and valid 256-bit register.
    let mut sum = unsafe { hsum256_ps_avx(sum_v) };

    while i < len {
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into a and b.
        let diff = unsafe { *a.get_unchecked(i) - *b.get_unchecked(i) };
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Calculates dot product using AVX2 and FMA SIMD intrinsics.
///
/// # Safety
/// Caller must ensure that:
/// - The target CPU supports `avx2` and `fma` target features.
/// - `a` and `b` have equal lengths or sufficient length to prevent out-of-bounds reads.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
// SAFETY: Target features "avx2" and "fma" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub(crate) unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Valid AVX2 zero vector initialization.
    let mut dot_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: Loop condition i + 8 <= len guarantees a pointer offset i..i+8 is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 8 <= len guarantees b pointer offset i..i+8 is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(i)) };

        // SAFETY: FMA target feature enabled and inputs are valid 256-bit float registers.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };

        i += 8;
    }

    // SAFETY: hsum256_ps_avx requires target_feature avx2 and valid 256-bit register.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };

    while i < len {
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into a and b.
        dot += unsafe { *a.get_unchecked(i) * *b.get_unchecked(i) };
        i += 1;
    }

    -dot
}

/// Calculates cosine distance between f32 slice and raw f32 byte slice using AVX2 and FMA.
///
/// # Safety
/// Caller must ensure that:
/// - The target CPU supports `avx2` and `fma` target features.
/// - `b_bytes` contains at least `a.len() * 4` bytes.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
// SAFETY: Target features "avx2" and "fma" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub(crate) unsafe fn cosine_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Valid AVX2 zero vector initialization.
    let mut dot_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: Valid AVX2 zero vector initialization.
    let mut norm_a_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: Valid AVX2 zero vector initialization.
    let mut norm_b_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: Loop condition i + 8 <= len guarantees a pointer offset i..i+8 is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees b_bytes pointer offset (i*4)..(i*4+32) is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let vb = unsafe { _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };

        // SAFETY: FMA target feature enabled and inputs are valid 256-bit float registers.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };
        // SAFETY: FMA target feature enabled and inputs are valid 256-bit float registers.
        norm_a_v = unsafe { _mm256_fmadd_ps(va, va, norm_a_v) };
        // SAFETY: FMA target feature enabled and inputs are valid 256-bit float registers.
        norm_b_v = unsafe { _mm256_fmadd_ps(vb, vb, norm_b_v) };

        i += 8;
    }

    // SAFETY: hsum256_ps_avx requires target_feature avx2 and valid 256-bit register.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };
    // SAFETY: hsum256_ps_avx requires target_feature avx2 and valid 256-bit register.
    let mut norm_a = unsafe { hsum256_ps_avx(norm_a_v) };
    // SAFETY: hsum256_ps_avx requires target_feature avx2 and valid 256-bit register.
    let mut norm_b = unsafe { hsum256_ps_avx(norm_b_v) };

    while i < len {
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into a.
        let x = unsafe { *a.get_unchecked(i) };
        let b_val = f32::from_le_bytes([
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4) },
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 + 1 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 + 2 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 + 3 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4 + 3) },
        ]);
        dot += x * b_val;
        norm_a += x * x;
        norm_b += b_val * b_val;
        i += 1;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    let sim = (dot / denom).clamp(-1.0, 1.0);
    1.0 - sim
}

/// Calculates Euclidean distance between f32 slice and raw f32 byte slice using AVX2 and FMA.
///
/// # Safety
/// Caller must ensure that:
/// - The target CPU supports `avx2` and `fma` target features.
/// - `b_bytes` contains at least `a.len() * 4` bytes.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
// SAFETY: Target features "avx2" and "fma" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub(crate) unsafe fn euclidean_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Valid AVX2 zero vector initialization.
    let mut sum_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: Loop condition i + 8 <= len guarantees a pointer offset i..i+8 is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees b_bytes pointer offset (i*4)..(i*4+32) is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let vb = unsafe { _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };
        // SAFETY: Valid AVX2 vector subtraction.
        let diff = unsafe { _mm256_sub_ps(va, vb) };

        // SAFETY: FMA target feature enabled and inputs are valid 256-bit float registers.
        sum_v = unsafe { _mm256_fmadd_ps(diff, diff, sum_v) };

        i += 8;
    }

    // SAFETY: hsum256_ps_avx requires target_feature avx2 and valid 256-bit register.
    let mut sum = unsafe { hsum256_ps_avx(sum_v) };

    while i < len {
        let b_val = f32::from_le_bytes([
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4) },
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 + 1 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 + 2 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 + 3 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4 + 3) },
        ]);
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into a.
        let diff = unsafe { *a.get_unchecked(i) } - b_val;
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Calculates dot product between f32 slice and raw f32 byte slice using AVX2 and FMA.
///
/// # Safety
/// Caller must ensure that:
/// - The target CPU supports `avx2` and `fma` target features.
/// - `b_bytes` contains at least `a.len() * 4` bytes.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
// SAFETY: Target features "avx2" and "fma" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub(crate) unsafe fn dot_product_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Valid AVX2 zero vector initialization.
    let mut dot_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: Loop condition i + 8 <= len guarantees a pointer offset i..i+8 is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees b_bytes pointer offset (i*4)..(i*4+32) is in bounds; unaligned load _mm256_loadu_ps is safe for unaligned pointers.
        let vb = unsafe { _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };

        // SAFETY: FMA target feature enabled and inputs are valid 256-bit float registers.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };

        i += 8;
    }

    // SAFETY: hsum256_ps_avx requires target_feature avx2 and valid 256-bit register.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };

    while i < len {
        let b_val = f32::from_le_bytes([
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4) },
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 + 1 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 + 2 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
            // SAFETY: Precondition b_bytes.len() >= a.len() * 4 guarantees i*4 + 3 is in bounds.
            unsafe { *b_bytes.get_unchecked(i * 4 + 3) },
        ]);
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into a.
        dot += unsafe { *a.get_unchecked(i) } * b_val;
        i += 1;
    }

    -dot
}

/// Horizontal sum of 256-bit float vector.
///
/// # Safety
/// Caller must ensure that:
/// - The target CPU supports `avx2` target feature.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
// SAFETY: Target feature "avx2" enabled by target_feature attribute.
pub(crate) unsafe fn hsum256_ps_avx(v: __m256) -> f32 {
    // SAFETY: Valid AVX2 cast from 256-bit float vector to lower 128-bit lane.
    let vlow = unsafe { _mm256_castps256_ps128(v) };
    // SAFETY: Valid AVX2 extraction of upper 128-bit lane (index 1).
    let vhigh = unsafe { _mm256_extractf128_ps(v, 1) };
    // SAFETY: Valid SSE float vector addition on two 128-bit registers.
    let v128 = unsafe { _mm_add_ps(vlow, vhigh) };
    // SAFETY: Valid SSE move high to low and float vector addition.
    let v64 = unsafe { _mm_add_ps(v128, _mm_movehl_ps(v128, v128)) };
    // SAFETY: Valid SSE shuffle and scalar float addition.
    let v32 = unsafe { _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55)) };
    // SAFETY: Valid SSE extraction of scalar float from 128-bit register.
    unsafe { _mm_cvtss_f32(v32) }
}

/// Calculates u8 dot product using AVX2 SIMD intrinsics.
///
/// # Safety
/// Caller must ensure that:
/// - The target CPU supports `avx2` target feature.
/// - `a` and `b` have equal lengths or sufficient length to prevent out-of-bounds reads.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
// SAFETY: Target feature "avx2" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub(crate) unsafe fn dot_product_u8_avx2(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Valid AVX2 zero vector initialization.
    let mut sum_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Valid AVX2 zero vector initialization.
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        // SAFETY: Loop condition i + 32 <= len guarantees a pointer offset i..i+32 is in bounds; unaligned load _mm256_loadu_si256 is safe for unaligned pointers.
        let va = unsafe { _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i) };
        // SAFETY: Loop condition i + 32 <= len guarantees b pointer offset i..i+32 is in bounds; unaligned load _mm256_loadu_si256 is safe for unaligned pointers.
        let vb = unsafe { _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i) };

        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        // SAFETY: Valid AVX2 multiply 16-bit and add adjacent pairs to 32-bit integers.
        let prod_lo = unsafe { _mm256_madd_epi16(va_lo, vb_lo) };

        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        // SAFETY: Valid AVX2 multiply 16-bit and add adjacent pairs to 32-bit integers.
        let prod_hi = unsafe { _mm256_madd_epi16(va_hi, vb_hi) };

        // SAFETY: Valid AVX2 32-bit integer vector addition.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_lo) };
        // SAFETY: Valid AVX2 32-bit integer vector addition.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_hi) };

        i += 32;
    }

    // SAFETY: hsum256_epi32_avx2 requires target_feature avx2 and valid 256-bit register.
    let mut sum = unsafe { hsum256_epi32_avx2(sum_v) } as u32;

    while i < len {
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into a and b.
        sum += unsafe { (*a.get_unchecked(i) as u32) * (*b.get_unchecked(i) as u32) };
        i += 1;
    }

    sum
}

/// Calculates u8 squared L2 distance using AVX2 SIMD intrinsics.
///
/// # Safety
/// Caller must ensure that:
/// - The target CPU supports `avx2` target feature.
/// - `a` and `b` have equal lengths or sufficient length to prevent out-of-bounds reads.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
// SAFETY: Target feature "avx2" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub(crate) unsafe fn euclidean_distance_sq_u8_avx2(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Valid AVX2 zero vector initialization.
    let mut sum_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Valid AVX2 zero vector initialization.
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        // SAFETY: Loop condition i + 32 <= len guarantees a pointer offset i..i+32 is in bounds; unaligned load _mm256_loadu_si256 is safe for unaligned pointers.
        let va = unsafe { _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i) };
        // SAFETY: Loop condition i + 32 <= len guarantees b pointer offset i..i+32 is in bounds; unaligned load _mm256_loadu_si256 is safe for unaligned pointers.
        let vb = unsafe { _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i) };

        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        // SAFETY: Valid AVX2 16-bit integer subtraction.
        let diff_lo = unsafe { _mm256_sub_epi16(va_lo, vb_lo) };
        // SAFETY: Valid AVX2 multiply 16-bit and add adjacent pairs to 32-bit integers.
        let prod_lo = unsafe { _mm256_madd_epi16(diff_lo, diff_lo) };

        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        // SAFETY: Valid AVX2 16-bit integer subtraction.
        let diff_hi = unsafe { _mm256_sub_epi16(va_hi, vb_hi) };
        // SAFETY: Valid AVX2 multiply 16-bit and add adjacent pairs to 32-bit integers.
        let prod_hi = unsafe { _mm256_madd_epi16(diff_hi, diff_hi) };

        // SAFETY: Valid AVX2 32-bit integer vector addition.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_lo) };
        // SAFETY: Valid AVX2 32-bit integer vector addition.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_hi) };

        i += 32;
    }

    // SAFETY: hsum256_epi32_avx2 requires target_feature avx2 and valid 256-bit register.
    let mut sum = unsafe { hsum256_epi32_avx2(sum_v) } as u32;

    while i < len {
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into a and b.
        let diff = unsafe { (*a.get_unchecked(i) as i32) - (*b.get_unchecked(i) as i32) };
        sum += (diff * diff) as u32;
        i += 1;
    }

    sum
}

/// Calculates u8 cosine similarity parts (dot, norm_a_sq, norm_b_sq) using AVX2 SIMD intrinsics.
///
/// # Safety
/// Caller must ensure that:
/// - The target CPU supports `avx2` target feature.
/// - `a` and `b` have equal lengths or sufficient length to prevent out-of-bounds reads.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
// SAFETY: Target feature "avx2" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub(crate) unsafe fn cosine_similarity_parts_u8_avx2(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Valid AVX2 zero vector initialization.
    let mut dot_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Valid AVX2 zero vector initialization.
    let mut norm_a_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Valid AVX2 zero vector initialization.
    let mut norm_b_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Valid AVX2 zero vector initialization.
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        // SAFETY: Loop condition i + 32 <= len guarantees a pointer offset i..i+32 is in bounds; unaligned load _mm256_loadu_si256 is safe for unaligned pointers.
        let va = unsafe { _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i) };
        // SAFETY: Loop condition i + 32 <= len guarantees b pointer offset i..i+32 is in bounds; unaligned load _mm256_loadu_si256 is safe for unaligned pointers.
        let vb = unsafe { _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i) };

        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        // SAFETY: Valid AVX2 32-bit addition and madd_epi16 on 16-bit unpacked registers.
        dot_v = unsafe { _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_lo, vb_lo)) };
        // SAFETY: Valid AVX2 32-bit addition and madd_epi16 on 16-bit unpacked registers.
        norm_a_v = unsafe { _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_lo, va_lo)) };
        // SAFETY: Valid AVX2 32-bit addition and madd_epi16 on 16-bit unpacked registers.
        norm_b_v = unsafe { _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_lo, vb_lo)) };

        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        // SAFETY: Valid AVX2 byte unpacking into 16-bit integers.
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        // SAFETY: Valid AVX2 32-bit addition and madd_epi16 on 16-bit unpacked registers.
        dot_v = unsafe { _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_hi, vb_hi)) };
        // SAFETY: Valid AVX2 32-bit addition and madd_epi16 on 16-bit unpacked registers.
        norm_a_v = unsafe { _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_hi, va_hi)) };
        // SAFETY: Valid AVX2 32-bit addition and madd_epi16 on 16-bit unpacked registers.
        norm_b_v = unsafe { _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_hi, vb_hi)) };

        i += 32;
    }

    // SAFETY: hsum256_epi32_avx2 requires target_feature avx2 and valid 256-bit register.
    let mut dot = unsafe { hsum256_epi32_avx2(dot_v) } as u32;
    // SAFETY: hsum256_epi32_avx2 requires target_feature avx2 and valid 256-bit register.
    let mut norm_a_sq = unsafe { hsum256_epi32_avx2(norm_a_v) } as u32;
    // SAFETY: hsum256_epi32_avx2 requires target_feature avx2 and valid 256-bit register.
    let mut norm_b_sq = unsafe { hsum256_epi32_avx2(norm_b_v) } as u32;

    while i < len {
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into a.
        let x = unsafe { *a.get_unchecked(i) as u32 };
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into b.
        let y = unsafe { *b.get_unchecked(i) as u32 };
        dot += x * y;
        norm_a_sq += x * x;
        norm_b_sq += y * y;
        i += 1;
    }

    CosineSimilarityPartsU8 {
        dot,
        norm_a_sq,
        norm_b_sq,
    }
}

/// Horizontal sum of 256-bit integer vector.
///
/// # Safety
/// Caller must ensure that:
/// - The target CPU supports `avx2` target feature.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
// SAFETY: Target feature "avx2" enabled by target_feature attribute.
pub(crate) unsafe fn hsum256_epi32_avx2(v: __m256i) -> i32 {
    // SAFETY: Valid AVX2 cast from 256-bit integer vector to lower 128-bit lane.
    let vlow = unsafe { _mm256_castsi256_si128(v) };
    // SAFETY: Valid AVX2 extraction of upper 128-bit lane (index 1).
    let vhigh = unsafe { _mm256_extracti128_si256(v, 1) };
    // SAFETY: Valid SSE 32-bit integer vector addition on two 128-bit registers.
    let v128 = unsafe { _mm_add_epi32(vlow, vhigh) };
    // SAFETY: Valid SSE 128-bit shift right logical by 8 bytes and 32-bit integer addition.
    let v64 = unsafe { _mm_add_epi32(v128, _mm_srli_si128(v128, 8)) };
    // SAFETY: Valid SSE 128-bit shift right logical by 4 bytes and 32-bit integer addition.
    let v32 = unsafe { _mm_add_epi32(v64, _mm_srli_si128(v64, 4)) };
    // SAFETY: Valid SSE extraction of lowest 32-bit integer from 128-bit register.
    unsafe { _mm_cvtsi128_si32(v32) }
}
