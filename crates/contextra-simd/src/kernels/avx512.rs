// FILE-CONTEXT
// STAND: 2026-10-06
// ZWECK: AVX-512 SIMD-Intrinsics für f32, f32_bytes und u8 VNNI Distanzberechnungen.
// INVARIANTEN: Target-Features (avx512f, avx512bw, avx512vnni) per caller/dispatcher garantiert; Slice-Längen in-bounds.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::kernels::scalar::CosineSimilarityPartsU8;

/// Calculates cosine distance between two `f32` slices using AVX-512 instructions.
///
/// # Safety
///
/// - The caller must guarantee that the host CPU supports `avx512f` (e.g. checked via `is_x86_feature_detected!`).
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
// SAFETY: Host CPU support for avx512f guaranteed by caller; equal slice lengths required.
pub(crate) unsafe fn cosine_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature avx512f enabled on function; zero vector initialization has no memory hazards.
    let mut dot_v = unsafe { _mm512_setzero_ps() };
    // SAFETY: Target feature avx512f enabled on function; zero vector initialization has no memory hazards.
    let mut norm_a_v = unsafe { _mm512_setzero_ps() };
    // SAFETY: Target feature avx512f enabled on function; zero vector initialization has no memory hazards.
    let mut norm_b_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY: Loop condition i + 16 <= len guarantees a[i..i+16] is in-bounds. _mm512_loadu_ps supports unaligned reads.
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 16 <= len guarantees b[i..i+16] is in-bounds (a.len() == b.len()). _mm512_loadu_ps supports unaligned reads.
        let vb = unsafe { _mm512_loadu_ps(b.as_ptr().add(i)) };

        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };
        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        norm_a_v = unsafe { _mm512_fmadd_ps(va, va, norm_a_v) };
        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        norm_b_v = unsafe { _mm512_fmadd_ps(vb, vb, norm_b_v) };

        i += 16;
    }

    // SAFETY: Target feature avx512f enabled on function; dot_v is a valid __m512 vector.
    let mut dot = unsafe { hsum512_ps_avx(dot_v) };
    // SAFETY: Target feature avx512f enabled on function; norm_a_v is a valid __m512 vector.
    let mut norm_a = unsafe { hsum512_ps_avx(norm_a_v) };
    // SAFETY: Target feature avx512f enabled on function; norm_b_v is a valid __m512 vector.
    let mut norm_b = unsafe { hsum512_ps_avx(norm_b_v) };

    while i < len {
        // SAFETY: Loop condition i < len guarantees index i is within slice bounds.
        let x = unsafe { *a.get_unchecked(i) };
        // SAFETY: Loop condition i < len guarantees index i is within slice bounds (a.len() == b.len()).
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

/// Calculates Euclidean distance between two `f32` slices using AVX-512 instructions.
///
/// # Safety
///
/// - The caller must guarantee that the host CPU supports `avx512f` (e.g. checked via `is_x86_feature_detected!`).
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
// SAFETY: Host CPU support for avx512f guaranteed by caller; equal slice lengths required.
pub(crate) unsafe fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature avx512f enabled on function; zero vector initialization has no memory hazards.
    let mut sum_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY: Loop condition i + 16 <= len guarantees a[i..i+16] is in-bounds. _mm512_loadu_ps supports unaligned reads.
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 16 <= len guarantees b[i..i+16] is in-bounds (a.len() == b.len()). _mm512_loadu_ps supports unaligned reads.
        let vb = unsafe { _mm512_loadu_ps(b.as_ptr().add(i)) };
        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        let diff = unsafe { _mm512_sub_ps(va, vb) };

        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        sum_v = unsafe { _mm512_fmadd_ps(diff, diff, sum_v) };

        i += 16;
    }

    // SAFETY: Target feature avx512f enabled on function; sum_v is a valid __m512 vector.
    let mut sum = unsafe { hsum512_ps_avx(sum_v) };

    while i < len {
        // SAFETY: Loop condition i < len guarantees index i is within bounds for both a and b (a.len() == b.len()).
        let diff = unsafe { *a.get_unchecked(i) - *b.get_unchecked(i) };
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Calculates dot product between two `f32` slices using AVX-512 instructions (returns negated sum).
///
/// # Safety
///
/// - The caller must guarantee that the host CPU supports `avx512f` (e.g. checked via `is_x86_feature_detected!`).
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
// SAFETY: Host CPU support for avx512f guaranteed by caller; equal slice lengths required.
pub(crate) unsafe fn dot_product_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature avx512f enabled on function; zero vector initialization has no memory hazards.
    let mut dot_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY: Loop condition i + 16 <= len guarantees a[i..i+16] is in-bounds. _mm512_loadu_ps supports unaligned reads.
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 16 <= len guarantees b[i..i+16] is in-bounds (a.len() == b.len()). _mm512_loadu_ps supports unaligned reads.
        let vb = unsafe { _mm512_loadu_ps(b.as_ptr().add(i)) };

        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };

        i += 16;
    }

    // SAFETY: Target feature avx512f enabled on function; dot_v is a valid __m512 vector.
    let mut dot = unsafe { hsum512_ps_avx(dot_v) };

    while i < len {
        // SAFETY: Loop condition i < len guarantees index i is within bounds for both a and b (a.len() == b.len()).
        dot += unsafe { *a.get_unchecked(i) * *b.get_unchecked(i) };
        i += 1;
    }

    -dot
}

