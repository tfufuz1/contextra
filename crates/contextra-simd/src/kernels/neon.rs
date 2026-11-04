// FILE-CONTEXT
// ZWECK: ARM NEON SIMD-Intrinsics für f32 Distanzberechnungen.
// INVARIANTEN: Kanonische Distanz nach SPEC D-122 (5b): 16 Teilsummen (Lane = Index mod 16),
//             kein FMA, feste paarweise Reduktion 16 -> 8 -> 4 -> 2 -> 1. Alle Kernel sind bitgleich.

//! # Modul `kernels::neon`
//!
//! **Ring-Zugehörigkeit:** Ring 1 (Unsafe-Insel).
//!
//! **Zweck:**
//! Hochoptimierte Vektordistanzberechnung auf ARM AArch64-Architekturen mit NEON-Befehlssatz.
//! Gemäß SPEC D-122 (5b) erfolgt die Berechnung ohne FMA und mit 16 Teilsummen.

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

#[cfg(target_arch = "aarch64")]
use crate::kernels::scalar::canonical_reduce_16;

/// Berechnet die Kosinus-Distanz zwischen zwei f32-Vektoren mittels ARM NEON.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub(crate) unsafe fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature neon ist aktiv.
    let (mut dot_0, mut dot_1, mut dot_2, mut dot_3) = unsafe {
        (
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
        )
    };
    // SAFETY: CPU-Feature neon ist aktiv.
    let (mut norm_a_0, mut norm_a_1, mut norm_a_2, mut norm_a_3) = unsafe {
        (
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
        )
    };
    // SAFETY: CPU-Feature neon ist aktiv.
    let (mut norm_b_0, mut norm_b_1, mut norm_b_2, mut norm_b_3) = unsafe {
        (
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
        )
    };

    let (a_ptr, b_ptr) = (a.as_ptr(), b.as_ptr());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets liegen innerhalb valider Grenzen und neon ist aktiv.
        unsafe {
            let (va0, vb0, va1, vb1) = (
                vld1q_f32(a_ptr.add(i)),
                vld1q_f32(b_ptr.add(i)),
                vld1q_f32(a_ptr.add(i + 4)),
                vld1q_f32(b_ptr.add(i + 4)),
            );
            let (va2, vb2, va3, vb3) = (
                vld1q_f32(a_ptr.add(i + 8)),
                vld1q_f32(b_ptr.add(i + 8)),
                vld1q_f32(a_ptr.add(i + 12)),
                vld1q_f32(b_ptr.add(i + 12)),
            );

            let p_dot0 = vmulq_f32(va0, vb0);
            let p_dot1 = vmulq_f32(va1, vb1);
            let p_dot2 = vmulq_f32(va2, vb2);
            let p_dot3 = vmulq_f32(va3, vb3);

            let p_na0 = vmulq_f32(va0, va0);
            let p_na1 = vmulq_f32(va1, va1);
            let p_na2 = vmulq_f32(va2, va2);
            let p_na3 = vmulq_f32(va3, va3);

            let p_nb0 = vmulq_f32(vb0, vb0);
            let p_nb1 = vmulq_f32(vb1, vb1);
            let p_nb2 = vmulq_f32(vb2, vb2);
            let p_nb3 = vmulq_f32(vb3, vb3);

            dot_0 = vaddq_f32(dot_0, p_dot0);
            dot_1 = vaddq_f32(dot_1, p_dot1);
            dot_2 = vaddq_f32(dot_2, p_dot2);
            dot_3 = vaddq_f32(dot_3, p_dot3);

            norm_a_0 = vaddq_f32(norm_a_0, p_na0);
            norm_a_1 = vaddq_f32(norm_a_1, p_na1);
            norm_a_2 = vaddq_f32(norm_a_2, p_na2);
            norm_a_3 = vaddq_f32(norm_a_3, p_na3);

            norm_b_0 = vaddq_f32(norm_b_0, p_nb0);
            norm_b_1 = vaddq_f32(norm_b_1, p_nb1);
            norm_b_2 = vaddq_f32(norm_b_2, p_nb2);
            norm_b_3 = vaddq_f32(norm_b_3, p_nb3);
        }
        i += 16;
    }

    let mut dot_lanes = [0.0f32; 16];
    let mut norm_a_lanes = [0.0f32; 16];
    let mut norm_b_lanes = [0.0f32; 16];

    // SAFETY: Auslesen der 128-Bit Register in 16-Element Float-Puffer.
    unsafe {
        vst1q_f32(dot_lanes.as_mut_ptr(), dot_0);
        vst1q_f32(dot_lanes.as_mut_ptr().add(4), dot_1);
        vst1q_f32(dot_lanes.as_mut_ptr().add(8), dot_2);
        vst1q_f32(dot_lanes.as_mut_ptr().add(12), dot_3);

        vst1q_f32(norm_a_lanes.as_mut_ptr(), norm_a_0);
        vst1q_f32(norm_a_lanes.as_mut_ptr().add(4), norm_a_1);
        vst1q_f32(norm_a_lanes.as_mut_ptr().add(8), norm_a_2);
        vst1q_f32(norm_a_lanes.as_mut_ptr().add(12), norm_a_3);

        vst1q_f32(norm_b_lanes.as_mut_ptr(), norm_b_0);
        vst1q_f32(norm_b_lanes.as_mut_ptr().add(4), norm_b_1);
        vst1q_f32(norm_b_lanes.as_mut_ptr().add(8), norm_b_2);
        vst1q_f32(norm_b_lanes.as_mut_ptr().add(12), norm_b_3);
    }

    while i < len {
        let lane = i % 16;
        if let (Some(&x), Some(&y)) = (a.get(i), b.get(i)) {
            if let Some(slot) = dot_lanes.get_mut(lane) {
                *slot += x * y;
            }
            if let Some(slot) = norm_a_lanes.get_mut(lane) {
                *slot += x * x;
            }
            if let Some(slot) = norm_b_lanes.get_mut(lane) {
                *slot += y * y;
            }
        }
        i += 1;
    }

    let dot = canonical_reduce_16(&dot_lanes);
    let norm_a = canonical_reduce_16(&norm_a_lanes);
    let norm_b = canonical_reduce_16(&norm_b_lanes);

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    let sim = (dot / denom).clamp(-1.0, 1.0);
    1.0 - sim
}

