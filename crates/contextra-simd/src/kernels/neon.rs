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
pub(crate) unsafe fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32 {
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
pub(crate) unsafe fn euclidean_distance_neon(a: &[f32], b: &[f32]) -> f32 {
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
pub(crate) unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
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

#[cfg(all(test, target_arch = "aarch64"))]
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
    }

    fn check_neon_support() -> bool {
        let has_neon = std::arch::is_aarch64_feature_detected!("neon");
        if !has_neon {
            eprintln!("SKIPPED: NEON feature not supported on host CPU");
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
    fn test_neon_cosine_distance_direct() {
        if !check_neon_support() {
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
                // SAFETY: CPU feature neon verified via check_neon_support.
                let simd = unsafe { cosine_distance_neon(a, b) };

                assert!(
                    approx_eq(s, simd, 1e-4),
                    "Cosine NEON mismatch at len {len}, offset {offset}: scalar={s}, simd={simd}"
                );
            }
        }
    }

    #[test]
    fn test_neon_euclidean_distance_direct() {
        if !check_neon_support() {
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
                // SAFETY: CPU feature neon verified via check_neon_support.
                let simd = unsafe { euclidean_distance_neon(a, b) };

                assert!(
                    approx_eq(s, simd, 1e-3),
                    "Euclidean NEON mismatch at len {len}, offset {offset}: scalar={s}, simd={simd}"
                );
            }
        }
    }

    #[test]
    fn test_neon_dot_product_direct() {
        if !check_neon_support() {
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
                // SAFETY: CPU feature neon verified via check_neon_support.
                let simd = unsafe { dot_product_neon(a, b) };

                assert!(
                    approx_eq(s, simd, 1e-3),
                    "Dot product NEON mismatch at len {len}, offset {offset}: scalar={s}, simd={simd}"
                );
            }
        }
    }

    #[test]
    fn test_neon_special_float_values() {
        if !check_neon_support() {
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
            let a = vec![v; 16];
            let b = vec![1.0f32; 16];

            let s_cos = cosine_distance_scalar(&a, &b);
            // SAFETY: CPU feature neon verified via check_neon_support.
            let simd_cos = unsafe { cosine_distance_neon(&a, &b) };
            assert!(approx_eq(s_cos, simd_cos, 1e-4));

            let s_euc = euclidean_distance_scalar(&a, &b);
            // SAFETY: CPU feature neon verified via check_neon_support.
            let simd_euc = unsafe { euclidean_distance_neon(&a, &b) };
            assert!(approx_eq(s_euc, simd_euc, 1e-3));

            let s_dot = dot_product_scalar(&a, &b);
            // SAFETY: CPU feature neon verified via check_neon_support.
            let simd_dot = unsafe { dot_product_neon(&a, &b) };
            assert!(approx_eq(s_dot, simd_dot, 1e-3));
        }
    }
}
