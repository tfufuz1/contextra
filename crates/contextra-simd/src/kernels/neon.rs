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
        // SAFETY:
        // 1. Feature: Target feature `neon` is enabled via function attribute.
        // 2. Length/Tail: `i + 4 <= len` where `len == a.len() == b.len()` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)` are valid for reading 4 contiguous `f32` elements.
        // 3. Alignment: `vld1q_f32` handles unaligned float pointer reads safely.
        let (va, vb) = unsafe { (vld1q_f32(a.as_ptr().add(i)), vld1q_f32(b.as_ptr().add(i))) };

        // SAFETY: Target feature `neon` is enabled and vector registers are valid `float32x4_t`.
        dot_v = unsafe { vfmaq_f32(dot_v, va, vb) };
        // SAFETY: Target feature `neon` is enabled and vector registers are valid `float32x4_t`.
        norm_a_v = unsafe { vfmaq_f32(norm_a_v, va, va) };
        // SAFETY: Target feature `neon` is enabled and vector registers are valid `float32x4_t`.
        norm_b_v = unsafe { vfmaq_f32(norm_b_v, vb, vb) };

        i += 4;
    }

    // SAFETY: Target feature `neon` is enabled and vector registers are valid `float32x4_t`.
    let mut dot = unsafe { vaddvq_f32(dot_v) };
    // SAFETY: Target feature `neon` is enabled and vector registers are valid `float32x4_t`.
    let mut norm_a = unsafe { vaddvq_f32(norm_a_v) };
    // SAFETY: Target feature `neon` is enabled and vector registers are valid `float32x4_t`.
    let mut norm_b = unsafe { vaddvq_f32(norm_b_v) };

    while i < len {
        // SAFETY: Loop condition `i < len` guarantees index `i` is within bounds of slice `a` and `b` (`a.len() == b.len()`).
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

/// Computes the Euclidean distance between two `f32` slices using ARM NEON intrinsics.
///
/// # Safety
///
/// - The caller must ensure that the CPU supports `aarch64` NEON instructions.
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub(crate) unsafe fn euclidean_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Initializing 128-bit vector register with 0.0 float values under NEON target feature.
    let mut sum_v = unsafe { vdupq_n_f32(0.0) };

    while i + 4 <= len {
        // SAFETY:
        // 1. Feature: Target feature `neon` is enabled via function attribute.
        // 2. Length/Tail: `i + 4 <= len` where `len == a.len() == b.len()` guarantees 4 readable contiguous `f32` elements.
        // 3. Alignment: `vld1q_f32` handles unaligned float pointer reads safely.
        let (va, vb) = unsafe { (vld1q_f32(a.as_ptr().add(i)), vld1q_f32(b.as_ptr().add(i))) };
        // SAFETY: Target feature `neon` is enabled and registers are valid `float32x4_t`.
        let diff = unsafe { vsubq_f32(va, vb) };

        // SAFETY: Target feature `neon` is enabled and registers are valid `float32x4_t`.
        sum_v = unsafe { vfmaq_f32(sum_v, diff, diff) };

        i += 4;
    }

    // SAFETY: Target feature `neon` is enabled and register is valid `float32x4_t`.
    let mut sum = unsafe { vaddvq_f32(sum_v) };

    while i < len {
        // SAFETY: Loop condition `i < len` guarantees index `i` is within bounds of slices `a` and `b` (`a.len() == b.len()`).
        let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };
        let diff = x - y;
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
#[target_feature(enable = "neon")]
pub(crate) unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    // SAFETY: Initializing 128-bit vector register with 0.0 float values under NEON target feature.
    let mut dot_v = unsafe { vdupq_n_f32(0.0) };

    while i + 4 <= len {
        // SAFETY:
        // 1. Feature: Target feature `neon` is enabled via function attribute.
        // 2. Length/Tail: `i + 4 <= len` where `len == a.len() == b.len()` guarantees 4 readable contiguous `f32` elements.
        // 3. Alignment: `vld1q_f32` handles unaligned float pointer reads safely.
        let (va, vb) = unsafe { (vld1q_f32(a.as_ptr().add(i)), vld1q_f32(b.as_ptr().add(i))) };

        // SAFETY: Target feature `neon` is enabled and registers are valid `float32x4_t`.
        dot_v = unsafe { vfmaq_f32(dot_v, va, vb) };

        i += 4;
    }

    // SAFETY: Target feature `neon` is enabled and register is valid `float32x4_t`.
    let mut dot = unsafe { vaddvq_f32(dot_v) };

    while i < len {
        // SAFETY: Loop condition `i < len` guarantees index `i` is within bounds of slices `a` and `b` (`a.len() == b.len()`).
        let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };
        dot += x * y;
        i += 1;
    }

    -dot
}
