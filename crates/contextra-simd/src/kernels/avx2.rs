// FILE-CONTEXT
// ZWECK: AVX2 SIMD-Intrinsics für f32, f32_bytes und u8 Distanzberechnungen.
// INVARIANTEN: Target-Feature "avx2", "fma" garantiert vor Aufruf via Feature-Check.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::kernels::scalar::CosineSimilarityPartsU8;

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = unsafe { _mm256_setzero_ps() };
    let mut norm_a_v = unsafe { _mm256_setzero_ps() };
    let mut norm_b_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(i)) };

        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };
        norm_a_v = unsafe { _mm256_fmadd_ps(va, va, norm_a_v) };
        norm_b_v = unsafe { _mm256_fmadd_ps(vb, vb, norm_b_v) };

        i += 8;
    }

    let mut dot = unsafe { hsum256_ps_avx(dot_v) };
    let mut norm_a = unsafe { hsum256_ps_avx(norm_a_v) };
    let mut norm_b = unsafe { hsum256_ps_avx(norm_b_v) };

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
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut sum_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(i)) };
        let diff = unsafe { _mm256_sub_ps(va, vb) };

        sum_v = unsafe { _mm256_fmadd_ps(diff, diff, sum_v) };

        i += 8;
    }

    let mut sum = unsafe { hsum256_ps_avx(sum_v) };

    while i < len {
        let diff = unsafe { *a.get_unchecked(i) - *b.get_unchecked(i) };
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(i)) };

        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };

        i += 8;
    }

    let mut dot = unsafe { hsum256_ps_avx(dot_v) };

    while i < len {
        dot += unsafe { *a.get_unchecked(i) * *b.get_unchecked(i) };
        i += 1;
    }

    -dot
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn cosine_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = unsafe { _mm256_setzero_ps() };
    let mut norm_a_v = unsafe { _mm256_setzero_ps() };
    let mut norm_b_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };

        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };
        norm_a_v = unsafe { _mm256_fmadd_ps(va, va, norm_a_v) };
        norm_b_v = unsafe { _mm256_fmadd_ps(vb, vb, norm_b_v) };

        i += 8;
    }

    let mut dot = unsafe { hsum256_ps_avx(dot_v) };
    let mut norm_a = unsafe { hsum256_ps_avx(norm_a_v) };
    let mut norm_b = unsafe { hsum256_ps_avx(norm_b_v) };

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
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn euclidean_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut sum_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };
        let diff = unsafe { _mm256_sub_ps(va, vb) };

        sum_v = unsafe { _mm256_fmadd_ps(diff, diff, sum_v) };

        i += 8;
    }

    let mut sum = unsafe { hsum256_ps_avx(sum_v) };

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
#[target_feature(enable = "avx2", enable = "fma")]
pub unsafe fn dot_product_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = unsafe { _mm256_setzero_ps() };

    while i + 8 <= len {
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm256_loadu_ps(b_bytes.as_ptr().add(i * 4) as *const f32) };

        dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };

        i += 8;
    }

    let mut dot = unsafe { hsum256_ps_avx(dot_v) };

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
#[target_feature(enable = "avx2")]
pub unsafe fn hsum256_ps_avx(v: __m256) -> f32 {
    let vlow = unsafe { _mm256_castps256_ps128(v) };
    let vhigh = unsafe { _mm256_extractf128_ps(v, 1) };
    let v128 = unsafe { _mm_add_ps(vlow, vhigh) };
    let v64 = unsafe { _mm_add_ps(v128, _mm_movehl_ps(v128, v128)) };
    let v32 = unsafe { _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55)) };
    unsafe { _mm_cvtss_f32(v32) }
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub unsafe fn dot_product_u8_avx2(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    let mut sum_v = unsafe { _mm256_setzero_si256() };
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        let va = unsafe { _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i) };
        let vb = unsafe { _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i) };

        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        let prod_lo = unsafe { _mm256_madd_epi16(va_lo, vb_lo) };

        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        let prod_hi = unsafe { _mm256_madd_epi16(va_hi, vb_hi) };

        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_lo) };
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_hi) };

        i += 32;
    }

    let mut sum = unsafe { hsum256_epi32_avx2(sum_v) } as u32;

    while i < len {
        sum += unsafe { (*a.get_unchecked(i) as u32) * (*b.get_unchecked(i) as u32) };
        i += 1;
    }

    sum
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub unsafe fn euclidean_distance_sq_u8_avx2(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut i = 0;

    let mut sum_v = unsafe { _mm256_setzero_si256() };
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        let va = unsafe { _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i) };
        let vb = unsafe { _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i) };

        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        let diff_lo = unsafe { _mm256_sub_epi16(va_lo, vb_lo) };
        let prod_lo = unsafe { _mm256_madd_epi16(diff_lo, diff_lo) };

        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        let diff_hi = unsafe { _mm256_sub_epi16(va_hi, vb_hi) };
        let prod_hi = unsafe { _mm256_madd_epi16(diff_hi, diff_hi) };

        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_lo) };
        sum_v = unsafe { _mm256_add_epi32(sum_v, prod_hi) };

        i += 32;
    }

    let mut sum = unsafe { hsum256_epi32_avx2(sum_v) } as u32;

    while i < len {
        let diff = unsafe { (*a.get_unchecked(i) as i32) - (*b.get_unchecked(i) as i32) };
        sum += (diff * diff) as u32;
        i += 1;
    }

    sum
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
pub unsafe fn cosine_similarity_parts_u8_avx2(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
    let len = a.len().min(b.len());
    let mut i = 0;

    let mut dot_v = unsafe { _mm256_setzero_si256() };
    let mut norm_a_v = unsafe { _mm256_setzero_si256() };
    let mut norm_b_v = unsafe { _mm256_setzero_si256() };
    let zero = unsafe { _mm256_setzero_si256() };

    while i + 32 <= len {
        let va = unsafe { _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i) };
        let vb = unsafe { _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i) };

        let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };
        let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };
        dot_v = unsafe { _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_lo, vb_lo)) };
        norm_a_v = unsafe { _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_lo, va_lo)) };
        norm_b_v = unsafe { _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_lo, vb_lo)) };

        let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };
        let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };
        dot_v = unsafe { _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_hi, vb_hi)) };
        norm_a_v = unsafe { _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_hi, va_hi)) };
        norm_b_v = unsafe { _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_hi, vb_hi)) };

        i += 32;
    }

    let mut dot = unsafe { hsum256_epi32_avx2(dot_v) } as u32;
    let mut norm_a_sq = unsafe { hsum256_epi32_avx2(norm_a_v) } as u32;
    let mut norm_b_sq = unsafe { hsum256_epi32_avx2(norm_b_v) } as u32;

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
#[target_feature(enable = "avx2")]
pub unsafe fn hsum256_epi32_avx2(v: __m256i) -> i32 {
    let vlow = unsafe { _mm256_castsi256_si128(v) };
    let vhigh = unsafe { _mm256_extracti128_si256(v, 1) };
    let v128 = unsafe { _mm_add_epi32(vlow, vhigh) };
    let v64 = unsafe { _mm_add_epi32(v128, _mm_srli_si128(v128, 8)) };
    let v32 = unsafe { _mm_add_epi32(v64, _mm_srli_si128(v64, 4)) };
    unsafe { _mm_cvtsi128_si32(v32) }
}