/// Berechnet die Euklidische Distanz zwischen zwei f32-Vektoren mittels ARM NEON.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub(crate) unsafe fn euclidean_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature neon ist aktiv.
    let (mut sum_0, mut sum_1, mut sum_2, mut sum_3) = unsafe {
        (
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
        )
    };

    let (a_ptr, b_ptr) = (a.as_ptr(), b.as_ptr());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets liegen innerhalb valider Grenzen und neon ist aktiv.
        unsafe {
            let (va0, vb0, va1, vb1) = (
                vld1q_f32(a_ptr.add(i)),
                vld1q_f32(b_ptr.add(i)),
                vld1q_f32(a_ptr.add(i + 4)),
                vld1q_f32(b_ptr.add(i + 4)),
            );
            let (va2, vb2, va3, vb3) = (
                vld1q_f32(a_ptr.add(i + 8)),
                vld1q_f32(b_ptr.add(i + 8)),
                vld1q_f32(a_ptr.add(i + 12)),
                vld1q_f32(b_ptr.add(i + 12)),
            );

            let diff0 = vsubq_f32(va0, vb0);
            let diff1 = vsubq_f32(va1, vb1);
            let diff2 = vsubq_f32(va2, vb2);
            let diff3 = vsubq_f32(va3, vb3);

            let p0 = vmulq_f32(diff0, diff0);
            let p1 = vmulq_f32(diff1, diff1);
            let p2 = vmulq_f32(diff2, diff2);
            let p3 = vmulq_f32(diff3, diff3);

            sum_0 = vaddq_f32(sum_0, p0);
            sum_1 = vaddq_f32(sum_1, p1);
            sum_2 = vaddq_f32(sum_2, p2);
            sum_3 = vaddq_f32(sum_3, p3);
        }
        i += 16;
    }

    let mut sum_lanes = [0.0f32; 16];
    // SAFETY: Auslesen der 128-Bit Register in 16-Element Float-Puffer.
    unsafe {
        vst1q_f32(sum_lanes.as_mut_ptr(), sum_0);
        vst1q_f32(sum_lanes.as_mut_ptr().add(4), sum_1);
        vst1q_f32(sum_lanes.as_mut_ptr().add(8), sum_2);
        vst1q_f32(sum_lanes.as_mut_ptr().add(12), sum_3);
    }

    while i < len {
        let lane = i % 16;
        if let (Some(&x), Some(&y)) = (a.get(i), b.get(i)) {
            let diff = x - y;
            if let Some(slot) = sum_lanes.get_mut(lane) {
                *slot += diff * diff;
            }
        }
        i += 1;
    }

    let sum = canonical_reduce_16(&sum_lanes);
    sum.sqrt()
}

/// Berechnet die Skalarprodukt-Distanz zwischen zwei f32-Vektoren mittels ARM NEON.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub(crate) unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature neon ist aktiv.
    let (mut dot_0, mut dot_1, mut dot_2, mut dot_3) = unsafe {
        (
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
            vdupq_n_f32(0.0),
        )
    };

    let (a_ptr, b_ptr) = (a.as_ptr(), b.as_ptr());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets liegen innerhalb valider Grenzen und neon ist aktiv.
        unsafe {
            let (va0, vb0, va1, vb1) = (
                vld1q_f32(a_ptr.add(i)),
                vld1q_f32(b_ptr.add(i)),
                vld1q_f32(a_ptr.add(i + 4)),
                vld1q_f32(b_ptr.add(i + 4)),
            );
            let (va2, vb2, va3, vb3) = (
                vld1q_f32(a_ptr.add(i + 8)),
                vld1q_f32(b_ptr.add(i + 8)),
                vld1q_f32(a_ptr.add(i + 12)),
                vld1q_f32(b_ptr.add(i + 12)),
            );

            let p0 = vmulq_f32(va0, vb0);
            let p1 = vmulq_f32(va1, vb1);
            let p2 = vmulq_f32(va2, vb2);
            let p3 = vmulq_f32(va3, vb3);

            dot_0 = vaddq_f32(dot_0, p0);
            dot_1 = vaddq_f32(dot_1, p1);
            dot_2 = vaddq_f32(dot_2, p2);
            dot_3 = vaddq_f32(dot_3, p3);
        }
        i += 16;
    }

    let mut dot_lanes = [0.0f32; 16];
    // SAFETY: Auslesen der 128-Bit Register in 16-Element Float-Puffer.
    unsafe {
        vst1q_f32(dot_lanes.as_mut_ptr(), dot_0);
        vst1q_f32(dot_lanes.as_mut_ptr().add(4), dot_1);
        vst1q_f32(dot_lanes.as_mut_ptr().add(8), dot_2);
        vst1q_f32(dot_lanes.as_mut_ptr().add(12), dot_3);
    }

    while i < len {
        let lane = i % 16;
        if let (Some(&x), Some(&y)) = (a.get(i), b.get(i)) {
            if let Some(slot) = dot_lanes.get_mut(lane) {
                *slot += x * y;
            }
        }
        i += 1;
    }

    let dot = canonical_reduce_16(&dot_lanes);
    -dot
}
