// FILE-CONTEXT
// ZWECK: ARM NEON SIMD-Intrinsics für f32 Distanzberechnungen.
// INVARIANTEN: Target-Architektur aarch64 / neon.

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
pub unsafe fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = unsafe { vdupq_n_f32(0.0) };
    let mut norm_a_v = unsafe { vdupq_n_f32(0.0) };
    let mut norm_b_v = unsafe { vdupq_n_f32(0.0) };

    while i + 4 <= len {
        let va = unsafe { vld1q_f32(a.as_ptr().add(i)) };
        let vb = unsafe { vld1q_f32(b.as_ptr().add(i)) };

        dot_v = unsafe { vfmaq_f32(dot_v, va, vb) };
        norm_a_v = unsafe { vfmaq_f32(norm_a_v, va, va) };
        norm_b_v = unsafe { vfmaq_f32(norm_b_v, vb, vb) };

        i += 4;
    }

    let mut dot = unsafe { vaddvq_f32(dot_v) };
    let mut norm_a = unsafe { vaddvq_f32(norm_a_v) };
    let mut norm_b = unsafe { vaddvq_f32(norm_b_v) };

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

#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
pub unsafe fn euclidean_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut sum_v = unsafe { vdupq_n_f32(0.0) };

    while i + 4 <= len {
        let va = unsafe { vld1q_f32(a.as_ptr().add(i)) };
        let vb = unsafe { vld1q_f32(b.as_ptr().add(i)) };
        let diff = unsafe { vsubq_f32(va, vb) };

        sum_v = unsafe { vfmaq_f32(sum_v, diff, diff) };

        i += 4;
    }

    let mut sum = unsafe { vaddvq_f32(sum_v) };

    while i < len {
        let diff = unsafe { *a.get_unchecked(i) - *b.get_unchecked(i) };
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
pub unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = unsafe { vdupq_n_f32(0.0) };

    while i + 4 <= len {
        let va = unsafe { vld1q_f32(a.as_ptr().add(i)) };
        let vb = unsafe { vld1q_f32(b.as_ptr().add(i)) };

        dot_v = unsafe { vfmaq_f32(dot_v, va, vb) };

        i += 4;
    }

    let mut dot = unsafe { vaddvq_f32(dot_v) };

    while i < len {
        dot += unsafe { *a.get_unchecked(i) * *b.get_unchecked(i) };
        i += 1;
    }

    -dot
}
