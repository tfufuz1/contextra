// FILE-CONTEXT
// ZWECK: AVX-512 SIMD-Intrinsics für f32 und f32_bytes Distanzberechnungen.
// INVARIANTEN: Kanonische Distanz nach SPEC D-122 (5b): 16 Teilsummen (Lane = Index mod 16),
//             kein FMA, feste paarweise Reduktion 16 -> 8 -> 4 -> 2 -> 1. Alle Kernel sind bitgleich.

//! # Modul `kernels::avx512`
//!
//! **Ring-Zugehörigkeit:** Ring 1 (Unsafe-Insel).
//!
//! **Zweck:**
//! AVX-512 Vektordistanzberechnung auf x86_64-Prozessoren.
//! Ein 512-Bit ZMM-Register fasst genau 16 `f32`-Werte (16 x 32-Bit = 512 Bit).
//! Gemäß SPEC D-122 (5b) erfolgt die Berechnung ohne FMA und mit 16 Teilsummen.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::kernels::scalar::canonical_reduce_16;

/// Berechnet die Kosinus-Distanz zwischen zwei f32-Vektoren mittels AVX-512.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn cosine_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx512f ist aktiv.
    let (mut dot_v, mut norm_a_v, mut norm_b_v) = unsafe {
        (
            _mm512_setzero_ps(),
            _mm512_setzero_ps(),
            _mm512_setzero_ps(),
        )
    };

    let (a_ptr, b_ptr) = (a.as_ptr(), b.as_ptr());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets [i, i+16) liegen innerhalb der Grenzen [0, len) und avx512f ist aktiv.
        unsafe {
            let (va, vb) = (_mm512_loadu_ps(a_ptr.add(i)), _mm512_loadu_ps(b_ptr.add(i)));

            let (prod_dot, prod_na, prod_nb) = (
                _mm512_mul_ps(va, vb),
                _mm512_mul_ps(va, va),
                _mm512_mul_ps(vb, vb),
            );

            dot_v = _mm512_add_ps(dot_v, prod_dot);
            norm_a_v = _mm512_add_ps(norm_a_v, prod_na);
            norm_b_v = _mm512_add_ps(norm_b_v, prod_nb);
        }
        i += 16;
    }

    let mut dot_lanes = [0.0f32; 16];
    let mut norm_a_lanes = [0.0f32; 16];
    let mut norm_b_lanes = [0.0f32; 16];

    // SAFETY: Auslesen der 512-Bit Register in 16-Element Float-Puffer.
    unsafe {
        _mm512_storeu_ps(dot_lanes.as_mut_ptr(), dot_v);
        _mm512_storeu_ps(norm_a_lanes.as_mut_ptr(), norm_a_v);
        _mm512_storeu_ps(norm_b_lanes.as_mut_ptr(), norm_b_v);
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

/// Berechnet die Euklidische Distanz zwischen zwei f32-Vektoren mittels AVX-512.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx512f ist aktiv.
    let mut sum_v = unsafe { _mm512_setzero_ps() };
    let (a_ptr, b_ptr) = (a.as_ptr(), b.as_ptr());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets [i, i+16) liegen innerhalb der Grenzen [0, len) und avx512f ist aktiv.
        unsafe {
            let (va, vb) = (_mm512_loadu_ps(a_ptr.add(i)), _mm512_loadu_ps(b_ptr.add(i)));
            let diff = _mm512_sub_ps(va, vb);
            let prod = _mm512_mul_ps(diff, diff);

            sum_v = _mm512_add_ps(sum_v, prod);
        }
        i += 16;
    }

    let mut sum_lanes = [0.0f32; 16];
    // SAFETY: Auslesen des 512-Bit Registers in 16-Element Float-Puffer.
    unsafe {
        _mm512_storeu_ps(sum_lanes.as_mut_ptr(), sum_v);
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

/// Berechnet die Skalarprodukt-Distanz zwischen zwei f32-Vektoren mittels AVX-512.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn dot_product_avx512(a: &[f32], b: &[f32]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx512f ist aktiv.
    let mut dot_v = unsafe { _mm512_setzero_ps() };
    let (a_ptr, b_ptr) = (a.as_ptr(), b.as_ptr());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets [i, i+16) liegen innerhalb der Grenzen [0, len) und avx512f ist aktiv.
        unsafe {
            let (va, vb) = (_mm512_loadu_ps(a_ptr.add(i)), _mm512_loadu_ps(b_ptr.add(i)));
            let prod = _mm512_mul_ps(va, vb);

            dot_v = _mm512_add_ps(dot_v, prod);
        }
        i += 16;
    }

    let mut dot_lanes = [0.0f32; 16];
    // SAFETY: Auslesen des 512-Bit Registers in 16-Element Float-Puffer.
    unsafe {
        _mm512_storeu_ps(dot_lanes.as_mut_ptr(), dot_v);
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

/// Berechnet die Kosinus-Distanz zwischen f32-Slice und LE Byte-Slice mittels AVX-512.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn cosine_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx512f ist aktiv.
    let (mut dot_v, mut norm_a_v, mut norm_b_v) = unsafe {
        (
            _mm512_setzero_ps(),
            _mm512_setzero_ps(),
            _mm512_setzero_ps(),
        )
    };

    let (a_ptr, b_ptr) = (a.as_ptr(), b_bytes.as_ptr().cast::<f32>());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets liegen innerhalb valider Grenzen und avx512f ist aktiv.
        unsafe {
            let (va, vb) = (_mm512_loadu_ps(a_ptr.add(i)), _mm512_loadu_ps(b_ptr.add(i)));

            let (prod_dot, prod_na, prod_nb) = (
                _mm512_mul_ps(va, vb),
                _mm512_mul_ps(va, va),
                _mm512_mul_ps(vb, vb),
            );

            dot_v = _mm512_add_ps(dot_v, prod_dot);
            norm_a_v = _mm512_add_ps(norm_a_v, prod_na);
            norm_b_v = _mm512_add_ps(norm_b_v, prod_nb);
        }
        i += 16;
    }

    let mut dot_lanes = [0.0f32; 16];
    let mut norm_a_lanes = [0.0f32; 16];
    let mut norm_b_lanes = [0.0f32; 16];

    // SAFETY: Auslesen der 512-Bit Register in 16-Element Float-Puffer.
    unsafe {
        _mm512_storeu_ps(dot_lanes.as_mut_ptr(), dot_v);
        _mm512_storeu_ps(norm_a_lanes.as_mut_ptr(), norm_a_v);
        _mm512_storeu_ps(norm_b_lanes.as_mut_ptr(), norm_b_v);
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

/// Berechnet die Euklidische Distanz zwischen f32-Slice und LE Byte-Slice mittels AVX-512.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn euclidean_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx512f ist aktiv.
    let mut sum_v = unsafe { _mm512_setzero_ps() };
    let (a_ptr, b_ptr) = (a.as_ptr(), b_bytes.as_ptr().cast::<f32>());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets liegen innerhalb valider Grenzen und avx512f ist aktiv.
        unsafe {
            let (va, vb) = (_mm512_loadu_ps(a_ptr.add(i)), _mm512_loadu_ps(b_ptr.add(i)));
            let diff = _mm512_sub_ps(va, vb);
            let prod = _mm512_mul_ps(diff, diff);

            sum_v = _mm512_add_ps(sum_v, prod);
        }
        i += 16;
    }

    let mut sum_lanes = [0.0f32; 16];
    // SAFETY: Auslesen des 512-Bit Registers in 16-Element Float-Puffer.
    unsafe {
        _mm512_storeu_ps(sum_lanes.as_mut_ptr(), sum_v);
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

/// Berechnet die Skalarprodukt-Distanz zwischen f32-Slice und LE Byte-Slice mittels AVX-512.
///
/// # Safety: CPU-Erweiterung und Pufferlängen müssen vorab validiert sein.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn dot_product_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {
    let (len, mut i) = (a.len(), 0);

    // SAFETY: CPU-Feature avx512f ist aktiv.
    let mut dot_v = unsafe { _mm512_setzero_ps() };
    let (a_ptr, b_ptr) = (a.as_ptr(), b_bytes.as_ptr().cast::<f32>());

    while i + 16 <= len {
        // SAFETY: Pointer-Offsets liegen innerhalb valider Grenzen und avx512f ist aktiv.
        unsafe {
            let (va, vb) = (_mm512_loadu_ps(a_ptr.add(i)), _mm512_loadu_ps(b_ptr.add(i)));
            let prod = _mm512_mul_ps(va, vb);

            dot_v = _mm512_add_ps(dot_v, prod);
        }
        i += 16;
    }

    let mut dot_lanes = [0.0f32; 16];
    // SAFETY: Auslesen des 512-Bit Registers in 16-Element Float-Puffer.
    unsafe {
        _mm512_storeu_ps(dot_lanes.as_mut_ptr(), dot_v);
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
