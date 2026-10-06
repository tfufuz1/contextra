// FILE-CONTEXT
// ZWECK: ARM NEON SIMD-Intrinsics für f32 Distanzberechnungen.
// INVARIANTEN: Target-Architektur aarch64 / neon.

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// Calculates cosine distance using ARM NEON intrinsics.
///
/// # Safety
/// Caller must ensure that:
/// - The target architecture supports ARM NEON intrinsics (`neon` target feature).
/// - `a` and `b` have equal lengths or sufficient length to prevent out-of-bounds reads.
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
// SAFETY: Target feature "neon" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub unsafe fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Valid NEON vector initialization with constant 0.0.
    let mut dot_v = unsafe { vdupq_n_f32(0.0) };
    // SAFETY: Valid NEON vector initialization with constant 0.0.
    let mut norm_a_v = unsafe { vdupq_n_f32(0.0) };
    // SAFETY: Valid NEON vector initialization with constant 0.0.
    let mut norm_b_v = unsafe { vdupq_n_f32(0.0) };

    while i + 4 <= len {
        // SAFETY: Loop condition i + 4 <= len guarantees a pointer offset i..i+4 is in bounds.
        let va = unsafe { vld1q_f32(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 4 <= len guarantees b pointer offset i..i+4 is in bounds.
        let vb = unsafe { vld1q_f32(b.as_ptr().add(i)) };

        // SAFETY: Valid NEON fused multiply-accumulate on initialized vector registers.
        dot_v = unsafe { vfmaq_f32(dot_v, va, vb) };
        // SAFETY: Valid NEON fused multiply-accumulate on initialized vector registers.
        norm_a_v = unsafe { vfmaq_f32(norm_a_v, va, va) };
        // SAFETY: Valid NEON fused multiply-accumulate on initialized vector registers.
        norm_b_v = unsafe { vfmaq_f32(norm_b_v, vb, vb) };

        i += 4;
    }

    // SAFETY: Valid NEON vector lane reduction sum across registers.
    let mut dot = unsafe { vaddvq_f32(dot_v) };
    // SAFETY: Valid NEON vector lane reduction sum across registers.
    let mut norm_a = unsafe { vaddvq_f32(norm_a_v) };
    // SAFETY: Valid NEON vector lane reduction sum across registers.
    let mut norm_b = unsafe { vaddvq_f32(norm_b_v) };

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

/// Calculates Euclidean distance using ARM NEON intrinsics.
///
/// # Safety
/// Caller must ensure that:
/// - The target architecture supports ARM NEON intrinsics (`neon` target feature).
/// - `a` and `b` have equal lengths or sufficient length to prevent out-of-bounds reads.
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
// SAFETY: Target feature "neon" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub unsafe fn euclidean_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Valid NEON vector initialization with constant 0.0.
    let mut sum_v = unsafe { vdupq_n_f32(0.0) };

    while i + 4 <= len {
        // SAFETY: Loop condition i + 4 <= len guarantees a pointer offset i..i+4 is in bounds.
        let va = unsafe { vld1q_f32(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 4 <= len guarantees b pointer offset i..i+4 is in bounds.
        let vb = unsafe { vld1q_f32(b.as_ptr().add(i)) };
        // SAFETY: Valid NEON vector subtraction.
        let diff = unsafe { vsubq_f32(va, vb) };

        // SAFETY: Valid NEON fused multiply-accumulate on initialized vector registers.
        sum_v = unsafe { vfmaq_f32(sum_v, diff, diff) };

        i += 4;
    }

    // SAFETY: Valid NEON vector lane reduction sum across registers.
    let mut sum = unsafe { vaddvq_f32(sum_v) };

    while i < len {
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into a and b.
        let diff = unsafe { *a.get_unchecked(i) - *b.get_unchecked(i) };
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Calculates dot product using ARM NEON intrinsics.
///
/// # Safety
/// Caller must ensure that:
/// - The target architecture supports ARM NEON intrinsics (`neon` target feature).
/// - `a` and `b` have equal lengths or sufficient length to prevent out-of-bounds reads.
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
// SAFETY: Target feature "neon" enabled by target_feature attribute; caller guarantees valid slice bounds.
pub unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Valid NEON vector initialization with constant 0.0.
    let mut dot_v = unsafe { vdupq_n_f32(0.0) };

    while i + 4 <= len {
        // SAFETY: Loop condition i + 4 <= len guarantees a pointer offset i..i+4 is in bounds.
        let va = unsafe { vld1q_f32(a.as_ptr().add(i)) };
        // SAFETY: Loop condition i + 4 <= len guarantees b pointer offset i..i+4 is in bounds.
        let vb = unsafe { vld1q_f32(b.as_ptr().add(i)) };

        // SAFETY: Valid NEON fused multiply-accumulate on initialized vector registers.
        dot_v = unsafe { vfmaq_f32(dot_v, va, vb) };

        i += 4;
    }

    // SAFETY: Valid NEON vector lane reduction sum across registers.
    let mut dot = unsafe { vaddvq_f32(dot_v) };

    while i < len {
        // SAFETY: Tail loop condition i < len guarantees i is a valid index into a and b.
        dot += unsafe { *a.get_unchecked(i) * *b.get_unchecked(i) };
        i += 1;
    }

    -dot
}
