// FILE-CONTEXT
// ZWECK: AVX-512 SIMD-Intrinsics für f32, f32_bytes und u8 VNNI Distanzberechnungen.
// INVARIANTEN: Target-Features guaranteed by caller or feature-checks.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::kernels::scalar::CosineSimilarityPartsU8;

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn cosine_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = unsafe { _mm512_setzero_ps() };
    let mut norm_a_v = unsafe { _mm512_setzero_ps() };
    let mut norm_b_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm512_loadu_ps(b.as_ptr().add(i)) };

        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };
        norm_a_v = unsafe { _mm512_fmadd_ps(va, va, norm_a_v) };
        norm_b_v = unsafe { _mm512_fmadd_ps(vb, vb, norm_b_v) };

        i += 16;
    }

    let mut dot = unsafe { hsum512_ps_avx(dot_v) };
    let mut norm_a = unsafe { hsum512_ps_avx(norm_a_v) };
    let mut norm_b = unsafe { hsum512_ps_avx(norm_b_v) };

    while i < len {
        let x = unsafe { *a.get_unchecked(i) };
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

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut sum_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm512_loadu_ps(b.as_ptr().add(i)) };
        let diff = unsafe { _mm512_sub_ps(va, vb) };

        sum_v = unsafe { _mm512_fmadd_ps(diff, diff, sum_v) };

        i += 16;
    }

    let mut sum = unsafe { hsum512_ps_avx(sum_v) };

    while i < len {
        let diff = unsafe { *a.get_unchecked(i) - *b.get_unchecked(i) };
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn dot_product_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm512_loadu_ps(b.as_ptr().add(i)) };

        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };

        i += 16;
    }

    let mut dot = unsafe { hsum512_ps_avx(dot_v) };

    while i < len {
        dot += unsafe { *a.get_unchecked(i) * *b.get_unchecked(i) };
        i += 1;
    }

    -dot
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn cosine_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = unsafe { _mm512_setzero_ps() };
    let mut norm_a_v = unsafe { _mm512_setzero_ps() };
    let mut norm_b_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };

        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };
        norm_a_v = unsafe { _mm512_fmadd_ps(va, va, norm_a_v) };
        norm_b_v = unsafe { _mm512_fmadd_ps(vb, vb, norm_b_v) };

        i += 16;
    }

    let mut dot = unsafe { hsum512_ps_avx(dot_v) };
    let mut norm_a = unsafe { hsum512_ps_avx(norm_a_v) };
    let mut norm_b = unsafe { hsum512_ps_avx(norm_b_v) };

    while i < len {
        let x = unsafe { *a.get_unchecked(i) };
        let b_val = f32::from_le_bytes([
            unsafe { *b_bytes.get_unchecked(i * 4) },
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
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

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn euclidean_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut sum_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };
        let diff = unsafe { _mm512_sub_ps(va, vb) };

        sum_v = unsafe { _mm512_fmadd_ps(diff, diff, sum_v) };

        i += 16;
    }

    let mut sum = unsafe { hsum512_ps_avx(sum_v) };

    while i < len {
        let b_val = f32::from_le_bytes([
            unsafe { *b_bytes.get_unchecked(i * 4) },
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
            unsafe { *b_bytes.get_unchecked(i * 4 + 3) },
        ]);
        let diff = unsafe { *a.get_unchecked(i) } - b_val;
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn dot_product_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = unsafe { _mm512_setzero_ps() };

    while i + 16 <= len {
        let va = unsafe { _mm512_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm512_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };

        dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };

        i += 16;
    }

    let mut dot = unsafe { hsum512_ps_avx(dot_v) };

    while i < len {
        let b_val = f32::from_le_bytes([
            unsafe { *b_bytes.get_unchecked(i * 4) },
            unsafe { *b_bytes.get_unchecked(i * 4 + 1) },
            unsafe { *b_bytes.get_unchecked(i * 4 + 2) },
            unsafe { *b_bytes.get_unchecked(i * 4 + 3) },
        ]);
        dot += unsafe { *a.get_unchecked(i) } * b_val;
        i += 1;
    }

    -dot
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn hsum512_ps_avx(v: __m512) -> f32 {
    let v256 = unsafe { _mm256_add_ps(_mm512_extractf32x8_ps(v, 0), _mm512_extractf32x8_ps(v, 1)) };
    let vlow = unsafe { _mm256_castps256_ps128(v256) };
    let vhigh = unsafe { _mm256_extractf128_ps(v256, 1) };
    let v128 = unsafe { _mm_add_ps(vlow, vhigh) };
    let v64 = unsafe { _mm_add_ps(v128, _mm_movehl_ps(v128, v128)) };
    let v32 = unsafe { _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55)) };
    unsafe { _mm_cvtss_f32(v32) }
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f", enable = "avx512bw", enable = "avx512vnni")]
pub unsafe fn dot_product_u8_avx512vnni(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    let mut sum_v = unsafe { _mm512_setzero_si512() };

    while i + 64 <= len {
        let va = unsafe { _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i) };
        let vb = unsafe { _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i) };

        sum_v = unsafe { _mm512_dpbusd_epi32(sum_v, va, vb) };

        i += 64;
    }

    let mut sum = unsafe { hsum512_epi32_avx512(sum_v) } as u32;

    while i < len {
        sum += unsafe { (*a.get_unchecked(i) as u32) * (*b.get_unchecked(i) as u32) };
        i += 1;
    }

    sum
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f", enable = "avx512bw")]
pub unsafe fn euclidean_distance_sq_u8_avx512(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    let mut sum_v = unsafe { _mm512_setzero_si512() };
    let zero = unsafe { _mm512_setzero_si512() };

    while i + 64 <= len {
        let va = unsafe { _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i) };
        let vb = unsafe { _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i) };

        let va_lo = unsafe { _mm512_unpacklo_epi8(va, zero) };
        let vb_lo = unsafe { _mm512_unpacklo_epi8(vb, zero) };
        let diff_lo = unsafe { _mm512_sub_epi16(va_lo, vb_lo) };
        let prod_lo = unsafe { _mm512_madd_epi16(diff_lo, diff_lo) };

        let va_hi = unsafe { _mm512_unpackhi_epi8(va, zero) };
        let vb_hi = unsafe { _mm512_unpackhi_epi8(vb, zero) };
        let diff_hi = unsafe { _mm512_sub_epi16(va_hi, vb_hi) };
        let prod_hi = unsafe { _mm512_madd_epi16(diff_hi, diff_hi) };

        sum_v = unsafe { _mm512_add_epi32(sum_v, prod_lo) };
        sum_v = unsafe { _mm512_add_epi32(sum_v, prod_hi) };

        i += 64;
    }

    let mut sum = unsafe { hsum512_epi32_avx512(sum_v) } as u32;

    while i < len {
        let diff = unsafe { (*a.get_unchecked(i) as i32) - (*b.get_unchecked(i) as i32) };
        sum += (diff * diff) as u32;
        i += 1;
    }

    sum
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f", enable = "avx512bw", enable = "avx512vnni")]
pub unsafe fn cosine_similarity_parts_u8_avx512(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
    let len = a.len().min(b.len());
    let mut i = 0;

    let mut dot_v = unsafe { _mm512_setzero_si512() };
    let mut norm_a_v = unsafe { _mm512_setzero_si512() };
    let mut norm_b_v = unsafe { _mm512_setzero_si512() };

    while i + 64 <= len {
        let va = unsafe { _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i) };
        let vb = unsafe { _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i) };

        dot_v = unsafe { _mm512_dpbusd_epi32(dot_v, va, vb) };
        norm_a_v = unsafe { _mm512_dpbusd_epi32(norm_a_v, va, va) };
        norm_b_v = unsafe { _mm512_dpbusd_epi32(norm_b_v, vb, vb) };

        i += 64;
    }

    let mut dot = unsafe { hsum512_epi32_avx512(dot_v) } as u32;
    let mut norm_a_sq = unsafe { hsum512_epi32_avx512(norm_a_v) } as u32;
    let mut norm_b_sq = unsafe { hsum512_epi32_avx512(norm_b_v) } as u32;

    while i < len {
        let x = unsafe { *a.get_unchecked(i) as u32 };
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

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx512f")]
pub unsafe fn hsum512_epi32_avx512(v: __m512i) -> i32 {
    let v256 =
        unsafe { _mm256_add_epi32(_mm512_castsi512_si256(v), _mm512_extracti32x8_epi32(v, 1)) };
    let vlow = unsafe { _mm256_castsi256_si128(v256) };
    let vhigh = unsafe { _mm256_extracti128_si256(v256, 1) };
    let v128 = unsafe { _mm_add_epi32(vlow, vhigh) };
    let v64 = unsafe { _mm_add_epi32(v128, _mm_srli_si128(v128, 8)) };
    let v32 = unsafe { _mm_add_epi32(v64, _mm_srli_si128(v64, 4)) };
    unsafe { _mm_cvtsi128_si32(v32) }
}
