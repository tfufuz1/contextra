// FILE-CONTEXT
// ZWECK: ARM NEON SIMD-Intrinsics für f32 Distanzberechnungen.
// INVARIANTEN: Target-Architektur aarch64 / neon.

#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unused_unsafe)]

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// Computes cosine distance between two `f32` slices using ARM NEON SIMD intrinsics.
///
/// # Safety
///
/// - Target architecture must be `aarch64` and the `neon` target feature must be enabled and supported by the CPU.
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
/// - Vector operations process elements in chunks of 4. The loop condition `i + 4 <= len` ensures raw pointer arithmetic
///   (`a.as_ptr().add(i)` and `b.as_ptr().add(i)`) remains within valid slice bounds. Pointers are loaded via `vld1q_f32`,
///   which supports unaligned memory access on AArch64.
/// - Remaining tail elements (`len % 4`) are accessed element-by-element where `i < len == a.len() == b.len()`.
/// - For empty input slices (`len == 0`), no vector or tail reads occur and 1.0 is returned safely.
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
pub unsafe fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let (mut dot_v, mut norm_a_v, mut norm_b_v) = (
        vdupq_n_f32(0.0),
        vdupq_n_f32(0.0),
        vdupq_n_f32(0.0),
    );

    while i + 4 <= len {
        // SAFETY: The loop condition `i + 4 <= len` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)`
        // are valid for reading 4 contiguous `f32` values within slice bounds (`len == a.len() == b.len()`).
        // `vld1q_f32` performs unaligned vector loads.
        let (va, vb) = unsafe {
            (
                vld1q_f32(a.as_ptr().add(i)),
                vld1q_f32(b.as_ptr().add(i)),
            )
        };

        dot_v = vfmaq_f32(dot_v, va, vb);
        norm_a_v = vfmaq_f32(norm_a_v, va, va);
        norm_b_v = vfmaq_f32(norm_b_v, vb, vb);

        i += 4;
    }

    let (mut dot, mut norm_a, mut norm_b) = (
        vaddvq_f32(dot_v),
        vaddvq_f32(norm_a_v),
        vaddvq_f32(norm_b_v),
    );

    while i < len {
        // SAFETY: `i < len` guarantees `i` is strictly within valid bounds of slices `a` and `b`
        // since `len == a.len() == b.len()`.
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

/// Computes Euclidean distance between two `f32` slices using ARM NEON SIMD intrinsics.
///
/// # Safety
///
/// - Target architecture must be `aarch64` and the `neon` target feature must be enabled and supported by the CPU.
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
/// - Vector operations process elements in chunks of 4. The loop condition `i + 4 <= len` ensures raw pointer arithmetic
///   (`a.as_ptr().add(i)` and `b.as_ptr().add(i)`) remains within valid slice bounds. Pointers are loaded via `vld1q_f32`,
///   which supports unaligned memory access on AArch64.
/// - Remaining tail elements (`len % 4`) are accessed element-by-element where `i < len == a.len() == b.len()`.
/// - For empty input slices (`len == 0`), no vector or tail reads occur and 0.0 is returned safely.
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
pub unsafe fn euclidean_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut sum_v = vdupq_n_f32(0.0);

    while i + 4 <= len {
        // SAFETY: The loop condition `i + 4 <= len` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)`
        // are valid for reading 4 contiguous `f32` values within slice bounds (`len == a.len() == b.len()`).
        // `vld1q_f32` performs unaligned vector loads.
        let (va, vb) = unsafe {
            (
                vld1q_f32(a.as_ptr().add(i)),
                vld1q_f32(b.as_ptr().add(i)),
            )
        };
        let diff = vsubq_f32(va, vb);

        sum_v = vfmaq_f32(sum_v, diff, diff);

        i += 4;
    }

    let mut sum = vaddvq_f32(sum_v);

    while i < len {
        // SAFETY: `i < len` guarantees `i` is strictly within valid bounds of slices `a` and `b`
        // since `len == a.len() == b.len()`.
        let diff = unsafe { *a.get_unchecked(i) - *b.get_unchecked(i) };
        sum += diff * diff;
        i += 1;
    }

    sum.sqrt()
}

/// Computes negated dot product distance between two `f32` slices using ARM NEON SIMD intrinsics.
///
/// # Safety
///
/// - Target architecture must be `aarch64` and the `neon` target feature must be enabled and supported by the CPU.
/// - Slices `a` and `b` must have equal lengths (`a.len() == b.len()`).
/// - Vector operations process elements in chunks of 4. The loop condition `i + 4 <= len` ensures raw pointer arithmetic
///   (`a.as_ptr().add(i)` and `b.as_ptr().add(i)`) remains within valid slice bounds. Pointers are loaded via `vld1q_f32`,
///   which supports unaligned memory access on AArch64.
/// - Remaining tail elements (`len % 4`) are accessed element-by-element where `i < len == a.len() == b.len()`.
/// - For empty input slices (`len == 0`), no vector or tail reads occur and -0.0 is returned safely.
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[target_feature(enable = "neon")]
pub unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut i = 0;

    let mut dot_v = vdupq_n_f32(0.0);

    while i + 4 <= len {
        // SAFETY: The loop condition `i + 4 <= len` guarantees `a.as_ptr().add(i)` and `b.as_ptr().add(i)`
        // are valid for reading 4 contiguous `f32` values within slice bounds (`len == a.len() == b.len()`).
        // `vld1q_f32` performs unaligned vector loads.
        let (va, vb) = unsafe {
            (
                vld1q_f32(a.as_ptr().add(i)),
                vld1q_f32(b.as_ptr().add(i)),
            )
        };

        dot_v = vfmaq_f32(dot_v, va, vb);

        i += 4;
    }

    let mut dot = vaddvq_f32(dot_v);

    while i < len {
        // SAFETY: `i < len` guarantees `i` is strictly within valid bounds of slices `a` and `b`
        // since `len == a.len() == b.len()`.
        dot += unsafe { *a.get_unchecked(i) * *b.get_unchecked(i) };
        i += 1;
    }

    -dot
}
