// FILE-CONTEXT
// ZWECK: AVX-512 SIMD-Intrinsics für f32, f32_bytes und u8 VNNI Distanzberechnungen.
// INVARIANTEN: Target-Features guaranteed by caller or feature-checks.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::kernels::scalar::CosineSimilarityPartsU8;

/// Calculates cosine distance between two float vectors using AVX-512 instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx512f` instruction set extension.
/// 2. `a` and `b` have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn cosine_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut dot_v = unsafe { _mm512_setzero_ps() };
    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut norm_a_v = unsafe { _mm512_setzero_ps() };
    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut norm_b_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY:
        // 1. Feature: Target feature `avx512f` is enabled via function attribute.
        // 2. Length/Tail: `i + 16 <= len` where `len == a.len() == b.len()` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)` point to 16 readable `f32` elements.
        // 3. Alignment: `_mm512_loadu_ps` handles unaligned memory addresses safely.
        // SAFETY: 3. Alignment: `_mm512_loadu_ps` handles unaligned memory addresses safely.
        let (va, vb) = unsafe {
            (
                _mm512_loadu_ps(a.as_ptr().add(i)),
                _mm512_loadu_ps(b.as_ptr().add(i)),
            )
        };

        // SAFETY: Target feature `avx512f` is enabled and vector registers are valid `__m512`.
        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };
        // SAFETY: Target feature `avx512f` is enabled and vector registers are valid `__m512`.
        norm_a_v = unsafe { _mm512_fmadd_ps(va, va, norm_a_v) };
        // SAFETY: Target feature `avx512f` is enabled and vector registers are valid `__m512`.
        norm_b_v = unsafe { _mm512_fmadd_ps(vb, vb, norm_b_v) };

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is enabled and `dot_v`, `norm_a_v`, `norm_b_v` are valid `__m512` vector registers.
    let (mut dot, mut norm_a, mut norm_b) = unsafe {
        (
            hsum512_ps_avx(dot_v),
            hsum512_ps_avx(norm_a_v),
            hsum512_ps_avx(norm_b_v),
        )
    };

    while i < len {
        // SAFETY: `i < len` where `len == a.len() == b.len()` guarantees `i` is within bounds of slices `a` and `b`.
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

/// Calculates Euclidean distance between two float vectors using AVX-512 instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx512f` instruction set extension.
/// 2. `a` and `b` have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut sum_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY:
        // 1. Feature: Target feature `avx512f` is enabled via function attribute.
        // 2. Length/Tail: `i + 16 <= len` where `len == a.len() == b.len()` guarantees 16 readable `f32` elements.
        // 3. Alignment: `_mm512_loadu_ps` handles unaligned memory addresses safely.
        // SAFETY: 3. Alignment: `_mm512_loadu_ps` handles unaligned memory addresses safely.
        let (va, vb) = unsafe {
            (
                _mm512_loadu_ps(a.as_ptr().add(i)),
                _mm512_loadu_ps(b.as_ptr().add(i)),
            )
        };
        // SAFETY: Target feature `avx512f` is enabled and registers are valid `__m512`.
        let diff = unsafe { _mm512_sub_ps(va, vb) };

        // SAFETY: Target feature `avx512f` is enabled and registers are valid `__m512`.
        sum_v = unsafe { _mm512_fmadd_ps(diff, diff, sum_v) };

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is enabled and `sum_v` is a valid `__m512` vector register.
    let mut sum = unsafe { hsum512_ps_avx(sum_v) };

    while i < len {
        // SAFETY: `i < len` where `len == a.len() == b.len()` guarantees `i` is within bounds of both slices.
        let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };
        let diff = x - y;
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Calculates dot product between two float vectors using AVX-512 instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx512f` instruction set extension.
/// 2. `a` and `b` have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn dot_product_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut dot_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY:
        // 1. Feature: Target feature `avx512f` is enabled via function attribute.
        // 2. Length/Tail: `i + 16 <= len` where `len == a.len() == b.len()` guarantees 16 readable `f32` elements.
        // 3. Alignment: `_mm512_loadu_ps` handles unaligned memory addresses safely.
        // SAFETY: 3. Alignment: `_mm512_loadu_ps` handles unaligned memory addresses safely.
        let (va, vb) = unsafe {
            (
                _mm512_loadu_ps(a.as_ptr().add(i)),
                _mm512_loadu_ps(b.as_ptr().add(i)),
            )
        };

        // SAFETY: Target feature `avx512f` is enabled and registers are valid `__m512`.
        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is enabled and `dot_v` is a valid `__m512` vector register.
    let mut dot = unsafe { hsum512_ps_avx(dot_v) };

    while i < len {
        // SAFETY: `i < len` where `len == a.len() == b.len()` guarantees `i` is within bounds of both slices.
        let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };
        dot += x * y;
        i += 1;
    }

    -dot
}

/// Calculates cosine distance between an f32 vector and raw LE f32 bytes using AVX-512 instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx512f` instruction set extension.
/// 2. `b_bytes` contains at least `a.len() * 4` readable bytes.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn cosine_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut dot_v = unsafe { _mm512_setzero_ps() };
    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut norm_a_v = unsafe { _mm512_setzero_ps() };
    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut norm_b_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY:
        // 1. Feature: Target feature `avx512f` is enabled via function attribute.
        // 2. Length/Tail: `i + 16 <= len` and `b_bytes.len() >= a.len() * 4` guarantee 16 readable `f32`s (64 bytes).
        // 3. Alignment: `_mm512_loadu_ps` handles unaligned memory addresses safely.
        // SAFETY: 3. Alignment: `_mm512_loadu_ps` handles unaligned memory addresses safely.
        let (va, vb) = unsafe {
            (
                _mm512_loadu_ps(a.as_ptr().add(i)),
                _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32),
            )
        };

        // SAFETY: Target feature `avx512f` is enabled and registers are valid `__m512`.
        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };
        // SAFETY: Target feature `avx512f` is enabled and registers are valid `__m512`.
        norm_a_v = unsafe { _mm512_fmadd_ps(va, va, norm_a_v) };
        // SAFETY: Target feature `avx512f` is enabled and registers are valid `__m512`.
        norm_b_v = unsafe { _mm512_fmadd_ps(vb, vb, norm_b_v) };

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is enabled and vector registers are valid `__m512`.
    let (mut dot, mut norm_a, mut norm_b) = unsafe {
        (
            hsum512_ps_avx(dot_v),
            hsum512_ps_avx(norm_a_v),
            hsum512_ps_avx(norm_b_v),
        )
    };

    while i < len {
        // SAFETY:
        // 1. Length/Tail: `i < len` and `b_bytes.len() >= a.len() * 4` guarantee indices are within bounds.
        // 2. Alignment: `from_le_bytes` constructs `f32` from unaligned byte slice.
        // SAFETY: 2. Alignment: `from_le_bytes` constructs `f32` from unaligned byte slice.
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

/// Calculates Euclidean distance between an f32 vector and raw LE f32 bytes using AVX-512 instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx512f` instruction set extension.
/// 2. `b_bytes` contains at least `a.len() * 4` readable bytes.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn euclidean_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut sum_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY:
        // 1. Feature: Target feature `avx512f` is enabled via function attribute.
        // 2. Length/Tail: `i + 16 <= len` and `b_bytes.len() >= a.len() * 4` guarantee 16 readable `f32`s (64 bytes).
        // 3. Alignment: `_mm512_loadu_ps` handles unaligned memory addresses safely.
        // SAFETY: 3. Alignment: `_mm512_loadu_ps` handles unaligned memory addresses safely.
        let (va, vb) = unsafe {
            (
                _mm512_loadu_ps(a.as_ptr().add(i)),
                _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32),
            )
        };
        // SAFETY: Target feature `avx512f` is enabled and registers are valid `__m512`.
        let diff = unsafe { _mm512_sub_ps(va, vb) };

        // SAFETY: Target feature `avx512f` is enabled and registers are valid `__m512`.
        sum_v = unsafe { _mm512_fmadd_ps(diff, diff, sum_v) };

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is enabled and `sum_v` is a valid `__m512` vector register.
    let mut sum = unsafe { hsum512_ps_avx(sum_v) };

    while i < len {
        // SAFETY:
        // 1. Length/Tail: `i < len` and `b_bytes.len() >= a.len() * 4` guarantee indices are within bounds.
        // 2. Alignment: `from_le_bytes` constructs `f32` from unaligned byte slice.
        // SAFETY: 2. Alignment: `from_le_bytes` constructs `f32` from unaligned byte slice.
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

/// Calculates dot product between an f32 vector and raw LE f32 bytes using AVX-512 instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx512f` instruction set extension.
/// 2. `b_bytes` contains at least `a.len() * 4` readable bytes.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn dot_product_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut dot_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY:
        // 1. Feature: Target feature `avx512f` is enabled via function attribute.
        // 2. Length/Tail: `i + 16 <= len` and `b_bytes.len() >= a.len() * 4` guarantee readable memory for 16 `f32`s.
        // 3. Alignment: `_mm512_loadu_ps` handles unaligned memory loads safely.
        // SAFETY: 3. Alignment: `_mm512_loadu_ps` handles unaligned memory loads safely.
        let (va, vb) = unsafe {
            (
                _mm512_loadu_ps(a.as_ptr().add(i)),
                _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32),
            )
        };

        // SAFETY: Target feature `avx512f` is enabled and registers are valid `__m512`.
        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };

        i += 16;
    }

    // SAFETY: Target feature `avx512f` is enabled and `dot_v` is a valid `__m512` vector register.
    let mut dot = unsafe { hsum512_ps_avx(dot_v) };

    while i < len {
        // SAFETY:
        // 1. Length/Tail: `i < len` and `b_bytes.len() >= a.len() * 4` guarantee indices are within bounds.
        // 2. Alignment: `from_le_bytes` constructs `f32` from unaligned byte slice.
        // SAFETY: 2. Alignment: `from_le_bytes` constructs `f32` from unaligned byte slice.
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

/// Horizontal sum of 16 x f32 values in a 512-bit AVX-512 vector register.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx512f` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn hsum512_ps_avx(v: __m512) -> f32 {
    // SAFETY: Target feature `avx512f` is enabled. `_mm512_extractf32x8_ps` requires `unsafe` block for intrinsic target feature `avx512dq`.
    let v256 = unsafe { _mm256_add_ps(_mm512_extractf32x8_ps(v, 0), _mm512_extractf32x8_ps(v, 1)) };
    // SAFETY: Target feature `avx512f` is enabled and `v256` is a valid `__m256` register.
    let vlow = unsafe { _mm256_castps256_ps128(v256) };
    // SAFETY: Target feature `avx512f` is enabled and `v256` is a valid `__m256` register.
    let vhigh = unsafe { _mm256_extractf128_ps(v256, 1) };
    // SAFETY: Target feature `avx512f` is enabled and `vlow`, `vhigh` are valid `__m128` registers.
    let v128 = unsafe { _mm_add_ps(vlow, vhigh) };
    // SAFETY: Target feature `avx512f` is enabled and `v128` is a valid `__m128` register.
    let v64 = unsafe { _mm_add_ps(v128, _mm_movehl_ps(v128, v128)) };
    // SAFETY: Target feature `avx512f` is enabled and `v64` is a valid `__m128` register.
    let v32 = unsafe { _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55)) };
    // SAFETY: Target feature `avx512f` is enabled and `v32` contains a valid scalar float at lane 0.
    unsafe { _mm_cvtss_f32(v32) }
}

