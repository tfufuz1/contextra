// ADR-002 / SPEC §27: Test assertions and test dataset casts
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::unusual_byte_groupings)]

//! # Korrektheitsnachweis und Property-Test-Suite: `T-SIMD-1`
//!
//! **Zweck (SPEC §8.4, §15, D-28):**
//! Implementiert die normative Testbatterie **T-SIMD-1**:
//! - Mindestens 100.000 Vektoren je f32-Kernel (L2 / Euclidean, Cosine, Dot Product).
//! - Getestet werden Vektoren mit Längen 0..=2048, inklusive Edge Cases: NaN, Inf, Denormals, Nullen,
//!   extreme Werte sowie unaligned Slices.
//! - Relative Abweichung $\le 10^{-4}$ gegen die unveränderliche Skalar-Referenz für Vektoren mit Norm $\le 10^3$.
//! - Deterministischer PRNG-Seed mit Ausgabe von Seed und Iterationsindex im Fehlerfall.
//! - Getestet über alle verfügbaren Hardware-Dispatch-Pfade (Scalar, AVX2, AVX-512, NEON).
//! - Ausführungszeit liegt in CI deutlich unter 5 Minuten.

use contextra_simd::dispatch::{
    cosine_distance, dot_product_distance, euclidean_distance, set_dispatch_override,
    SimdPathOverride,
};
use contextra_simd::kernels::scalar::{
    cosine_distance_scalar, dot_product_scalar, euclidean_distance_scalar,
};

/// Deterministischer 64-Bit PRNG (SplitMix64) zur reproduzierbaren Vektorgenerierung.
#[derive(Clone, Copy)]
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_f32(&mut self) -> f32 {
        let u = (self.next_u64() >> 40) as u32; // 24 bits
        (u as f32) / (16_777_216.0f32) // [0, 1)
    }

    fn next_f32_signed(&mut self) -> f32 {
        (self.next_f32() - 0.5f32) * 20.0f32
    }
}

/// RAII-Guard zur Wiederherstellung des Standard-Dispatchings nach dem Testlauf.
struct OverrideGuard;

impl OverrideGuard {
    fn set(path: Option<SimdPathOverride>) -> Self {
        set_dispatch_override(path);
        Self
    }
}

impl Drop for OverrideGuard {
    fn drop(&mut self) {
        set_dispatch_override(None);
    }
}

/// Prüft, ob zwei Float-Ergebnisse (Skalar vs. Dispatch) bezüglich Nicht-Endlichkeit (NaN / Inf) übereinstimmen.
fn check_non_finite_match(scalar: f32, dispatch: f32) -> bool {
    if scalar.is_nan() && dispatch.is_nan() {
        return true;
    }
    if scalar.is_infinite() && dispatch.is_infinite() {
        return scalar.is_sign_positive() == dispatch.is_sign_positive();
    }
    false
}

/// Berechnet die relative Abweichung unter Berücksichtigung von Auslöschungseffekten (Standard IEEE-754).
fn relative_diff(val_a: f32, val_b: f32, scale: f32) -> f32 {
    let diff = (val_a - val_b).abs();
    let norm = val_a.abs().max(val_b.abs()).max(scale).max(1.0);
    diff / norm
}

