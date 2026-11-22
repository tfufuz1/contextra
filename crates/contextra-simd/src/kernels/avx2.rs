// FILE-CONTEXT
// ZWECK: AVX2 SIMD-Intrinsics für f32, f32_bytes und u8 Distanzberechnungen.
// INVARIANTEN: Target-Feature "avx2", "fma" vor Aufruf via Feature-Check verifiziert.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::kernels::scalar::CosineSimilarityPartsU8;

/// Calculates cosine distance between two float vectors using AVX2 and FMA instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx2` and `fma` instruction set extensions.
/// 2. `a` and `b` have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target features `avx2` and `fma` are verified by caller and enabled on function attribute.
    let mut dot_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: Target features `avx2` and `fma` are verified by caller and enabled on function attribute.
    let mut norm_a_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: Target features `avx2` and `fma` are verified by caller and enabled on function attribute.
    let mut norm_b_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY:
        // 1. Feature: Target features `avx2` and `fma` are enabled via function attribute.
        // 2. Length/Tail: `i + 8 <= len` where `len == a.len() == b.len()` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)` are valid for reading 8 contiguous `f32` elements.
        // 3. Alignment: `_mm256_loadu_ps` handles unaligned memory addresses safely.
        let (va, vb) = unsafe {
            (
                _mm256_loadu_ps(a.as_ptr().add(i)),
                _mm256_loadu_ps(b.as_ptr().add(i)),
            )
        };

        // SAFETY: Target features `avx2` and `fma` are enabled and vector registers are valid `__m256`.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };
        // SAFETY: Target features `avx2` and `fma` are enabled and vector registers are valid `__m256`.
        norm_a_v = unsafe { _mm256_fmadd_ps(va, va, norm_a_v) };
        // SAFETY: Target features `avx2` and `fma` are enabled and vector registers are valid `__m256`.
        norm_b_v = unsafe { _mm256_fmadd_ps(vb, vb, norm_b_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `dot_v`, `norm_a_v`, `norm_b_v` are valid `__m256` vector registers.
    let (mut dot, mut norm_a, mut norm_b) = unsafe {
        (
            hsum256_ps_avx(dot_v),
            hsum256_ps_avx(norm_a_v),
            hsum256_ps_avx(norm_b_v),
        )
    };

    while i < len {
        // SAFETY:
        // 1. Length/Tail: `i < len` where `len == a.len() == b.len()` guarantees `i` is within bounds of slices `a` and `b`.
        let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };
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

/// Calculates Euclidean distance between two float vectors using AVX2 and FMA instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx2` and `fma` instruction set extensions.
/// 2. `a` and `b` have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target features `avx2` and `fma` are verified by caller and enabled on function attribute.
    let mut sum_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY:
        // 1. Feature: Target features `avx2` and `fma` are enabled via function attribute.
        // 2. Length/Tail: `i + 8 <= len` where `len == a.len() == b.len()` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)` point to 8 readable `f32` elements.
        // 3. Alignment: `_mm256_loadu_ps` handles unaligned memory addresses safely.
        let (va, vb) = unsafe {
            (
                _mm256_loadu_ps(a.as_ptr().add(i)),
                _mm256_loadu_ps(b.as_ptr().add(i)),
            )
        };
        // SAFETY: Target feature `avx2` is enabled and `va`, `vb` are valid `__m256` vector registers.
        let diff = unsafe { _mm256_sub_ps(va, vb) };

        // SAFETY: Target features `avx2` and `fma` are enabled and vector registers are valid `__m256`.
        sum_v = unsafe { _mm256_fmadd_ps(diff, diff, sum_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `sum_v` is a valid `__m256` vector register.
    let mut sum = unsafe { hsum256_ps_avx(sum_v) };

    while i < len {
        // SAFETY: `i < len` where `len == a.len() == b.len()` guarantees `i` is within bounds of slices `a` and `b`.
        let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };
        let diff = x - y;
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Calculates dot product between two float vectors using AVX2 and FMA instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx2` and `fma` instruction set extensions.
/// 2. `a` and `b` have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target features `avx2` and `fma` are verified by caller and enabled on function attribute.
    let mut dot_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY:
        // 1. Feature: Target features `avx2` and `fma` are enabled via function attribute.
        // 2. Length/Tail: `i + 8 <= len` where `len == a.len() == b.len()` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)` point to 8 readable `f32` elements.
        // 3. Alignment: `_mm256_loadu_ps` handles unaligned memory addresses safely.
        let (va, vb) = unsafe {
            (
                _mm256_loadu_ps(a.as_ptr().add(i)),
                _mm256_loadu_ps(b.as_ptr().add(i)),
            )
        };

        // SAFETY: Target features `avx2` and `fma` are enabled and vector registers are valid `__m256`.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `dot_v` is a valid `__m256` vector register.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };

    while i < len {
        // SAFETY: `i < len` where `len == a.len() == b.len()` guarantees `i` is within bounds of slices `a` and `b`.
        let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };
        dot += x * y;
        i += 1;
    }

    -dot
}

/// Calculates cosine distance between an f32 vector and raw LE f32 bytes using AVX2 and FMA instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx2` and `fma` instruction set extensions.
/// 2. `b_bytes` contains at least `a.len() * 4` readable bytes.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn cosine_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target features `avx2` and `fma` are verified by caller and enabled on function attribute.
    let mut dot_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: Target features `avx2` and `fma` are verified by caller and enabled on function attribute.
    let mut norm_a_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: Target features `avx2` and `fma` are verified by caller and enabled on function attribute.
    let mut norm_b_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY:
        // 1. Feature: Target features `avx2` and `fma` are enabled via function attribute.
        // 2. Length/Tail: `i + 8 <= len` and `b_bytes.len() >= a.len() * 4` guarantee `a.as_ptr().add(i)` and `b_bytes.as_ptr().add(i * 4)` point to readable memory for 8 `f32`s (32 bytes).
        // 3. Alignment: `_mm256_loadu_ps` handles unaligned memory loads safely.
        let (va, vb) = unsafe {
            (
                _mm256_loadu_ps(a.as_ptr().add(i)),
                _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32),
            )
        };

        // SAFETY: Target features `avx2` and `fma` are enabled and vector registers are valid `__m256`.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };
        // SAFETY: Target features `avx2` and `fma` are enabled and vector registers are valid `__m256`.
        norm_a_v = unsafe { _mm256_fmadd_ps(va, va, norm_a_v) };
        // SAFETY: Target features `avx2` and `fma` are enabled and vector registers are valid `__m256`.
        norm_b_v = unsafe { _mm256_fmadd_ps(vb, vb, norm_b_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `dot_v`, `norm_a_v`, `norm_b_v` are valid `__m256` vector registers.
    let (mut dot, mut norm_a, mut norm_b) = unsafe {
        (
            hsum256_ps_avx(dot_v),
            hsum256_ps_avx(norm_a_v),
            hsum256_ps_avx(norm_b_v),
        )
    };

    while i < len {
        // SAFETY:
        // 1. Length/Tail: `i < len` and `b_bytes.len() >= a.len() * 4` guarantee indices are within bounds of `a` and `b_bytes`.
        // 2. Alignment: `from_le_bytes` constructs `f32` from unaligned byte slice.
        let (x, b_val) = unsafe {
            (
                *a.get_unchecked(i),
                f32::from_le_bytes([
                    *b_bytes.get_unchecked(i * 4),
                    *b_bytes.get_unchecked(i * 4 + 1),
                    *b_bytes.get_unchecked(i * 4 + 2),
                    *b_bytes.get_unchecked(i * 4 + 3),
                ]),
            )
        };
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

/// Calculates Euclidean distance between an f32 vector and raw LE f32 bytes using AVX2 and FMA instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx2` and `fma` instruction set extensions.
/// 2. `b_bytes` contains at least `a.len() * 4` readable bytes.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn euclidean_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target features `avx2` and `fma` are verified by caller and enabled on function attribute.
    let mut sum_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY:
        // 1. Feature: Target features `avx2` and `fma` are enabled via function attribute.
        // 2. Length/Tail: `i + 8 <= len` and `b_bytes.len() >= a.len() * 4` guarantee `a.as_ptr().add(i)` and `b_bytes.as_ptr().add(i * 4)` point to readable memory for 8 `f32`s (32 bytes).
        // 3. Alignment: `_mm256_loadu_ps` handles unaligned memory loads safely.
        let (va, vb) = unsafe {
            (
                _mm256_loadu_ps(a.as_ptr().add(i)),
                _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32),
            )
        };
        // SAFETY: Target feature `avx2` is enabled and `va`, `vb` are valid `__m256` vector registers.
        let diff = unsafe { _mm256_sub_ps(va, vb) };

        // SAFETY: Target features `avx2` and `fma` are enabled and vector registers are valid `__m256`.
        sum_v = unsafe { _mm256_fmadd_ps(diff, diff, sum_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `sum_v` is a valid `__m256` vector register.
    let mut sum = unsafe { hsum256_ps_avx(sum_v) };

    while i < len {
        // SAFETY:
        // 1. Length/Tail: `i < len` and `b_bytes.len() >= a.len() * 4` guarantee indices are within bounds.
        // 2. Alignment: `from_le_bytes` constructs `f32` from unaligned byte slice.
        let (x, b_val) = unsafe {
            (
                *a.get_unchecked(i),
                f32::from_le_bytes([
                    *b_bytes.get_unchecked(i * 4),
                    *b_bytes.get_unchecked(i * 4 + 1),
                    *b_bytes.get_unchecked(i * 4 + 2),
                    *b_bytes.get_unchecked(i * 4 + 3),
                ]),
            )
        };
        let diff = x - b_val;
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Calculates dot product between an f32 vector and raw LE f32 bytes using AVX2 and FMA instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx2` and `fma` instruction set extensions.
/// 2. `b_bytes` contains at least `a.len() * 4` readable bytes.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn dot_product_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target features `avx2` and `fma` are verified by caller and enabled on function attribute.
    let mut dot_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY:
        // 1. Feature: Target features `avx2` and `fma` are enabled via function attribute.
        // 2. Length/Tail: `i + 8 <= len` and `b_bytes.len() >= a.len() * 4` guarantee readable memory.
        // 3. Alignment: `_mm256_loadu_ps` handles unaligned memory loads safely.
        let (va, vb) = unsafe {
            (
                _mm256_loadu_ps(a.as_ptr().add(i)),
                _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32),
            )
        };

        // SAFETY: Target features `avx2` and `fma` are enabled and vector registers are valid `__m256`.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `dot_v` is a valid `__m256` vector register.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };

    while i < len {
        // SAFETY:
        // 1. Length/Tail: `i < len` and `b_bytes.len() >= a.len() * 4` guarantee indices are within bounds.
        // 2. Alignment: `from_le_bytes` constructs `f32` from unaligned byte slice.
        let (x, b_val) = unsafe {
            (
                *a.get_unchecked(i),
                f32::from_le_bytes([
                    *b_bytes.get_unchecked(i * 4),
                    *b_bytes.get_unchecked(i * 4 + 1),
                    *b_bytes.get_unchecked(i * 4 + 2),
                    *b_bytes.get_unchecked(i * 4 + 3),
                ]),
            )
        };
        dot += x * b_val;
        i += 1;
    }

    -dot
}

/// Horizontal sum of 8 x f32 values in a 256-bit AVX vector register.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx2` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn hsum256_ps_avx(v: __m256) -> f32 {
    // SAFETY: Target feature `avx2` is enabled and `v` is a valid `__m256` register.
    let vlow = unsafe { _mm256_castps256_ps128(v) };
    // SAFETY: Target feature `avx2` is enabled and `v` is a valid `__m256` register.
    let vhigh = unsafe { _mm256_extractf128_ps(v, 1) };
    // SAFETY: Target feature `avx2` is enabled and `vlow`, `vhigh` are valid `__m128` registers.
    let v128 = unsafe { _mm_add_ps(vlow, vhigh) };
    // SAFETY: Target feature `avx2` is enabled and `v128` is a valid `__m128` register.
    let v64 = unsafe { _mm_add_ps(v128, _mm_movehl_ps(v128, v128)) };
    // SAFETY: Target feature `avx2` is enabled and `v64` is a valid `__m128` register.
    let v32 = unsafe { _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55)) };
    // SAFETY: Target feature `avx2` is enabled and `v32` contains a valid scalar float at lane 0.
    unsafe { _mm_cvtss_f32(v32) }
}

/// Calculates dot product between two byte vectors using AVX2 instructions.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx2` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn dot_product_u8_avx2(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target feature `avx2` is verified by caller and enabled on function attribute.
    let mut sum_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Target feature `avx2` is verified by caller and enabled on function attribute.
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        // SAFETY:
        // 1. Feature: Target feature `avx2` is enabled via function attribute.
        // 2. Length/Tail: `i + 32 <= len` where `len = min(a.len(), b.len())` guarantees 32 readable `u8` elements.
        // 3. Alignment: `_mm256_loadu_si256` handles unaligned byte loads safely.
        let (va, vb) = unsafe {
            (
                _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i),
                _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i),
            )
        };

        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let prod_lo = unsafe { _mm256_madd_epi16(va_lo, vb_lo) };

        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let prod_hi = unsafe { _mm256_madd_epi16(va_hi, vb_hi) };

        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_lo) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_hi) };

        i += 32;
    }

    // SAFETY: Target feature `avx2` is enabled and `sum_v` is a valid `__m256i` vector register.
    let mut sum = u32::from_ne_bytes(unsafe { hsum256_epi32_avx2(sum_v) }.to_ne_bytes());

    while i < len {
        // SAFETY: `i < len` where `len = min(a.len(), b.len())` guarantees `i` is within bounds.
        let (x, y) = unsafe { (*a.get_unchecked(i) as u32, *b.get_unchecked(i) as u32) };
        sum += x * y;
        i += 1;
    }

    sum
}