/// Calculates dot product between two byte vectors using AVX-512 VNNI instructions.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx512f`, `avx512bw`, and `avx512vnni` instruction set extensions.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f", enable = "avx512bw", enable = "avx512vnni")]
pub(crate) unsafe fn dot_product_u8_avx512vnni(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut sum_v = unsafe { _mm512_setzero_si512() };

    while i + 64 <= len {
        // SAFETY:
        // 1. Feature: Target features `avx512f`, `avx512bw`, `avx512vnni` are enabled via function attribute.
        // 2. Length/Tail: `i + 64 <= len` guarantees 64 readable `u8` elements in `a` and `b`.
        // 3. Alignment: `_mm512_loadu_si512` handles unaligned byte loads safely.
        // SAFETY: 3. Alignment: `_mm512_loadu_si512` handles unaligned byte loads safely.
        let (va, vb) = unsafe {
            (
                _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i),
                _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i),
            )
        };

        // SAFETY: Target feature `avx512vnni` is enabled and registers are valid `__m512i`.
        sum_v = unsafe { _mm512_dpbusd_epi32(sum_v, va, vb) };

        i += 64;
    }

    // SAFETY: Target feature `avx512f` is enabled and `sum_v` is a valid `__m512i` vector register.
    let mut sum = u32::from_ne_bytes(unsafe { hsum512_epi32_avx512(sum_v) }.to_ne_bytes());

    while i < len {
        // SAFETY: `i < len` where `len = min(a.len(), b.len())` guarantees `i` is within bounds.
        let (x, y) = unsafe { (*a.get_unchecked(i) as u32, *b.get_unchecked(i) as u32) };
        sum = sum.wrapping_add(x * y);
        i += 1;
    }

    sum
}