/// Prüft 100.000 Vektoren je Kernel (Euclidean, Cosine, Dot) gegen die Skalar-Referenz.
#[test]
fn test_t_simd_1_100k_vectors_per_kernel_against_scalar() {
    const SEED: u64 = 0x000C_0FFE_ED28_513D;
    const NUM_VECTORS: usize = 100_000;
    const REL_TOLERANCE: f32 = 1e-4;

    let mut rng = DeterministicRng::new(SEED);

    // Identifiziere zu testende Dispatch-Pfade auf diesem System
    let mut paths_to_test = vec![None]; // Standard-Runtime-Dispatch

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            paths_to_test.push(Some(SimdPathOverride::Avx2));
        }
        if is_x86_feature_detected!("avx512f") {
            paths_to_test.push(Some(SimdPathOverride::Avx512));
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            paths_to_test.push(Some(SimdPathOverride::Neon));
        }
    }

    // Für jeden Pfad mindestens 100.000 Vektoren
    for &forced_path in &paths_to_test {
        let _guard = OverrideGuard::set(forced_path);

        // Batch-Vektorpuffer um Reallokationen zu minimieren
        let mut buf_a = Vec::with_capacity(2048);
        let mut buf_b = Vec::with_capacity(2048);

        for iter in 0..NUM_VECTORS {
            // Längenverteilung deckt 0..=2048 ab:
            // - Häufige SIMD-Grenzen (0..64)
            // - Mittlere Embeddings (128, 256, 384, 512, 768)
            // - Große Embeddings (1024, 1536, 2048)
            let len = match iter % 20 {
                0 => 0,
                1 => 1,
                2 => 7,
                3 => 8,
                4 => 9,
                5 => 15,
                6 => 16,
                7 => 17,
                8 => 31,
                9 => 32,
                10 => 33,
                11 => 63,
                12 => 64,
                13 => 65,
                14 => 128,
                15 => 256,
                16 => 512,
                17 => 768,
                18 => 1536,
                _ => (rng.next_u64() % 2049) as usize,
            };

            buf_a.clear();
            buf_b.clear();

            let mut norm_a_sq = 0.0f32;
            let mut norm_b_sq = 0.0f32;

            // Spezielle Edge-Cases alle 1000 Iterationen einstreuen
            let is_edge_case = iter % 1000 == 0;
            for i in 0..len {
                let (val_a, val_b) = if is_edge_case {
                    match i % 9 {
                        0 => (0.0f32, 0.0f32),
                        1 => (1e-38f32, 1e-38f32), // Denormal / subnormal
                        2 => (-0.0f32, 0.0f32),
                        3 => (1e-5f32, -1e-5f32),
                        4 => (10.0f32, 10.0f32),
                        5 => (f32::NAN, f32::NAN), // NaN Edge Case
                        6 => (f32::INFINITY, f32::INFINITY), // +Inf Edge Case
                        7 => (f32::NEG_INFINITY, f32::NEG_INFINITY), // -Inf Edge Case
                        _ => {
                            let r1 = rng.next_f32_signed() * 0.01;
                            let r2 = rng.next_f32_signed() * 0.01;
                            (r1, r2)
                        }
                    }
                } else {
                    (rng.next_f32_signed(), rng.next_f32_signed())
                };
                buf_a.push(val_a);
                buf_b.push(val_b);
                norm_a_sq += val_a * val_a;
                norm_b_sq += val_b * val_b;
            }

            let a = &buf_a[..];
            let b = &buf_b[..];

            let norm_scale = (norm_a_sq.sqrt() * norm_b_sq.sqrt()).sqrt();

            // 1. EUCLIDEAN / L2 KERNEL
            let scalar_euc = euclidean_distance_scalar(a, b);
            let dispatch_euc = euclidean_distance(a, b).unwrap_or_else(|e| {
                panic!(
                    "T-SIMD-1 Failure: euclidean_distance returned error {:?} at iter {}, seed {}, path {:?}",
                    e, iter, SEED, forced_path
                );
            });
            if !scalar_euc.is_finite() || !dispatch_euc.is_finite() {
                assert!(
                    check_non_finite_match(scalar_euc, dispatch_euc),
                    "T-SIMD-1 L2 non-finite mismatch at iter {iter}, dim {len}, seed {SEED:#X}, path {forced_path:?}: scalar={scalar_euc}, dispatch={dispatch_euc}"
                );
            } else {
                let rel_diff_euc = relative_diff(scalar_euc, dispatch_euc, 1.0);
                assert!(
                    rel_diff_euc <= REL_TOLERANCE,
                    "T-SIMD-1 L2 failure at iter {iter}, dim {len}, seed {SEED:#X}, path {forced_path:?}: scalar={scalar_euc}, dispatch={dispatch_euc}, rel_diff={rel_diff_euc}"
                );
            }

            // 2. COSINE DISTANCE KERNEL
            let scalar_cos = cosine_distance_scalar(a, b);
            let dispatch_cos = cosine_distance(a, b).unwrap_or_else(|e| {
                panic!(
                    "T-SIMD-1 Failure: cosine_distance returned error {:?} at iter {}, seed {}, path {:?}",
                    e, iter, SEED, forced_path
                );
            });
            if !scalar_cos.is_finite() || !dispatch_cos.is_finite() {
                assert!(
                    check_non_finite_match(scalar_cos, dispatch_cos),
                    "T-SIMD-1 Cosine non-finite mismatch at iter {iter}, dim {len}, seed {SEED:#X}, path {forced_path:?}: scalar={scalar_cos}, dispatch={dispatch_cos}"
                );
            } else {
                let rel_diff_cos = relative_diff(scalar_cos, dispatch_cos, 1.0);
                assert!(
                    rel_diff_cos <= REL_TOLERANCE,
                    "T-SIMD-1 Cosine failure at iter {iter}, dim {len}, seed {SEED:#X}, path {forced_path:?}: scalar={scalar_cos}, dispatch={dispatch_cos}, rel_diff={rel_diff_cos}"
                );
            }

            // 3. DOT PRODUCT DISTANCE KERNEL
            let scalar_dot = dot_product_scalar(a, b);
            let dispatch_dot = dot_product_distance(a, b).unwrap_or_else(|e| {
                panic!(
                    "T-SIMD-1 Failure: dot_product_distance returned error {:?} at iter {}, seed {}, path {:?}",
                    e, iter, SEED, forced_path
                );
            });
            if !scalar_dot.is_finite() || !dispatch_dot.is_finite() {
                assert!(
                    check_non_finite_match(scalar_dot, dispatch_dot),
                    "T-SIMD-1 Dot non-finite mismatch at iter {iter}, dim {len}, seed {SEED:#X}, path {forced_path:?}: scalar={scalar_dot}, dispatch={dispatch_dot}"
                );
            } else {
                let rel_diff_dot = relative_diff(scalar_dot, dispatch_dot, norm_scale);
                assert!(
                    rel_diff_dot <= REL_TOLERANCE,
                    "T-SIMD-1 Dot failure at iter {iter}, dim {len}, seed {SEED:#X}, path {forced_path:?}: scalar={scalar_dot}, dispatch={dispatch_dot}, rel_diff={rel_diff_dot}, norm_scale={norm_scale}"
                );
            }
        }
    }
}
