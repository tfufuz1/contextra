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
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: _mm256_setzero_ps requires AVX2 target feature, enabled via #[target_feature(enable = "avx2", enable = "fma")].
    let mut dot_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: _mm256_setzero_ps requires AVX2 target feature, enabled via #[target_feature(enable = "avx2", enable = "fma")].
    let mut norm_a_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: _mm256_setzero_ps requires AVX2 target feature, enabled via #[target_feature(enable = "avx2", enable = "fma")].
    let mut norm_b_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: `i + 8 <= len` guarantees `a.as_ptr().add(i)` points to 8 readable `f32` elements within `a`'s bounds. `_mm256_loadu_ps` handles unaligned memory. Target feature `avx2` is enabled.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: `a.len() == b.len()` (caller precondition) and `i + 8 <= len` guarantee `b.as_ptr().add(i)` points to 8 readable `f32` elements within `b`'s bounds. `_mm256_loadu_ps` supports unaligned reads. Target feature `avx2` is enabled.
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(i)) };

        // SAFETY: Target features `avx2` and `fma` are enabled. Inputs `va`, `vb`, `dot_v` are valid 256-bit AVX2 vector registers.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };
        // SAFETY: Target features `avx2` and `fma` are enabled. Inputs `va`, `norm_a_v` are valid 256-bit AVX2 vector registers.
        norm_a_v = unsafe { _mm256_fmadd_ps(va, va, norm_a_v) };
        // SAFETY: Target features `avx2` and `fma` are enabled. Inputs `vb`, `norm_b_v` are valid 256-bit AVX2 vector registers.
        norm_b_v = unsafe { _mm256_fmadd_ps(vb, vb, norm_b_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `dot_v` is a valid `__m256` vector register.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };
    // SAFETY: Target feature `avx2` is enabled and `norm_a_v` is a valid `__m256` vector register.
    let mut norm_a = unsafe { hsum256_ps_avx(norm_a_v) };
    // SAFETY: Target feature `avx2` is enabled and `norm_b_v` is a valid `__m256` vector register.
    let mut norm_b = unsafe { hsum256_ps_avx(norm_b_v) };

    while i < len {
        // SAFETY: `i < len` where `len == a.len()` guarantees `i` is within bounds of slice `a`.
        let x = unsafe { *a.get_unchecked(i) };
        // SAFETY: `i < len` where `len == b.len()` (caller precondition `a.len() == b.len()`) guarantees `i` is within bounds of slice `b`.
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

/// Calculates Euclidean distance between two float vectors using AVX2 and FMA instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx2` and `fma` instruction set extensions.
/// 2. `a` and `b` have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut sum_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: `i + 8 <= len` guarantees `a.as_ptr().add(i)` points to 8 readable `f32` elements within `a`'s bounds. `_mm256_loadu_ps` handles unaligned memory. Target feature `avx2` is enabled.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: `a.len() == b.len()` (caller precondition) and `i + 8 <= len` guarantee `b.as_ptr().add(i)` points to 8 readable `f32` elements within `b`'s bounds. `_mm256_loadu_ps` supports unaligned reads. Target feature `avx2` is enabled.
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(i)) };
        // SAFETY: Target feature `avx2` is enabled. Inputs `va` and `vb` are valid `__m256` vector registers.
        let diff = unsafe { _mm256_sub_ps(va, vb) };

        // SAFETY: Target features `avx2` and `fma` are enabled. Inputs `diff` and `sum_v` are valid `__m256` vector registers.
        sum_v = unsafe { _mm256_fmadd_ps(diff, diff, sum_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `sum_v` is a valid `__m256` vector register.
    let mut sum = unsafe { hsum256_ps_avx(sum_v) };

    while i < len {
        // SAFETY: `i < len` where `len == a.len() == b.len()` guarantees `i` is within bounds of both slices `a` and `b`.
        let diff = unsafe { *a.get_unchecked(i) - *b.get_unchecked(i) };
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
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut dot_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: `i + 8 <= len` guarantees `a.as_ptr().add(i)` points to 8 readable `f32` elements within `a`'s bounds. `_mm256_loadu_ps` handles unaligned memory. Target feature `avx2` is enabled.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: `a.len() == b.len()` (caller precondition) and `i + 8 <= len` guarantee `b.as_ptr().add(i)` points to 8 readable `f32` elements within `b`'s bounds. `_mm256_loadu_ps` supports unaligned reads. Target feature `avx2` is enabled.
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(i)) };

        // SAFETY: Target features `avx2` and `fma` are enabled. Inputs `va`, `vb`, `dot_v` are valid 256-bit AVX2 vector registers.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `dot_v` is a valid `__m256` vector register.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };

    while i < len {
        // SAFETY: `i < len` where `len == a.len() == b.len()` guarantees `i` is within bounds of both slices `a` and `b`.
        dot += unsafe { *a.get_unchecked(i) * *b.get_unchecked(i) };
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
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn cosine_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut dot_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut norm_a_v = unsafe { _mm256_setzero_ps() };
    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut norm_b_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: `i + 8 <= len` guarantees `a.as_ptr().add(i)` points to 8 readable `f32` elements within `a`'s bounds. `_mm256_loadu_ps` handles unaligned memory. Target feature `avx2` is enabled.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i + 8 <= len` guarantee `b_bytes.as_ptr().add(i * 4)` has at least 32 readable bytes (8 `f32`s). `_mm256_loadu_ps` supports unaligned memory reads from raw pointers. Target feature `avx2` is enabled.
        let vb = unsafe { _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };

        // SAFETY: Target features `avx2` and `fma` are enabled. Inputs `va`, `vb`, `dot_v` are valid 256-bit vector registers.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };
        // SAFETY: Target features `avx2` and `fma` are enabled. Inputs `va`, `norm_a_v` are valid 256-bit vector registers.
        norm_a_v = unsafe { _mm256_fmadd_ps(va, va, norm_a_v) };
        // SAFETY: Target features `avx2` and `fma` are enabled. Inputs `vb`, `norm_b_v` are valid 256-bit vector registers.
        norm_b_v = unsafe { _mm256_fmadd_ps(vb, vb, norm_b_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `dot_v` is a valid `__m256` vector register.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };
    // SAFETY: Target feature `avx2` is enabled and `norm_a_v` is a valid `__m256` vector register.
    let mut norm_a = unsafe { hsum256_ps_avx(norm_a_v) };
    // SAFETY: Target feature `avx2` is enabled and `norm_b_v` is a valid `__m256` vector register.
    let mut norm_b = unsafe { hsum256_ps_avx(norm_b_v) };

    while i < len {
        // SAFETY: `i < len` guarantees `i` is within bounds of slice `a`.
        let x = unsafe { *a.get_unchecked(i) };
        let b_val = f32::from_le_bytes([
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4` is within bounds of `b_bytes`.
            unsafe { *b_bytes.get_unchecked(i * 4) },
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4 + 1` is within bounds of `b_bytes`.
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4 + 2` is within bounds of `b_bytes`.
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4 + 3` is within bounds of `b_bytes`.
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

/// Calculates Euclidean distance between an f32 vector and raw LE f32 bytes using AVX2 and FMA instructions.
///
/// # Safety
/// Caller must ensure that:
/// 1. Target CPU supports `avx2` and `fma` instruction set extensions.
/// 2. `b_bytes` contains at least `a.len() * 4` readable bytes.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn euclidean_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut sum_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: `i + 8 <= len` guarantees `a.as_ptr().add(i)` points to 8 readable `f32` elements within `a`'s bounds. `_mm256_loadu_ps` handles unaligned memory. Target feature `avx2` is enabled.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i + 8 <= len` guarantee `b_bytes.as_ptr().add(i * 4)` has at least 32 readable bytes (8 `f32`s). `_mm256_loadu_ps` supports unaligned memory reads from raw pointers. Target feature `avx2` is enabled.
        let vb = unsafe { _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };
        // SAFETY: Target feature `avx2` is enabled. Inputs `va` and `vb` are valid `__m256` vector registers.
        let diff = unsafe { _mm256_sub_ps(va, vb) };

        // SAFETY: Target features `avx2` and `fma` are enabled. Inputs `diff` and `sum_v` are valid `__m256` vector registers.
        sum_v = unsafe { _mm256_fmadd_ps(diff, diff, sum_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `sum_v` is a valid `__m256` vector register.
    let mut sum = unsafe { hsum256_ps_avx(sum_v) };

    while i < len {
        let b_val = f32::from_le_bytes([
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4` is within bounds of `b_bytes`.
            unsafe { *b_bytes.get_unchecked(i * 4) },
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4 + 1` is within bounds of `b_bytes`.
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4 + 2` is within bounds of `b_bytes`.
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4 + 3` is within bounds of `b_bytes`.
            unsafe { *b_bytes.get_unchecked(i * 4 + 3) },
        ]);
        // SAFETY: `i < len` guarantees `i` is within bounds of slice `a`.
        let diff = unsafe { *a.get_unchecked(i) } - b_val;
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
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub(crate) unsafe fn dot_product_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut dot_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        // SAFETY: `i + 8 <= len` guarantees `a.as_ptr().add(i)` points to 8 readable `f32` elements within `a`'s bounds. `_mm256_loadu_ps` handles unaligned memory. Target feature `avx2` is enabled.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i + 8 <= len` guarantee `b_bytes.as_ptr().add(i * 4)` has at least 32 readable bytes (8 `f32`s). `_mm256_loadu_ps` supports unaligned memory reads from raw pointers. Target feature `avx2` is enabled.
        let vb = unsafe { _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };

        // SAFETY: Target features `avx2` and `fma` are enabled. Inputs `va`, `vb`, `dot_v` are valid 256-bit vector registers.
        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };

        i += 8;
    }

    // SAFETY: Target feature `avx2` is enabled and `dot_v` is a valid `__m256` vector register.
    let mut dot = unsafe { hsum256_ps_avx(dot_v) };

    while i < len {
        let b_val = f32::from_le_bytes([
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4` is within bounds of `b_bytes`.
            unsafe { *b_bytes.get_unchecked(i * 4) },
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4 + 1` is within bounds of `b_bytes`.
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4 + 2` is within bounds of `b_bytes`.
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
            // SAFETY: `b_bytes.len() >= a.len() * 4` (caller precondition) and `i < len` guarantee `i * 4 + 3` is within bounds of `b_bytes`.
            unsafe { *b_bytes.get_unchecked(i * 4 + 3) },
        ]);
        // SAFETY: `i < len` guarantees `i` is within bounds of slice `a`.
        dot += unsafe { *a.get_unchecked(i) } * b_val;
        i += 1;
    }

    -dot
}

/// Horizontal sum of 8 x f32 values in a 256-bit AVX vector register.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx2` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn hsum256_ps_avx(v: __m256) -> f32 {
    // SAFETY: Target feature `avx2` is enabled and `v` is a valid `__m256` vector register.
    let vlow = unsafe { _mm256_castps256_ps128(v) };
    // SAFETY: Target feature `avx2` is enabled and imm8 index 1 is in-bounds for 256-bit split into 128-bit halves.
    let vhigh = unsafe { _mm256_extractf128_ps(v, 1) };
    // SAFETY: Target feature `avx2` (SSE/AVX) is enabled and `vlow`, `vhigh` are valid `__m128` vector registers.
    let v128 = unsafe { _mm_add_ps(vlow, vhigh) };
    // SAFETY: Target feature `avx2` (SSE/AVX) is enabled and `_mm_movehl_ps` unpacks high 64 bits from `v128`.
    let v64 = unsafe { _mm_add_ps(v128, _mm_movehl_ps(v128, v128)) };
    // SAFETY: Target feature `avx2` (SSE/AVX) is enabled and immediate `0x55` selects elements for shuffle.
    let v32 = unsafe { _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55)) };
    // SAFETY: Target feature `avx2` (SSE/AVX) is enabled and `v32` contains a valid scalar float at index 0.
    unsafe { _mm_cvtss_f32(v32) }
}

/// Calculates dot product between two byte vectors using AVX2 instructions.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx2` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn dot_product_u8_avx2(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut sum_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        // SAFETY: `i + 32 <= len` where `len = min(a.len(), b.len())` guarantees `a.as_ptr().add(i)` points to 32 readable `u8` elements within `a`'s bounds. `_mm256_loadu_si256` handles unaligned reads from pointer. Target feature `avx2` is enabled.
        let va = unsafe { _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i) };
        // SAFETY: `i + 32 <= len` where `len = min(a.len(), b.len())` guarantees `b.as_ptr().add(i)` points to 32 readable `u8` elements within `b`'s bounds. `_mm256_loadu_si256` handles unaligned reads from pointer. Target feature `avx2` is enabled.
        let vb = unsafe { _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i) };

        // SAFETY: Target feature `avx2` is enabled and `va`, `zero` are valid `__m256i` vector registers.
        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and `vb`, `zero` are valid `__m256i` vector registers.
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled and `va_lo`, `vb_lo` are valid `__m256i` vector registers.
        let prod_lo = unsafe { _mm256_madd_epi16(va_lo, vb_lo) };

        // SAFETY: Target feature `avx2` is enabled and `va`, `zero` are valid `__m256i` vector registers.
        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and `vb`, `zero` are valid `__m256i` vector registers.
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled and `va_hi`, `vb_hi` are valid `__m256i` vector registers.
        let prod_hi = unsafe { _mm256_madd_epi16(va_hi, vb_hi) };

        // SAFETY: Target feature `avx2` is enabled and `sum_v`, `prod_lo` are valid `__m256i` vector registers.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_lo) };
        // SAFETY: Target feature `avx2` is enabled and `sum_v`, `prod_hi` are valid `__m256i` vector registers.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_hi) };

        i += 32;
    }

    // SAFETY: Target feature `avx2` is enabled and `sum_v` is a valid `__m256i` vector register.
    let mut sum = unsafe { hsum256_epi32_avx2(sum_v) } as u32;

    while i < len {
        // SAFETY: `i < len` where `len = min(a.len(), b.len())` guarantees `i` is within bounds of both slices `a` and `b`.
        sum += unsafe { (*a.get_unchecked(i) as u32) * (*b.get_unchecked(i) as u32) };
        i += 1;
    }

    sum
}

/// Calculates squared Euclidean distance between two byte vectors using AVX2 instructions.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx2` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn euclidean_distance_sq_u8_avx2(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut sum_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        // SAFETY: `i + 32 <= len` where `len = min(a.len(), b.len())` guarantees `a.as_ptr().add(i)` points to 32 readable `u8` elements within `a`'s bounds. `_mm256_loadu_si256` handles unaligned reads from pointer. Target feature `avx2` is enabled.
        let va = unsafe { _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i) };
        // SAFETY: `i + 32 <= len` where `len = min(a.len(), b.len())` guarantees `b.as_ptr().add(i)` points to 32 readable `u8` elements within `b`'s bounds. `_mm256_loadu_si256` handles unaligned reads from pointer. Target feature `avx2` is enabled.
        let vb = unsafe { _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i) };

        // SAFETY: Target feature `avx2` is enabled and `va`, `zero` are valid `__m256i` vector registers.
        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and `vb`, `zero` are valid `__m256i` vector registers.
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled and `va_lo`, `vb_lo` are valid `__m256i` vector registers.
        let diff_lo = unsafe { _mm256_sub_epi16(va_lo, vb_lo) };
        // SAFETY: Target feature `avx2` is enabled and `diff_lo` is a valid `__m256i` vector register.
        let prod_lo = unsafe { _mm256_madd_epi16(diff_lo, diff_lo) };

        // SAFETY: Target feature `avx2` is enabled and `va`, `zero` are valid `__m256i` vector registers.
        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and `vb`, `zero` are valid `__m256i` vector registers.
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled and `va_hi`, `vb_hi` are valid `__m256i` vector registers.
        let diff_hi = unsafe { _mm256_sub_epi16(va_hi, vb_hi) };
        // SAFETY: Target feature `avx2` is enabled and `diff_hi` is a valid `__m256i` vector register.
        let prod_hi = unsafe { _mm256_madd_epi16(diff_hi, diff_hi) };

        // SAFETY: Target feature `avx2` is enabled and `sum_v`, `prod_lo` are valid `__m256i` vector registers.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_lo) };
        // SAFETY: Target feature `avx2` is enabled and `sum_v`, `prod_hi` are valid `__m256i` vector registers.
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_hi) };

        i += 32;
    }

    // SAFETY: Target feature `avx2` is enabled and `sum_v` is a valid `__m256i` vector register.
    let mut sum = unsafe { hsum256_epi32_avx2(sum_v) } as u32;

    while i < len {
        // SAFETY: `i < len` where `len = min(a.len(), b.len())` guarantees `i` is within bounds of both slices `a` and `b`.
        let diff = unsafe { (*a.get_unchecked(i) as i32) - (*b.get_unchecked(i) as i32) };
        sum += (diff * diff) as u32;
        i += 1;
    }

    sum
}

