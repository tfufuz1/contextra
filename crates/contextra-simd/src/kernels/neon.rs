// FILE-CONTEXT
// ZWECK: ARM NEON SIMD-Intrinsics für f32 Distanzberechnungen.
// INVARIANTEN: Target-Architektur aarch64 / neon.

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// Computes the cosine distance between two `f32` slices using ARM NEON intrinsics.
///
/// # Safety
///
/// - The caller must ensure that the CPU supports `aarch64` NEON instructions.
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
pub(crate) unsafe fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Initializing 128-bit vector register with 0.0 float values under NEON target feature.
    let mut dot_v = unsafe { vdupq_n_f32(0.0) };
    // SAFETY: Initializing 128-bit vector register with 0.0 float values under NEON target feature.
    let mut norm_a_v = unsafe { vdupq_n_f32(0.0) };
    // SAFETY: Initializing 128-bit vector register with 0.0 float values under NEON target feature.
    let mut norm_b_v = unsafe { vdupq_n_f32(0.0) };

    while i + 4 <= len {
        // SAFETY: `i + 4 <= len` guarantees `a.as_ptr().add(i)` is valid for reading 4 contiguous `f32` elements.
        let va = unsafe { vld1q_f32(a.as_ptr().add(i)) };
        // SAFETY: `i + 4 <= len` guarantees `b.as_ptr().add(i)` is valid for reading 4 contiguous `f32` elements.
        let vb = unsafe { vld1q_f32(b.as_ptr().add(i)) };

        // SAFETY: NEON multiply-accumulate on valid 128-bit float vectors.
        dot_v = unsafe { vfmaq_f32(dot_v, va, vb) };
        // SAFETY: NEON multiply-accumulate on valid 128-bit float vectors.
        norm_a_v = unsafe { vfmaq_f32(norm_a_v, va, va) };
        // SAFETY: NEON multiply-accumulate on valid 128-bit float vectors.
        norm_b_v = unsafe { vfmaq_f32(norm_b_v, vb, vb) };

        i += 4;
    }

    // SAFETY: NEON vector reduction adding across 4 lanes of a valid 128-bit float register.
    let mut dot = unsafe { vaddvq_f32(dot_v) };
    // SAFETY: NEON vector reduction adding across 4 lanes of a valid 128-bit float register.
    let mut norm_a = unsafe { vaddvq_f32(norm_a_v) };
    // SAFETY: NEON vector reduction adding across 4 lanes of a valid 128-bit float register.
    let mut norm_b = unsafe { vaddvq_f32(norm_b_v) };

    while i < len {
        // SAFETY: Loop condition `i < len` guarantees index `i` is within bounds of slice `a`.
        let x = unsafe { *a.get_unchecked(i) };
        // SAFETY: Loop condition `i < len` guarantees index `i` is within bounds of slice `b` (given `a.len() == b.len()`).
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

/// Computes the Euclidean distance between two `f32` slices using ARM NEON intrinsics.
///
/// # Safety
///
/// - The caller must ensure that the CPU supports `aarch64` NEON instructions.
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
pub(crate) unsafe fn euclidean_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Initializing 128-bit vector register with 0.0 float values under NEON target feature.
    let mut sum_v = unsafe { vdupq_n_f32(0.0) };

    while i + 4 <= len {
        // SAFETY: `i + 4 <= len` guarantees `a.as_ptr().add(i)` is valid for reading 4 contiguous `f32` elements.
        let va = unsafe { vld1q_f32(a.as_ptr().add(i)) };
        // SAFETY: `i + 4 <= len` guarantees `b.as_ptr().add(i)` is valid for reading 4 contiguous `f32` elements.
        let vb = unsafe { vld1q_f32(b.as_ptr().add(i)) };
        // SAFETY: NEON vector subtract on valid 128-bit float vectors.
        let diff = unsafe { vsubq_f32(va, vb) };

        // SAFETY: NEON multiply-accumulate on valid 128-bit float vectors.
        sum_v = unsafe { vfmaq_f32(sum_v, diff, diff) };

        i += 4;
    }

    // SAFETY: NEON vector reduction adding across 4 lanes of a valid 128-bit float register.
    let mut sum = unsafe { vaddvq_f32(sum_v) };

    while i < len {
        // SAFETY: Loop condition `i < len` guarantees index `i` is within bounds of slices `a` and `b` (`a.len() == b.len()`).
        let diff = unsafe { *a.get_unchecked(i) - *b.get_unchecked(i) };
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Computes the dot product distance between two `f32` slices using ARM NEON intrinsics.
///
/// # Safety
///
/// - The caller must ensure that the CPU supports `aarch64` NEON instructions.
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
pub(crate) unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Initializing 128-bit vector register with 0.0 float values under NEON target feature.
    let mut dot_v = unsafe { vdupq_n_f32(0.0) };

    while i + 4 <= len {
        // SAFETY: `i + 4 <= len` guarantees `a.as_ptr().add(i)` is valid for reading 4 contiguous `f32` elements.
        let va = unsafe { vld1q_f32(a.as_ptr().add(i)) };
        // SAFETY: `i + 4 <= len` guarantees `b.as_ptr().add(i)` is valid for reading 4 contiguous `f32` elements.
        let vb = unsafe { vld1q_f32(b.as_ptr().add(i)) };

        // SAFETY: NEON multiply-accumulate on valid 128-bit float vectors.
        dot_v = unsafe { vfmaq_f32(dot_v, va, vb) };

        i += 4;
    }

    // SAFETY: NEON vector reduction adding across 4 lanes of a valid 128-bit float register.
    let mut dot = unsafe { vaddvq_f32(dot_v) };

    while i < len {
        // SAFETY: Loop condition `i < len` guarantees index `i` is within bounds of slices `a` and `b` (`a.len() == b.len()`).
        dot += unsafe { *a.get_unchecked(i) * *b.get_unchecked(i) };
        i += 1;
    }

    -dot
}