/// Calculates squared Euclidean distance between two byte vectors using AVX-512 instructions.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx512f` and `avx512bw` instruction set extensions.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f", enable = "avx512bw")]
pub(crate) unsafe fn euclidean_distance_sq_u8_avx512(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut sum_v = unsafe { _mm512_setzero_si512() };
    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let zero = unsafe { _mm512_setzero_si512() };

    while i + 64 <= len {
        // SAFETY:
        // 1. Feature: Target features `avx512f` and `avx512bw` are enabled via function attribute.
        // 2. Length/Tail: `i + 64 <= len` guarantees 64 readable `u8` elements in `a` and `b`.
        // 3. Alignment: `_mm512_loadu_si512` handles unaligned byte loads safely.
        // SAFETY: 3. Alignment: `_mm512_loadu_si512` handles unaligned byte loads safely.
        let (va, vb) = unsafe {
            (
                _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i),
                _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i),
            )
        };

        // SAFETY: Target feature `avx512bw` is enabled and registers are valid `__m512i`.
        let va_lo = unsafe { _mm512_unpacklo_epi8(va, zero) };
        // SAFETY: Target feature `avx512bw` is enabled and registers are valid `__m512i`.
        let vb_lo = unsafe { _mm512_unpacklo_epi8(vb, zero) };
        // SAFETY: Target feature `avx512bw` is enabled and registers are valid `__m512i`.
        let diff_lo = unsafe { _mm512_sub_epi16(va_lo, vb_lo) };
        // SAFETY: Target feature `avx512bw` is enabled and registers are valid `__m512i`.
        let prod_lo = unsafe { _mm512_madd_epi16(diff_lo, diff_lo) };

        // SAFETY: Target feature `avx512bw` is enabled and registers are valid `__m512i`.
        let va_hi = unsafe { _mm512_unpackhi_epi8(va, zero) };
        // SAFETY: Target feature `avx512bw` is enabled and registers are valid `__m512i`.
        let vb_hi = unsafe { _mm512_unpackhi_epi8(vb, zero) };
        // SAFETY: Target feature `avx512bw` is enabled and registers are valid `__m512i`.
        let diff_hi = unsafe { _mm512_sub_epi16(va_hi, vb_hi) };
        // SAFETY: Target feature `avx512bw` is enabled and registers are valid `__m512i`.
        let prod_hi = unsafe { _mm512_madd_epi16(diff_hi, diff_hi) };

        // SAFETY: Target feature `avx512f` is enabled and registers are valid `__m512i`.
        sum_v = unsafe { _mm512_add_epi32(sum_v, prod_lo) };
        // SAFETY: Target feature `avx512f` is enabled and registers are valid `__m512i`.
        sum_v = unsafe { _mm512_add_epi32(sum_v, prod_hi) };

        i += 64;
    }

    // SAFETY: Target feature `avx512f` is enabled and `sum_v` is a valid `__m512i` vector register.
    let mut sum = u32::from_ne_bytes(unsafe { hsum512_epi32_avx512(sum_v) }.to_ne_bytes());

    while i < len {
        // SAFETY: `i < len` where `len = min(a.len(), b.len())` guarantees `i` is within bounds.
        let diff = unsafe { (*a.get_unchecked(i) as i32) - (*b.get_unchecked(i) as i32) };
        let abs_diff = diff.unsigned_abs();
        sum += abs_diff * abs_diff;
        i += 1;
    }

    sum
}