/// Calculates squared Euclidean distance between two byte vectors using AVX2 instructions.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx2` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn euclidean_distance_sq_u8_avx2(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target feature `avx2` is verified by caller and enabled on function attribute.
    let mut sum_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Target feature `avx2` is verified by caller and enabled on function attribute.
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        // SAFETY:
        // 1. Feature: Target feature `avx2` is enabled via function attribute.
        // 2. Length/Tail: `i + 32 <= len` guarantees 32 readable `u8` elements in `a` and `b`.
        // 3. Alignment: `_mm256_loadu_si256` handles unaligned byte loads safely.
        let (va, vb) = unsafe {
            (
                _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i),
                _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i),
            )
        };

        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let diff_lo = unsafe { _mm256_sub_epi16(va_lo, vb_lo) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let prod_lo = unsafe { _mm256_madd_epi16(diff_lo, diff_lo) };

        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let diff_hi = unsafe { _mm256_sub_epi16(va_hi, vb_hi) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let prod_hi = unsafe { _mm256_madd_epi16(diff_hi, diff_hi) };

        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_lo) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_hi) };

        i += 32;
    }

    // SAFETY: Target feature `avx2` is enabled and `sum_v` is a valid `__m256i` vector register.
    let mut sum = u32::from_ne_bytes(unsafe { hsum256_epi32_avx2(sum_v) }.to_ne_bytes());

    while i < len {
        // SAFETY: `i < len` where `len = min(a.len(), b.len())` guarantees `i` is within bounds.
        let diff = unsafe { (*a.get_unchecked(i) as i32) - (*b.get_unchecked(i) as i32) };
        let abs_diff = diff.unsigned_abs();
        sum += abs_diff * abs_diff;
        i += 1;
    }

    sum
}