/// Calculates cosine distance between a `f32` slice and byte-encoded `f32` slice using AVX-512.
///
/// # Safety
///
/// - The caller must guarantee that the host CPU supports `avx512f` (e.g. checked via `is_x86_feature_detected!`).
/// - Byte slice `b_bytes` must have length at least `a.len() * 4`.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
// SAFETY: Host CPU support for avx512f guaranteed by caller; b_bytes.len() >= a.len() * 4 required.
pub(crate) unsafe fn cosine_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature avx512f enabled on function; zero vector initialization has no memory hazards.
    let mut dot_v = unsafe { _mm512_setzero_ps() };
    // SAFETY: Target feature avx512f enabled on function; zero vector initialization has no memory hazards.
    let mut norm_a_v = unsafe { _mm512_setzero_ps() };
    // SAFETY: Target feature avx512f enabled on function; zero vector initialization has no memory hazards.
    let mut norm_b_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY: Loop condition i + 16 <= len guarantees a[i..i+16] is in-bounds. _mm512_loadu_ps supports unaligned reads.
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 16 <= len with b_bytes.len() >= len * 4 guarantees (i+16)*4 <= b_bytes.len(). _mm512_loadu_ps supports unaligned float pointer reads.
        let vb = unsafe { _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };

        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };
        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        norm_a_v = unsafe { _mm512_fmadd_ps(va, va, norm_a_v) };
        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        norm_b_v = unsafe { _mm512_fmadd_ps(vb, vb, norm_b_v) };

        i += 16;
    }

    // SAFETY: Target feature avx512f enabled on function; dot_v is a valid __m512 vector.
    let mut dot = unsafe { hsum512_ps_avx(dot_v) };
    // SAFETY: Target feature avx512f enabled on function; norm_a_v is a valid __m512 vector.
    let mut norm_a = unsafe { hsum512_ps_avx(norm_a_v) };
    // SAFETY: Target feature avx512f enabled on function; norm_b_v is a valid __m512 vector.
    let mut norm_b = unsafe { hsum512_ps_avx(norm_b_v) };

    while i < len {
        // SAFETY: Loop condition i < len guarantees index i is in-bounds for a.
        let x = unsafe { *a.get_unchecked(i) };
        let b_val = f32::from_le_bytes([
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4 < b_bytes.len().
            unsafe { *b_bytes.get_unchecked(i * 4) },
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4+1 < b_bytes.len().
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4+2 < b_bytes.len().
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4+3 < b_bytes.len().
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

/// Calculates Euclidean distance between a `f32` slice and byte-encoded `f32` slice using AVX-512.
///
/// # Safety
///
/// - The caller must guarantee that the host CPU supports `avx512f` (e.g. checked via `is_x86_feature_detected!`).
/// - Byte slice `b_bytes` must have length at least `a.len() * 4`.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
// SAFETY: Host CPU support for avx512f guaranteed by caller; b_bytes.len() >= a.len() * 4 required.
pub(crate) unsafe fn euclidean_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature avx512f enabled on function; zero vector initialization has no memory hazards.
    let mut sum_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY: Loop condition i + 16 <= len guarantees a[i..i+16] is in-bounds. _mm512_loadu_ps supports unaligned reads.
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 16 <= len with b_bytes.len() >= len * 4 guarantees (i+16)*4 <= b_bytes.len(). _mm512_loadu_ps supports unaligned float pointer reads.
        let vb = unsafe { _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };
        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        let diff = unsafe { _mm512_sub_ps(va, vb) };

        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        sum_v = unsafe { _mm512_fmadd_ps(diff, diff, sum_v) };

        i += 16;
    }

    // SAFETY: Target feature avx512f enabled on function; sum_v is a valid __m512 vector.
    let mut sum = unsafe { hsum512_ps_avx(sum_v) };

    while i < len {
        let b_val = f32::from_le_bytes([
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4 < b_bytes.len().
            unsafe { *b_bytes.get_unchecked(i * 4) },
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4+1 < b_bytes.len().
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4+2 < b_bytes.len().
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4+3 < b_bytes.len().
            unsafe { *b_bytes.get_unchecked(i * 4 + 3) },
        ]);
        // SAFETY: Loop condition i < len guarantees index i is in-bounds for a.
        let diff = unsafe { *a.get_unchecked(i) } - b_val;
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Calculates dot product between a `f32` slice and byte-encoded `f32` slice using AVX-512 (returns negated sum).
///
/// # Safety
///
/// - The caller must guarantee that the host CPU supports `avx512f` (e.g. checked via `is_x86_feature_detected!`).
/// - Byte slice `b_bytes` must have length at least `a.len() * 4`.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
// SAFETY: Host CPU support for avx512f guaranteed by caller; b_bytes.len() >= a.len() * 4 required.
pub(crate) unsafe fn dot_product_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Target feature avx512f enabled on function; zero vector initialization has no memory hazards.
    let mut dot_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        // SAFETY: Loop condition i + 16 <= len guarantees a[i..i+16] is in-bounds. _mm512_loadu_ps supports unaligned reads.
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 16 <= len with b_bytes.len() >= len * 4 guarantees (i+16)*4 <= b_bytes.len(). _mm512_loadu_ps supports unaligned float pointer reads.
        let vb = unsafe { _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };

        // SAFETY: Target feature avx512f enabled on function; operates on valid register values.
        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };

        i += 16;
    }

    // SAFETY: Target feature avx512f enabled on function; dot_v is a valid __m512 vector.
    let mut dot = unsafe { hsum512_ps_avx(dot_v) };

    while i < len {
        let b_val = f32::from_le_bytes([
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4 < b_bytes.len().
            unsafe { *b_bytes.get_unchecked(i * 4) },
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4+1 < b_bytes.len().
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4+2 < b_bytes.len().
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
            // SAFETY: Loop condition i < len with b_bytes.len() >= len * 4 guarantees i*4+3 < b_bytes.len().
            unsafe { *b_bytes.get_unchecked(i * 4 + 3) },
        ]);
        // SAFETY: Loop condition i < len guarantees index i is in-bounds for a.
        dot += unsafe { *a.get_unchecked(i) } * b_val;
        i += 1;
    }

    -dot
}