/// Calculates cosine similarity parts (dot, norm_a_sq, norm_b_sq) for byte vectors using AVX-512 VNNI instructions.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx512f`, `avx512bw`, and `avx512vnni` instruction set extensions.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f", enable = "avx512bw", enable = "avx512vnni")]
pub(crate) unsafe fn cosine_similarity_parts_u8_avx512(
    a: &[u8],
    b: &[u8],
) -> CosineSimilarityPartsU8 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut dot_v = unsafe { _mm512_setzero_si512() };
    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut norm_a_v = unsafe { _mm512_setzero_si512() };
    // SAFETY: Target feature `avx512f` is verified by caller and enabled on function attribute.
    let mut norm_b_v = unsafe { _mm512_setzero_si512() };

    while i + 64 <= len {
        // SAFETY:
        // 1. Feature: Target features `avx512f`, `avx512bw`, `avx512vnni` are enabled via function attribute.
        // 2. Length/Tail: `i + 64 <= len` guarantees 64 readable `u8` elements in `a` and `b`.
        // 3. Alignment: `_mm512_loadu_si512` handles unaligned byte loads safely.
        // SAFETY: 3. Alignment: `_mm512_loadu_si512` handles unaligned byte loads safely.
        let (va, vb) = unsafe {
            (
                _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i),
                _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i),
            )
        };

        // SAFETY: Target feature `avx512vnni` is enabled and registers are valid `__m512i`.
        dot_v = unsafe { _mm512_dpbusd_epi32(dot_v, va, vb) };
        // SAFETY: Target feature `avx512vnni` is enabled and registers are valid `__m512i`.
        norm_a_v = unsafe { _mm512_dpbusd_epi32(norm_a_v, va, va) };
        // SAFETY: Target feature `avx512vnni` is enabled and registers are valid `__m512i`.
        norm_b_v = unsafe { _mm512_dpbusd_epi32(norm_b_v, vb, vb) };

        i += 64;
    }

    // SAFETY: Target feature `avx512f` is enabled and vector registers are valid `__m512i`.
    let (mut dot, mut norm_a_sq, mut norm_b_sq) = unsafe {
        (
            u32::from_ne_bytes(hsum512_epi32_avx512(dot_v).to_ne_bytes()),
            u32::from_ne_bytes(hsum512_epi32_avx512(norm_a_v).to_ne_bytes()),
            u32::from_ne_bytes(hsum512_epi32_avx512(norm_b_v).to_ne_bytes()),
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

/// Horizontal sum of 16 x 32-bit signed integers in a 512-bit AVX-512 vector register.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx512f` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn hsum512_epi32_avx512(v: __m512i) -> i32 {
    // SAFETY: Target feature `avx512f` is enabled. `_mm512_extracti32x8_epi32` requires `unsafe` block for intrinsic target feature `avx512dq`.
    let v256 =
        // SAFETY: Invariants verified for unsafe operations.
        unsafe { _mm256_add_epi32(_mm512_castsi512_si256(v), _mm512_extracti32x8_epi32(v, 1)) };
    // SAFETY: Target feature `avx512f` is enabled and `v256` is a valid `__m256i` register.
    let vlow = unsafe { _mm256_castsi256_si128(v256) };
    // SAFETY: Target feature `avx512f` is enabled and `v256` is a valid `__m256i` register.
    let vhigh = unsafe { _mm256_extracti128_si256(v256, 1) };
    // SAFETY: Target feature `avx512f` is enabled and `vlow`, `vhigh` are valid `__m128i` registers.
    let v128 = unsafe { _mm_add_epi32(vlow, vhigh) };
    // SAFETY: Target feature `avx512f` is enabled and `v128` is a valid `__m128i` register.
    let v64 = unsafe { _mm_add_epi32(v128, _mm_srli_si128(v128, 8)) };
    // SAFETY: Target feature `avx512f` is enabled and `v64` is a valid `__m128i` register.
    let v32 = unsafe { _mm_add_epi32(v64, _mm_srli_si128(v64, 4)) };
    // SAFETY: Target feature `avx512f` is enabled and `v32` contains a valid 32-bit int at lane 0.
    unsafe { _mm_cvtsi128_si32(v32) }
}
