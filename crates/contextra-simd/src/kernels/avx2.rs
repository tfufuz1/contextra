// FILE-CONTEXT
// ZWECK: AVX2 SIMD-Intrinsics für f32 und f32_bytes Distanzberechnungen.
// INVARIANTEN: Kanonische Distanz nach SPEC D-122 (5b): 16 Teilsummen (Lane = Index mod 16),
//             kein FMA, feste paarweise Reduktion 16 -> 8 -> 4 -> 2 -> 1. Alle Kernel sind bitgleich.

//! # Modul `kernels::avx2`
//!
//! **Ring-Zugehörigkeit:** Ring 1 (Unsafe-Insel).
//!
//! **Zweck:**
//! Hochoptimierte Vektordistanzberechnung auf x86_64-Prozessoren mit AVX2.
//! Gemäß SPEC D-122 (5b) erfolgt die Berechnung ohne FMA und mit 16 Teilsummen,
//! sodass die Ergebnisse bitidentisch zur Skalar-Referenz sind.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::kernels::scalar::canonical_reduce_16;

/// Berechnet die Kosinus-Distanz zwischen zwei f32-Vektoren mittels AVX2.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx2 ist aktiv.
    let (mut dot_0, mut dot_1) = unsafe { (_mm256_setzero_ps(), _mm256_setzero_ps()) };
    // SAFETY: CPU-Feature avx2 ist aktiv.
    let (mut norm_a_0, mut norm_a_1) = unsafe { (_mm256_setzero_ps(), _mm256_setzero_ps()) };
    // SAFETY: CPU-Feature avx2 ist aktiv.
    let (mut norm_b_0, mut norm_b_1) = unsafe { (_mm256_setzero_ps(), _mm256_setzero_ps()) };

    let (a_ptr, b_ptr) = (a.as_ptr(), b.as_ptr());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets [i, i+16) liegen innerhalb der Grenzen [0, len) und avx2 ist aktiv.
        unsafe {
            let (va0, vb0, va1, vb1) = (
                _mm256_loadu_ps(a_ptr.add(i)),
                _mm256_loadu_ps(b_ptr.add(i)),
                _mm256_loadu_ps(a_ptr.add(i + 8)),
                _mm256_loadu_ps(b_ptr.add(i + 8)),
            );

            let (prod_dot0, prod_dot1) = (_mm256_mul_ps(va0, vb0), _mm256_mul_ps(va1, vb1));
            let (prod_na0, prod_na1) = (_mm256_mul_ps(va0, va0), _mm256_mul_ps(va1, va1));
            let (prod_nb0, prod_nb1) = (_mm256_mul_ps(vb0, vb0), _mm256_mul_ps(vb1, vb1));

            dot_0 = _mm256_add_ps(dot_0, prod_dot0);
            dot_1 = _mm256_add_ps(dot_1, prod_dot1);
            norm_a_0 = _mm256_add_ps(norm_a_0, prod_na0);
            norm_a_1 = _mm256_add_ps(norm_a_1, prod_na1);
            norm_b_0 = _mm256_add_ps(norm_b_0, prod_nb0);
            norm_b_1 = _mm256_add_ps(norm_b_1, prod_nb1);
        }
        i += 16;
    }

    let mut dot_lanes = [0.0f32; 16];
    let mut norm_a_lanes = [0.0f32; 16];
    let mut norm_b_lanes = [0.0f32; 16];

    // SAFETY: Auslesen der 256-Bit Register in 16-Element Float-Puffer.
    unsafe {
        _mm256_storeu_ps(dot_lanes.as_mut_ptr(), dot_0);
        _mm256_storeu_ps(dot_lanes.as_mut_ptr().add(8), dot_1);
        _mm256_storeu_ps(norm_a_lanes.as_mut_ptr(), norm_a_0);
        _mm256_storeu_ps(norm_a_lanes.as_mut_ptr().add(8), norm_a_1);
        _mm256_storeu_ps(norm_b_lanes.as_mut_ptr(), norm_b_0);
        _mm256_storeu_ps(norm_b_lanes.as_mut_ptr().add(8), norm_b_1);
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

/// Berechnet die Euklidische Distanz zwischen zwei f32-Vektoren mittels AVX2.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx2 ist aktiv.
    let (mut sum_0, mut sum_1) = unsafe { (_mm256_setzero_ps(), _mm256_setzero_ps()) };

    let (a_ptr, b_ptr) = (a.as_ptr(), b.as_ptr());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets [i, i+16) liegen innerhalb der Grenzen [0, len) und avx2 ist aktiv.
        unsafe {
            let (va0, vb0, va1, vb1) = (
                _mm256_loadu_ps(a_ptr.add(i)),
                _mm256_loadu_ps(b_ptr.add(i)),
                _mm256_loadu_ps(a_ptr.add(i + 8)),
                _mm256_loadu_ps(b_ptr.add(i + 8)),
            );

            let (diff0, diff1) = (_mm256_sub_ps(va0, vb0), _mm256_sub_ps(va1, vb1));

            let (prod0, prod1) = (_mm256_mul_ps(diff0, diff0), _mm256_mul_ps(diff1, diff1));

            sum_0 = _mm256_add_ps(sum_0, prod0);
            sum_1 = _mm256_add_ps(sum_1, prod1);
        }
        i += 16;
    }

    let mut sum_lanes = [0.0f32; 16];
    // SAFETY: Auslesen der 256-Bit Register in 16-Element Float-Puffer.
    unsafe {
        _mm256_storeu_ps(sum_lanes.as_mut_ptr(), sum_0);
        _mm256_storeu_ps(sum_lanes.as_mut_ptr().add(8), sum_1);
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

/// Berechnet die Skalarprodukt-Distanz zwischen zwei f32-Vektoren mittels AVX2.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx2 ist aktiv.
    let (mut dot_0, mut dot_1) = unsafe { (_mm256_setzero_ps(), _mm256_setzero_ps()) };

    let (a_ptr, b_ptr) = (a.as_ptr(), b.as_ptr());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets [i, i+16) liegen innerhalb der Grenzen [0, len) und avx2 ist aktiv.
        unsafe {
            let (va0, vb0, va1, vb1) = (
                _mm256_loadu_ps(a_ptr.add(i)),
                _mm256_loadu_ps(b_ptr.add(i)),
                _mm256_loadu_ps(a_ptr.add(i + 8)),
                _mm256_loadu_ps(b_ptr.add(i + 8)),
            );

            let (prod0, prod1) = (_mm256_mul_ps(va0, vb0), _mm256_mul_ps(va1, vb1));

            dot_0 = _mm256_add_ps(dot_0, prod0);
            dot_1 = _mm256_add_ps(dot_1, prod1);
        }
        i += 16;
    }

    let mut dot_lanes = [0.0f32; 16];
    // SAFETY: Auslesen der 256-Bit Register in 16-Element Float-Puffer.
    unsafe {
        _mm256_storeu_ps(dot_lanes.as_mut_ptr(), dot_0);
        _mm256_storeu_ps(dot_lanes.as_mut_ptr().add(8), dot_1);
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

/// Berechnet die Kosinus-Distanz zwischen f32-Slice und LE Byte-Slice mittels AVX2.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn cosine_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx2 ist aktiv.
    let (mut dot_0, mut dot_1) = unsafe { (_mm256_setzero_ps(), _mm256_setzero_ps()) };
    // SAFETY: CPU-Feature avx2 ist aktiv.
    let (mut norm_a_0, mut norm_a_1) = unsafe { (_mm256_setzero_ps(), _mm256_setzero_ps()) };
    // SAFETY: CPU-Feature avx2 ist aktiv.
    let (mut norm_b_0, mut norm_b_1) = unsafe { (_mm256_setzero_ps(), _mm256_setzero_ps()) };

    let (a_ptr, b_ptr) = (a.as_ptr(), b_bytes.as_ptr().cast::<f32>());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets liegen innerhalb valider Grenzen und avx2 ist aktiv.
        unsafe {
            let (va0, vb0, va1, vb1) = (
                _mm256_loadu_ps(a_ptr.add(i)),
                _mm256_loadu_ps(b_ptr.add(i)),
                _mm256_loadu_ps(a_ptr.add(i + 8)),
                _mm256_loadu_ps(b_ptr.add(i + 8)),
            );

            let (prod_dot0, prod_dot1) = (_mm256_mul_ps(va0, vb0), _mm256_mul_ps(va1, vb1));
            let (prod_na0, prod_na1) = (_mm256_mul_ps(va0, va0), _mm256_mul_ps(va1, va1));
            let (prod_nb0, prod_nb1) = (_mm256_mul_ps(vb0, vb0), _mm256_mul_ps(vb1, vb1));

            dot_0 = _mm256_add_ps(dot_0, prod_dot0);
            dot_1 = _mm256_add_ps(dot_1, prod_dot1);
            norm_a_0 = _mm256_add_ps(norm_a_0, prod_na0);
            norm_a_1 = _mm256_add_ps(norm_a_1, prod_na1);
            norm_b_0 = _mm256_add_ps(norm_b_0, prod_nb0);
            norm_b_1 = _mm256_add_ps(norm_b_1, prod_nb1);
        }
        i += 16;
    }

    let mut dot_lanes = [0.0f32; 16];
    let mut norm_a_lanes = [0.0f32; 16];
    let mut norm_b_lanes = [0.0f32; 16];

    // SAFETY: Auslesen der 256-Bit Register in 16-Element Float-Puffer.
    unsafe {
        _mm256_storeu_ps(dot_lanes.as_mut_ptr(), dot_0);
        _mm256_storeu_ps(dot_lanes.as_mut_ptr().add(8), dot_1);
        _mm256_storeu_ps(norm_a_lanes.as_mut_ptr(), norm_a_0);
        _mm256_storeu_ps(norm_a_lanes.as_mut_ptr().add(8), norm_a_1);
        _mm256_storeu_ps(norm_b_lanes.as_mut_ptr(), norm_b_0);
        _mm256_storeu_ps(norm_b_lanes.as_mut_ptr().add(8), norm_b_1);
    }

    while i < len {
        let lane = i % 16;
        let offset = i * 4;
        if let (Some(&x), Some(chunk)) = (a.get(i), b_bytes.get(offset..offset + 4)) {
            if let Ok(bytes) = chunk.try_into() {
                let b_val = f32::from_le_bytes(bytes);
                if let Some(slot) = dot_lanes.get_mut(lane) {
                    *slot += x * b_val;
                }
                if let Some(slot) = norm_a_lanes.get_mut(lane) {
                    *slot += x * x;
                }
                if let Some(slot) = norm_b_lanes.get_mut(lane) {
                    *slot += b_val * b_val;
                }
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

/// Berechnet die Euklidische Distanz zwischen f32-Slice und LE Byte-Slice mittels AVX2.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn euclidean_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx2 ist aktiv.
    let (mut sum_0, mut sum_1) = unsafe { (_mm256_setzero_ps(), _mm256_setzero_ps()) };

    let (a_ptr, b_ptr) = (a.as_ptr(), b_bytes.as_ptr().cast::<f32>());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets liegen innerhalb valider Grenzen und avx2 ist aktiv.
        unsafe {
            let (va0, vb0, va1, vb1) = (
                _mm256_loadu_ps(a_ptr.add(i)),
                _mm256_loadu_ps(b_ptr.add(i)),
                _mm256_loadu_ps(a_ptr.add(i + 8)),
                _mm256_loadu_ps(b_ptr.add(i + 8)),
            );

            let (diff0, diff1) = (_mm256_sub_ps(va0, vb0), _mm256_sub_ps(va1, vb1));

            let (prod0, prod1) = (_mm256_mul_ps(diff0, diff0), _mm256_mul_ps(diff1, diff1));

            sum_0 = _mm256_add_ps(sum_0, prod0);
            sum_1 = _mm256_add_ps(sum_1, prod1);
        }
        i += 16;
    }

    let mut sum_lanes = [0.0f32; 16];
    // SAFETY: Auslesen der 256-Bit Register in 16-Element Float-Puffer.
    unsafe {
        _mm256_storeu_ps(sum_lanes.as_mut_ptr(), sum_0);
        _mm256_storeu_ps(sum_lanes.as_mut_ptr().add(8), sum_1);
    }

    while i < len {
        let lane = i % 16;
        let offset = i * 4;
        if let (Some(&x), Some(chunk)) = (a.get(i), b_bytes.get(offset..offset + 4)) {
            if let Ok(bytes) = chunk.try_into() {
                let b_val = f32::from_le_bytes(bytes);
                let diff = x - b_val;
                if let Some(slot) = sum_lanes.get_mut(lane) {
                    *slot += diff * diff;
                }
            }
        }
        i += 1;
    }

    let sum = canonical_reduce_16(&sum_lanes);
    sum.sqrt()
}

/// Berechnet die Skalarprodukt-Distanz zwischen f32-Slice und LE Byte-Slice mittels AVX2.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn dot_product_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx2 ist aktiv.
    let (mut dot_0, mut dot_1) = unsafe { (_mm256_setzero_ps(), _mm256_setzero_ps()) };

    let (a_ptr, b_ptr) = (a.as_ptr(), b_bytes.as_ptr().cast::<f32>());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets liegen innerhalb valider Grenzen und avx2 ist aktiv.
        unsafe {
            let (va0, vb0, va1, vb1) = (
                _mm256_loadu_ps(a_ptr.add(i)),
                _mm256_loadu_ps(b_ptr.add(i)),
                _mm256_loadu_ps(a_ptr.add(i + 8)),
                _mm256_loadu_ps(b_ptr.add(i + 8)),
            );

            let (prod0, prod1) = (_mm256_mul_ps(va0, vb0), _mm256_mul_ps(va1, vb1));

            dot_0 = _mm256_add_ps(dot_0, prod0);
            dot_1 = _mm256_add_ps(dot_1, prod1);
        }
        i += 16;
    }

    let mut dot_lanes = [0.0f32; 16];
    // SAFETY: Auslesen der 256-Bit Register in 16-Element Float-Puffer.
    unsafe {
        _mm256_storeu_ps(dot_lanes.as_mut_ptr(), dot_0);
        _mm256_storeu_ps(dot_lanes.as_mut_ptr().add(8), dot_1);
    }

    while i < len {
        let lane = i % 16;
        let offset = i * 4;
        if let (Some(&x), Some(chunk)) = (a.get(i), b_bytes.get(offset..offset + 4)) {
            if let Ok(bytes) = chunk.try_into() {
                let b_val = f32::from_le_bytes(bytes);
                if let Some(slot) = dot_lanes.get_mut(lane) {
                    *slot += x * b_val;
                }
            }
        }
        i += 1;
    }

    let dot = canonical_reduce_16(&dot_lanes);
    -dot
}