/// Horizontally sums all 16 `f32` components in a 512-bit vector.
///
/// # Safety
///
/// - The caller must guarantee that the host CPU supports `avx512f` (e.g. checked via `is_x86_feature_detected!`).
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
// SAFETY: Host CPU support for avx512f guaranteed by caller.
pub(crate) unsafe fn hsum512_ps_avx(v: __m512) -> f32 {
    // SAFETY: Target feature avx512f enabled on function; extracts and adds valid 256-bit halves of __m512 vector.
    let v256 = unsafe { _mm256_add_ps(_mm512_extractf32x8_ps(v, 0), _mm512_extractf32x8_ps(v, 1)) };
    // SAFETY: Target feature avx512f enabled; valid 256-bit to 128-bit vector cast.
    let vlow = unsafe { _mm256_castps256_ps128(v256) };
    // SAFETY: Target feature avx512f enabled; extracts upper 128-bit lane from valid __m256 vector.
    let vhigh = unsafe { _mm256_extractf128_ps(v256, 1) };
    // SAFETY: Target feature avx512f enabled; adds valid 128-bit float vectors.
    let v128 = unsafe { _mm_add_ps(vlow, vhigh) };
    // SAFETY: Target feature avx512f enabled; moves high 64 bits to low and adds.
    let v64 = unsafe { _mm_add_ps(v128, _mm_movehl_ps(v128, v128)) };
    // SAFETY: Target feature avx512f enabled; shuffles second 32-bit element and adds low scalar.
    let v32 = unsafe { _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55)) };
    // SAFETY: Target feature avx512f enabled; extracts scalar f32 from low element of valid __m128.
    unsafe { _mm_cvtss_f32(v32) }
}