/// Calculates cosine similarity parts (dot, norm_a_sq, norm_b_sq) for byte vectors using AVX2 instructions.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx2` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn cosine_similarity_parts_u8_avx2(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut dot_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut norm_a_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let mut norm_b_v = unsafe { _mm256_setzero_si256() };
    // SAFETY: Target feature `avx2` is enabled via function target attribute.
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        // SAFETY: `i + 32 <= len` where `len = min(a.len(), b.len())` guarantees `a.as_ptr().add(i)` points to 32 readable `u8` elements within `a`'s bounds. `_mm256_loadu_si256` handles unaligned reads from pointer. Target feature `avx2` is enabled.
        let va = unsafe { _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i) };
        // SAFETY: `i + 32 <= len` where `len = min(a.len(), b.len())` guarantees `b.as_ptr().add(i)` points to 32 readable `u8` elements within `b`'s bounds. `_mm256_loadu_si256` handles unaligned reads from pointer. Target feature `avx2` is enabled.
        let vb = unsafe { _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i) };

        // SAFETY: Target feature `avx2` is enabled and `va`, `zero` are valid `__m256i` vector registers.
        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and `vb`, `zero` are valid `__m256i` vector registers.
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled. `dot_v`, `va_lo`, `vb_lo` are valid `__m256i` vector registers.
        dot_v = unsafe { _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_lo, vb_lo)) };
        // SAFETY: Target feature `avx2` is enabled. `norm_a_v`, `va_lo` are valid `__m256i` vector registers.
        norm_a_v = unsafe { _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_lo, va_lo)) };
        // SAFETY: Target feature `avx2` is enabled. `norm_b_v`, `vb_lo` are valid `__m256i` vector registers.
        norm_b_v = unsafe { _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_lo, vb_lo)) };

        // SAFETY: Target feature `avx2` is enabled and `va`, `zero` are valid `__m256i` vector registers.
        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        // SAFETY: Target feature `avx2` is enabled and `vb`, `zero` are valid `__m256i` vector registers.
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        // SAFETY: Target feature `avx2` is enabled. `dot_v`, `va_hi`, `vb_hi` are valid `__m256i` vector registers.
        dot_v = unsafe { _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_hi, vb_hi)) };
        // SAFETY: Target feature `avx2` is enabled. `norm_a_v`, `va_hi` are valid `__m256i` vector registers.
        norm_a_v = unsafe { _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_hi, va_hi)) };
        // SAFETY: Target feature `avx2` is enabled. `norm_b_v`, `vb_hi` are valid `__m256i` vector registers.
        norm_b_v = unsafe { _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_hi, vb_hi)) };

        i += 32;
    }

    // SAFETY: Target feature `avx2` is enabled and `dot_v` is a valid `__m256i` vector register.
    let mut dot = unsafe { hsum256_epi32_avx2(dot_v) } as u32;
    // SAFETY: Target feature `avx2` is enabled and `norm_a_v` is a valid `__m256i` vector register.
    let mut norm_a_sq = unsafe { hsum256_epi32_avx2(norm_a_v) } as u32;
    // SAFETY: Target feature `avx2` is enabled and `norm_b_v` is a valid `__m256i` vector register.
    let mut norm_b_sq = unsafe { hsum256_epi32_avx2(norm_b_v) } as u32;

    while i < len {
        // SAFETY: `i < len` where `len = min(a.len(), b.len())` guarantees `i` is within bounds of slice `a`.
        let x = unsafe { *a.get_unchecked(i) as u32 };
        // SAFETY: `i < len` where `len = min(a.len(), b.len())` guarantees `i` is within bounds of slice `b`.
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

/// Horizontal sum of 8 x 32-bit signed integers in a 256-bit AVX2 vector register.
///
/// # Safety
/// Caller must ensure that target CPU supports `avx2` instruction set extension.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn hsum256_epi32_avx2(v: __m256i) -> i32 {
    // SAFETY: Target feature `avx2` is enabled and `v` is a valid `__m256i` vector register.
    let vlow = unsafe { _mm256_castsi256_si128(v) };
    // SAFETY: Target feature `avx2` is enabled and imm8 index 1 is in-bounds for 256-bit split into 128-bit halves.
    let vhigh = unsafe { _mm256_extracti128_si256(v, 1) };
    // SAFETY: Target feature `avx2` (SSE/AVX) is enabled and `vlow`, `vhigh` are valid `__m128i` vector registers.
    let v128 = unsafe { _mm_add_epi32(vlow, vhigh) };
    // SAFETY: Target feature `avx2` (SSE/AVX) is enabled and `_mm_srli_si128` shifts 128-bit vector right by 8 bytes.
    let v64 = unsafe { _mm_add_epi32(v128, _mm_srli_si128(v128, 8)) };
    // SAFETY: Target feature `avx2` (SSE/AVX) is enabled and `_mm_srli_si128` shifts 128-bit vector right by 4 bytes.
    let v32 = unsafe { _mm_add_epi32(v64, _mm_srli_si128(v64, 4)) };
    // SAFETY: Target feature `avx2` (SSE/AVX) is enabled and `v32` contains a valid 32-bit integer at lane 0.
    unsafe { _mm_cvtsi128_si32(v32) }
}