/// Calculates cosine similarity parts (dot, norm_a_sq, norm_b_sq) for byte vectors using AVX2 instructions.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx2` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn cosine_similarity_parts_u8_avx2(
    a: &[u8],
    b: &[u8],
) -> CosineSimilarityPartsU8 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target feature `avx2` is verified by caller and enabled on function attribute.
    let mut dot_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Target feature `avx2` is verified by caller and enabled on function attribute.
    let mut norm_a_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Target feature `avx2` is verified by caller and enabled on function attribute.
    let mut norm_b_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Target feature `avx2` is verified by caller and enabled on function attribute.
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        // SAFETY:
        // 1. Feature: Target feature `avx2` is enabled via function attribute.
        // 2. Length/Tail: `i + 32 <= len` guarantees 32 readable `u8` elements in `a` and `b`.
        // 3. Alignment: `_mm256_loadu_si256` handles unaligned byte loads safely.
        let (va, vb) = unsafe {
            (
                _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i),
                _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i),
            )
        };

        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        dot_v = unsafe { _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_lo, vb_lo)) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        norm_a_v = unsafe { _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_lo, va_lo)) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        norm_b_v = unsafe { _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_lo, vb_lo)) };

        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        dot_v = unsafe { _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_hi, vb_hi)) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        norm_a_v = unsafe { _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_hi, va_hi)) };
        // SAFETY: Target feature `avx2` is enabled and registers are valid `__m256i`.
        norm_b_v = unsafe { _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_hi, vb_hi)) };

        i += 32;
    }

    // SAFETY: Target feature `avx2` is enabled and vector registers are valid `__m256i`.
    let (mut dot, mut norm_a_sq, mut norm_b_sq) = unsafe {
        (
            u32::from_ne_bytes(hsum256_epi32_avx2(dot_v).to_ne_bytes()),
            u32::from_ne_bytes(hsum256_epi32_avx2(norm_a_v).to_ne_bytes()),
            u32::from_ne_bytes(hsum256_epi32_avx2(norm_b_v).to_ne_bytes()),
        )
    };

    while i < len {
        // SAFETY: `i < len` where `len = min(a.len(), b.len())` guarantees `i` is within bounds.
        let (x, y) = unsafe { (*a.get_unchecked(i) as u32, *b.get_unchecked(i) as u32) };
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

/// Horizontal sum of 8 x 32-bit signed integers in a 256-bit AVX2 vector register.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx2` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn hsum256_epi32_avx2(v: __m256i) -> i32 {
    // SAFETY: Target feature `avx2` is enabled and `v` is a valid `__m256i` register.
    let vlow = unsafe { _mm256_castsi256_si128(v) };
    // SAFETY: Target feature `avx2` is enabled and `v` is a valid `__m256i` register.
    let vhigh = unsafe { _mm256_extracti128_si256(v, 1) };
    // SAFETY: Target feature `avx2` is enabled and `vlow`, `vhigh` are valid `__m128i` registers.
    let v128 = unsafe { _mm_add_epi32(vlow, vhigh) };
    // SAFETY: Target feature `avx2` is enabled and `v128` is a valid `__m128i` register.
    let v64 = unsafe { _mm_add_epi32(v128, _mm_srli_si128(v128, 8)) };
    // SAFETY: Target feature `avx2` is enabled and `v64` is a valid `__m128i` register.
    let v32 = unsafe { _mm_add_epi32(v64, _mm_srli_si128(v64, 4)) };
    // SAFETY: Target feature `avx2` is enabled and `v32` contains a valid 32-bit int at lane 0.
    unsafe { _mm_cvtsi128_si32(v32) }
}