/// Calculates dot product of two `u8` byte slices using AVX-512 VNNI (`dpbusd`).
///
/// # Safety
///
/// - The caller must guarantee that the host CPU supports `avx512f`, `avx512bw`, and `avx512vnni` (e.g. checked via `is_x86_feature_detected!`).
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f", enable = "avx512bw", enable = "avx512vnni")]
// SAFETY: Host CPU support for avx512f, avx512bw, and avx512vnni guaranteed by caller.
pub(crate) unsafe fn dot_product_u8_avx512vnni(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target features avx512f/bw/vnni enabled on function; zero vector initialization has no memory hazards.
    let mut sum_v = unsafe { _mm512_setzero_si512() };
    // SAFETY: Target features avx512f/bw/vnni enabled on function; zero vector initialization has no memory hazards.
    let zero = unsafe { _mm512_setzero_si512() };

    while i + 64 <= len {
        // SAFETY: Loop condition i + 64 <= len guarantees a[i..i+64] is in-bounds. _mm512_loadu_si512 supports unaligned reads.
        let va = unsafe { _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i) };
        // SAFETY: Loop condition i + 64 <= len guarantees b[i..i+64] is in-bounds. _mm512_loadu_si512 supports unaligned reads.
        let vb = unsafe { _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i) };

        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        let va_lo = unsafe { _mm512_unpacklo_epi8(va, zero) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        let vb_lo = unsafe { _mm512_unpacklo_epi8(vb, zero) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        let prod_lo = unsafe { _mm512_madd_epi16(va_lo, vb_lo) };

        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        let va_hi = unsafe { _mm512_unpackhi_epi8(va, zero) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        let vb_hi = unsafe { _mm512_unpackhi_epi8(vb, zero) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        let prod_hi = unsafe { _mm512_madd_epi16(va_hi, vb_hi) };

        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        sum_v = unsafe { _mm512_add_epi32(sum_v, prod_lo) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        sum_v = unsafe { _mm512_add_epi32(sum_v, prod_hi) };

        i += 64;
    }

    // SAFETY: Target feature avx512f enabled on function; sum_v is a valid __m512i vector.
    let mut sum = unsafe { hsum512_epi32_avx512(sum_v) } as u32;

    while i < len {
        // SAFETY: Loop condition i < len guarantees index i is in-bounds for both a and b.
        sum += unsafe { (*a.get_unchecked(i) as u32) * (*b.get_unchecked(i) as u32) };
        i += 1;
    }

    sum
}

/// Calculates squared Euclidean distance between two `u8` byte slices using AVX-512 BW.
///
/// # Safety
///
/// - The caller must guarantee that the host CPU supports `avx512f` and `avx512bw` (e.g. checked via `is_x86_feature_detected!`).
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f", enable = "avx512bw")]
// SAFETY: Host CPU support for avx512f and avx512bw guaranteed by caller.
pub(crate) unsafe fn euclidean_distance_sq_u8_avx512(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target features avx512f/bw enabled on function; zero vector initialization has no memory hazards.
    let mut sum_v = unsafe { _mm512_setzero_si512() };
    // SAFETY: Target features avx512f/bw enabled on function; zero vector initialization has no memory hazards.
    let zero = unsafe { _mm512_setzero_si512() };

    while i + 64 <= len {
        // SAFETY: Loop condition i + 64 <= len guarantees a[i..i+64] is in-bounds. _mm512_loadu_si512 supports unaligned reads.
        let va = unsafe { _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i) };
        // SAFETY: Loop condition i + 64 <= len guarantees b[i..i+64] is in-bounds. _mm512_loadu_si512 supports unaligned reads.
        let vb = unsafe { _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i) };

        // SAFETY: Target features avx512f/bw enabled on function; operates on valid __m512i vector registers.
        let va_lo = unsafe { _mm512_unpacklo_epi8(va, zero) };
        // SAFETY: Target features avx512f/bw enabled on function; operates on valid __m512i vector registers.
        let vb_lo = unsafe { _mm512_unpacklo_epi8(vb, zero) };
        // SAFETY: Target features avx512f/bw enabled on function; operates on valid __m512i vector registers.
        let diff_lo = unsafe { _mm512_sub_epi16(va_lo, vb_lo) };
        // SAFETY: Target features avx512f/bw enabled on function; operates on valid __m512i vector registers.
        let prod_lo = unsafe { _mm512_madd_epi16(diff_lo, diff_lo) };

        // SAFETY: Target features avx512f/bw enabled on function; operates on valid __m512i vector registers.
        let va_hi = unsafe { _mm512_unpackhi_epi8(va, zero) };
        // SAFETY: Target features avx512f/bw enabled on function; operates on valid __m512i vector registers.
        let vb_hi = unsafe { _mm512_unpackhi_epi8(vb, zero) };
        // SAFETY: Target features avx512f/bw enabled on function; operates on valid __m512i vector registers.
        let diff_hi = unsafe { _mm512_sub_epi16(va_hi, vb_hi) };
        // SAFETY: Target features avx512f/bw enabled on function; operates on valid __m512i vector registers.
        let prod_hi = unsafe { _mm512_madd_epi16(diff_hi, diff_hi) };

        // SAFETY: Target features avx512f/bw enabled on function; operates on valid __m512i vector registers.
        sum_v = unsafe { _mm512_add_epi32(sum_v, prod_lo) };
        // SAFETY: Target features avx512f/bw enabled on function; operates on valid __m512i vector registers.
        sum_v = unsafe { _mm512_add_epi32(sum_v, prod_hi) };

        i += 64;
    }

    // SAFETY: Target feature avx512f enabled on function; sum_v is a valid __m512i vector.
    let mut sum = unsafe { hsum512_epi32_avx512(sum_v) } as u32;

    while i < len {
        // SAFETY: Loop condition i < len guarantees index i is in-bounds for both a and b.
        let diff = unsafe { (*a.get_unchecked(i) as i32) - (*b.get_unchecked(i) as i32) };
        sum += (diff * diff) as u32;
        i += 1;
    }

    sum
}

/// Calculates dot product, norm_a^2, and norm_b^2 for two `u8` byte slices using AVX-512 VNNI.
///
/// # Safety
///
/// - The caller must guarantee that the host CPU supports `avx512f`, `avx512bw`, and `avx512vnni` (e.g. checked via `is_x86_feature_detected!`).
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f", enable = "avx512bw", enable = "avx512vnni")]
// SAFETY: Host CPU support for avx512f, avx512bw, and avx512vnni guaranteed by caller.
pub(crate) unsafe fn cosine_similarity_parts_u8_avx512(
    a: &[u8],
    b: &[u8],
) -> CosineSimilarityPartsU8 {
    let len = a.len().min(b.len());
    let mut i = 0;

    // SAFETY: Target features avx512f/bw/vnni enabled on function; zero vector initialization has no memory hazards.
    let mut dot_v = unsafe { _mm512_setzero_si512() };
    // SAFETY: Target features avx512f/bw/vnni enabled on function; zero vector initialization has no memory hazards.
    let mut norm_a_v = unsafe { _mm512_setzero_si512() };
    // SAFETY: Target features avx512f/bw/vnni enabled on function; zero vector initialization has no memory hazards.
    let mut norm_b_v = unsafe { _mm512_setzero_si512() };
    // SAFETY: Target features avx512f/bw/vnni enabled on function; zero vector initialization has no memory hazards.
    let zero = unsafe { _mm512_setzero_si512() };

    while i + 64 <= len {
        // SAFETY: Loop condition i + 64 <= len guarantees a[i..i+64] is in-bounds. _mm512_loadu_si512 supports unaligned reads.
        let va = unsafe { _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i) };
        // SAFETY: Loop condition i + 64 <= len guarantees b[i..i+64] is in-bounds. _mm512_loadu_si512 supports unaligned reads.
        let vb = unsafe { _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i) };

        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        let va_lo = unsafe { _mm512_unpacklo_epi8(va, zero) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        let vb_lo = unsafe { _mm512_unpacklo_epi8(vb, zero) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        dot_v = unsafe { _mm512_add_epi32(dot_v, _mm512_madd_epi16(va_lo, vb_lo)) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        norm_a_v = unsafe { _mm512_add_epi32(norm_a_v, _mm512_madd_epi16(va_lo, va_lo)) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        norm_b_v = unsafe { _mm512_add_epi32(norm_b_v, _mm512_madd_epi16(vb_lo, vb_lo)) };

        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        let va_hi = unsafe { _mm512_unpackhi_epi8(va, zero) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        let vb_hi = unsafe { _mm512_unpackhi_epi8(vb, zero) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        dot_v = unsafe { _mm512_add_epi32(dot_v, _mm512_madd_epi16(va_hi, vb_hi)) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        norm_a_v = unsafe { _mm512_add_epi32(norm_a_v, _mm512_madd_epi16(va_hi, va_hi)) };
        // SAFETY: Target features avx512f/bw/vnni enabled on function; operates on valid register values.
        norm_b_v = unsafe { _mm512_add_epi32(norm_b_v, _mm512_madd_epi16(vb_hi, vb_hi)) };

        i += 64;
    }

    // SAFETY: Target feature avx512f enabled on function; dot_v is a valid __m512i vector.
    let mut dot = unsafe { hsum512_epi32_avx512(dot_v) } as u32;
    // SAFETY: Target feature avx512f enabled on function; norm_a_v is a valid __m512i vector.
    let mut norm_a_sq = unsafe { hsum512_epi32_avx512(norm_a_v) } as u32;
    // SAFETY: Target feature avx512f enabled on function; norm_b_v is a valid __m512i vector.
    let mut norm_b_sq = unsafe { hsum512_epi32_avx512(norm_b_v) } as u32;

    while i < len {
        // SAFETY: Loop condition i < len guarantees index i is in-bounds for a.
        let x = unsafe { *a.get_unchecked(i) as u32 };
        // SAFETY: Loop condition i < len guarantees index i is in-bounds for b.
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

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::kernels::scalar::*;

    struct XorShift64(u64);
    impl XorShift64 {
        fn new(seed: u64) -> Self {
            Self(if seed == 0 { 0x1234_5678_9ABC_DEF0 } else { seed })
        }
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        #[allow(clippy::cast_possible_truncation)]
        fn next_f32_range(&mut self, min: f32, max: f32) -> f32 {
            let val = (self.next_u64() as f64) / (u64::MAX as f64);
            (min as f64 + val * (max - min) as f64) as f32
        }
        #[allow(clippy::cast_possible_truncation)]
        fn next_u8(&mut self) -> u8 {
            self.next_u64() as u8
        }
    }

    fn check_avx512f_support() -> bool {
        let has_avx512f = is_x86_feature_detected!("avx512f");
        if !has_avx512f {
            eprintln!("SKIPPED: AVX-512F feature not supported on host CPU");
            false
        } else {
            true
        }
    }

    fn check_avx512bw_support() -> bool {
        let has_avx512f = is_x86_feature_detected!("avx512f");
        let has_avx512bw = is_x86_feature_detected!("avx512bw");
        if !has_avx512f || !has_avx512bw {
            eprintln!("SKIPPED: AVX-512BW feature not supported on host CPU (avx512f={has_avx512f}, avx512bw={has_avx512bw})");
            false
        } else {
            true
        }
    }

    fn check_avx512vnni_support() -> bool {
        let has_avx512f = is_x86_feature_detected!("avx512f");
        let has_avx512bw = is_x86_feature_detected!("avx512bw");
        let has_avx512vnni = is_x86_feature_detected!("avx512vnni");
        if !has_avx512f || !has_avx512bw || !has_avx512vnni {
            eprintln!("SKIPPED: AVX-512VNNI feature not supported on host CPU (f={has_avx512f}, bw={has_avx512bw}, vnni={has_avx512vnni})");
            false
        } else {
            true
        }
    }

    const TEST_LENGTHS: &[usize] = &[
        0, 1, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 1_000_003,
    ];

    fn approx_eq(a: f32, b: f32, tol: f32) -> bool {
        if a.is_nan() && b.is_nan() {
            return true;
        }
        if a.is_infinite() && b.is_infinite() && a.is_sign_positive() == b.is_sign_positive() {
            return true;
        }
        let diff = (a - b).abs();
        let scale = a.abs().max(b.abs()).max(1.0);
        diff <= tol * scale
    }

    #[test]
    fn test_avx512_cosine_distance_direct() {
        if !check_avx512f_support() {
            return;
        }
        let mut rng = XorShift64::new(42);

        for &len in TEST_LENGTHS {
            for offset in 0..8 {
                let mut backing_a = vec![0.0f32; len + offset];
                let mut backing_b = vec![0.0f32; len + offset];
                for i in offset..(offset + len) {
                    backing_a[i] = rng.next_f32_range(-10.0, 10.0);
                    backing_b[i] = rng.next_f32_range(-10.0, 10.0);
                }
                let a = &backing_a[offset..offset + len];
                let b = &backing_b[offset..offset + len];

                let s = cosine_distance_scalar(a, b);
                // SAFETY: CPU feature avx512f verified via check_avx512f_support.
                let simd = unsafe { cosine_distance_avx512(a, b) };

                assert!(
                    approx_eq(s, simd, 1e-4),
                    "Cosine AVX512 mismatch at len {len}, offset {offset}: scalar={s}, simd={simd}"
                );
            }
        }
    }

    #[test]
    fn test_avx512_euclidean_distance_direct() {
        if !check_avx512f_support() {
            return;
        }
        let mut rng = XorShift64::new(123);

        for &len in TEST_LENGTHS {
            for offset in 0..8 {
                let mut backing_a = vec![0.0f32; len + offset];
                let mut backing_b = vec![0.0f32; len + offset];
                for i in offset..(offset + len) {
                    backing_a[i] = rng.next_f32_range(-10.0, 10.0);
                    backing_b[i] = rng.next_f32_range(-10.0, 10.0);
                }
                let a = &backing_a[offset..offset + len];
                let b = &backing_b[offset..offset + len];

                let s = euclidean_distance_scalar(a, b);
                // SAFETY: CPU feature avx512f verified via check_avx512f_support.
                let simd = unsafe { euclidean_distance_avx512(a, b) };

                assert!(
                    approx_eq(s, simd, 1e-3),
                    "Euclidean AVX512 mismatch at len {len}, offset {offset}: scalar={s}, simd={simd}"
                );
            }
        }
    }

    #[test]
    fn test_avx512_dot_product_direct() {
        if !check_avx512f_support() {
            return;
        }
        let mut rng = XorShift64::new(456);

        for &len in TEST_LENGTHS {
            for offset in 0..8 {
                let mut backing_a = vec![0.0f32; len + offset];
                let mut backing_b = vec![0.0f32; len + offset];
                for i in offset..(offset + len) {
                    backing_a[i] = rng.next_f32_range(-10.0, 10.0);
                    backing_b[i] = rng.next_f32_range(-10.0, 10.0);
                }
                let a = &backing_a[offset..offset + len];
                let b = &backing_b[offset..offset + len];

                let s = dot_product_scalar(a, b);
                // SAFETY: CPU feature avx512f verified via check_avx512f_support.
                let simd = unsafe { dot_product_avx512(a, b) };

                assert!(
                    approx_eq(s, simd, 1e-3),
                    "Dot product AVX512 mismatch at len {len}, offset {offset}: scalar={s}, simd={simd}"
                );
            }
        }
    }

    #[test]
    fn test_avx512_f32_bytes_direct() {
        if !check_avx512f_support() {
            return;
        }
        let mut rng = XorShift64::new(789);

        for &len in &[0, 1, 7, 8, 15, 16, 31, 32, 100] {
            let a: Vec<f32> = (0..len).map(|_| rng.next_f32_range(-5.0, 5.0)).collect();
            let b: Vec<f32> = (0..len).map(|_| rng.next_f32_range(-5.0, 5.0)).collect();
            let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

            let s_cos = cosine_distance_f32_bytes_scalar(&a, &b_bytes);
            // SAFETY: CPU feature avx512f verified via check_avx512f_support.
            let simd_cos = unsafe { cosine_distance_f32_bytes_avx512(&a, &b_bytes) };
            assert!(approx_eq(s_cos, simd_cos, 1e-4));

            let s_euc = euclidean_distance_f32_bytes_scalar(&a, &b_bytes);
            // SAFETY: CPU feature avx512f verified via check_avx512f_support.
            let simd_euc = unsafe { euclidean_distance_f32_bytes_avx512(&a, &b_bytes) };
            assert!(approx_eq(s_euc, simd_euc, 1e-3));

            let s_dot = dot_product_f32_bytes_scalar(&a, &b_bytes);
            // SAFETY: CPU feature avx512f verified via check_avx512f_support.
            let simd_dot = unsafe { dot_product_f32_bytes_avx512(&a, &b_bytes) };
            assert!(approx_eq(s_dot, simd_dot, 1e-3));
        }
    }

    #[test]
    fn test_avx512_u8_kernels_exact() {
        let mut rng = XorShift64::new(999);

        for &len in &[0, 1, 3, 16, 31, 32, 33, 64, 128, 255] {
            for offset in 0..8 {
                let mut backing_a = vec![0u8; len + offset];
                let mut backing_b = vec![0u8; len + offset];
                for i in offset..(offset + len) {
                    backing_a[i] = rng.next_u8();
                    backing_b[i] = rng.next_u8();
                }
                let a = &backing_a[offset..offset + len];
                let b = &backing_b[offset..offset + len];

                if check_avx512vnni_support() {
                    let s_dot = dot_product_u8_scalar(a, b);
                    // SAFETY: CPU features avx512f, bw, vnni verified via check_avx512vnni_support.
                    let simd_dot = unsafe { dot_product_u8_avx512vnni(a, b) };
                    assert_eq!(s_dot, simd_dot, "u8 dot mismatch len={len}");

                    let s_parts = cosine_similarity_parts_u8_scalar(a, b);
                    // SAFETY: CPU features avx512f, bw, vnni verified via check_avx512vnni_support.
                    let simd_parts = unsafe { cosine_similarity_parts_u8_avx512(a, b) };
                    assert_eq!(s_parts, simd_parts, "u8 cosine parts mismatch len={len}");
                }

                if check_avx512bw_support() {
                    let s_euc = euclidean_distance_sq_u8_scalar(a, b);
                    // SAFETY: CPU features avx512f, bw verified via check_avx512bw_support.
                    let simd_euc = unsafe { euclidean_distance_sq_u8_avx512(a, b) };
                    assert_eq!(s_euc, simd_euc, "u8 euc sq mismatch len={len}");
                }
            }
        }
    }

    #[test]
    fn test_avx512_special_float_values() {
        if !check_avx512f_support() {
            return;
        }

        let special_vals = [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            -0.0f32,
            0.0f32,
            1e-38f32,
            f32::MIN_POSITIVE,
        ];

        for &v in &special_vals {
            let a = vec![v; 32];
            let b = vec![1.0f32; 32];

            let s_cos = cosine_distance_scalar(&a, &b);
            // SAFETY: CPU feature avx512f verified via check_avx512f_support.
            let simd_cos = unsafe { cosine_distance_avx512(&a, &b) };
            assert!(approx_eq(s_cos, simd_cos, 1e-4));

            let s_euc = euclidean_distance_scalar(&a, &b);
            // SAFETY: CPU feature avx512f verified via check_avx512f_support.
            let simd_euc = unsafe { euclidean_distance_avx512(&a, &b) };
            assert!(approx_eq(s_euc, simd_euc, 1e-3));

            let s_dot = dot_product_scalar(&a, &b);
            // SAFETY: CPU feature avx512f verified via check_avx512f_support.
            let simd_dot = unsafe { dot_product_avx512(&a, &b) };
            assert!(approx_eq(s_dot, simd_dot, 1e-3));
        }
    }
}

/// Horizontally sums 16 32-bit integers in a 512-bit vector.
///
/// # Safety
///
/// - The caller must guarantee that the host CPU supports `avx512f` (e.g. checked via `is_x86_feature_detected!`).
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
// SAFETY: Host CPU support for avx512f guaranteed by caller.
pub(crate) unsafe fn hsum512_epi32_avx512(v: __m512i) -> i32 {
    // SAFETY: Target feature avx512f enabled on function; extracts and adds valid 256-bit halves of __m512i vector.
    let v256 =
        unsafe { _mm256_add_epi32(_mm512_castsi512_si256(v), _mm512_extracti32x8_epi32(v, 1)) };
    // SAFETY: Target feature avx512f enabled; valid 256-bit to 128-bit vector cast.
    let vlow = unsafe { _mm256_castsi256_si128(v256) };
    // SAFETY: Target feature avx512f enabled; extracts upper 128-bit lane from valid __m256i vector.
    let vhigh = unsafe { _mm256_extracti128_si256(v256, 1) };
    // SAFETY: Target feature avx512f enabled; adds valid 128-bit integer vectors.
    let v128 = unsafe { _mm_add_epi32(vlow, vhigh) };
    // SAFETY: Target feature avx512f enabled; shifts high 64 bits to low and adds.
    let v64 = unsafe { _mm_add_epi32(v128, _mm_srli_si128(v128, 8)) };
    // SAFETY: Target feature avx512f enabled; shifts second 32-bit element and adds.
    let v32 = unsafe { _mm_add_epi32(v64, _mm_srli_si128(v64, 4)) };
    // SAFETY: Target feature avx512f enabled; extracts scalar i32 from low element of valid __m128i.
    unsafe { _mm_cvtsi128_si32(v32) }
}
