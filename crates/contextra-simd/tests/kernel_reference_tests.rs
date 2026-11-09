// FILE-CONTEXT
// ZWECK: Skalar-Referenztests fuer alle Kernel, ISA-Pfade, Kantenlaengen und Unaligned-Slices.

//! # Kernel-Referenztests: `kernel_reference_tests`
//!
//! **Zweck (WP-90a):**
//! Vollstaendige Verifikation aller Distanz-Kernel (L2 / Euclidean, Cosine, Dot Product)
//! und ISA-Pfade (Skalar, AVX2, AVX-512, NEON) gegen die skalare Referenz (`kernels/scalar.rs`).
//!
//! **Testmatrix & Anforderungen:**
//! - Explizite Kantenlaengen: 0, 1, 7, 8, 15, 16, 31, 33 (sowie beliebige Dimensionen).
//! - Unaligned Slices (Offset 1..7 fuer Float- bzw. Byte-Slices).
//! - Eigenschaftsbasierte Tests (`proptest`) mit festen Seeds (`TestRunner` mit deterministic RNG).
//! - Typen: `f32`, `f32_bytes`, `u8`.
//! - Alle ISA-Varianten (Skalar, AVX2, AVX-512, NEON, Auto) via `Dispatcher::new(SimdTarget)`.
//!
//! **Fließkomma-Toleranzen & Summationsreihenfolge:**
//! - Skalar berechnet Summen sequenziell: `(a[0]*b[0] + a[1]*b[1]) + ...` mit sequentiellen Rundungsfehlern.
//! - AVX2 / AVX-512 verwenden Vektor-Akkumulation und FMA (`fmadd`), wodurch hoehere Zwischenpraezision
//!   und eine andere paarweise Baum-Summationsreihenfolge entsteht.
//! - Festgelegte Toleranzen:
//!   - f32 Cosine: relative Toleranz 1e-4 bzw. max(1.0, |scalar|) * 1e-4.
//!   - f32 Euclidean: relative Toleranz 1e-3 bzw. max(1.0, |scalar|) * 1e-3.
//!   - f32 Dot Product: relative Toleranz 1e-3 bzw. max(1.0, |scalar|) * 1e-3.
//!   - u8 Integer-Metriken (Dot, Euclidean Sq, Cosine Parts): Exakt 0 Toleranz (Bitgenaue Gleichheit).

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use contextra_simd::dispatch::{Dispatcher, SimdTarget};
use contextra_simd::kernels::scalar::{
    cosine_distance_f32_bytes_scalar, cosine_distance_scalar, cosine_similarity_parts_u8_scalar,
    dot_product_f32_bytes_scalar, dot_product_scalar, dot_product_u8_scalar,
    euclidean_distance_f32_bytes_scalar, euclidean_distance_scalar,
    euclidean_distance_sq_u8_scalar,
};
use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, TestRng, TestRunner};

const EXPLICIT_LENGTHS: [usize; 8] = [0, 1, 7, 8, 15, 16, 31, 33];

/// Helfer zur Ausfuehrung von Tests ueber alle verfuegbaren bzw. unterstuetzten ISA-Pfade.
fn run_across_isa_targets<F>(mut f: F)
where
    F: FnMut(Dispatcher),
{
    let targets = [
        SimdTarget::Scalar,
        SimdTarget::Auto,
        SimdTarget::Avx2,
        SimdTarget::Avx512,
        SimdTarget::Neon,
    ];

    for target in targets {
        let dispatcher = Dispatcher::new(target);
        f(dispatcher);
    }
}

#[test]
// ANCHOR[contextra-simd:TEST-REF-EXPLICIT-LENGTHS-F32]
// Prueft L2, Cosine und Dot Product fuer explizite Kantenlaengen 0, 1, 7, 8, 15, 16, 31, 33 gegen Skalar-Referenz.
fn test_explicit_lengths_f32_all_kernels() {
    run_across_isa_targets(|dispatcher| {
        for &len in &EXPLICIT_LENGTHS {
            let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.15 + 0.1).collect();
            let b: Vec<f32> = (0..len).map(|i| ((i + 1) as f32) * 0.12 + 0.2).collect();

            let euc_res = dispatcher.euclidean_distance(&a, &b);
            let cos_res = dispatcher.cosine_distance(&a, &b);
            let dot_res = dispatcher.dot_product_distance(&a, &b);

            // Falls dieses ISA-Target auf der aktuellen Host-CPU nicht unterstuetzt wird, ueberspringen
            if euc_res.is_err() {
                continue;
            }

            if len == 0 {
                assert_eq!(euc_res.unwrap(), 0.0);
                assert_eq!(cos_res.unwrap(), 1.0);
                assert_eq!(dot_res.unwrap(), 0.0);
                continue;
            }

            let euc_d = euc_res.unwrap();
            let euc_s = euclidean_distance_scalar(&a, &b);
            let euc_tol = 1e-3 * euc_s.abs().max(euc_d.abs()).max(1.0);
            assert!(
                (euc_d - euc_s).abs() <= euc_tol,
                "Euclidean mismatch at len {len}: dispatch={euc_d}, scalar={euc_s}, tol={euc_tol}"
            );

            let cos_d = cos_res.unwrap();
            let cos_s = cosine_distance_scalar(&a, &b);
            let cos_tol = 1e-4 * cos_s.abs().max(cos_d.abs()).max(1.0);
            assert!(
                (cos_d - cos_s).abs() <= cos_tol,
                "Cosine mismatch at len {len}: dispatch={cos_d}, scalar={cos_s}, tol={cos_tol}"
            );

            let dot_d = dot_res.unwrap();
            let dot_s = dot_product_scalar(&a, &b);
            let dot_tol = 1e-3 * dot_s.abs().max(dot_d.abs()).max(1.0);
            assert!(
                (dot_d - dot_s).abs() <= dot_tol,
                "Dot product mismatch at len {len}: dispatch={dot_d}, scalar={dot_s}, tol={dot_tol}"
            );
        }
    });
}

#[test]
// ANCHOR[contextra-simd:TEST-REF-EXPLICIT-LENGTHS-F32-BYTES]
// Prueft f32_bytes-Kernel fuer explizite Kantenlaengen 0, 1, 7, 8, 15, 16, 31, 33 gegen Skalar-Referenz.
fn test_explicit_lengths_f32_bytes_all_kernels() {
    run_across_isa_targets(|dispatcher| {
        for &len in &EXPLICIT_LENGTHS {
            let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.15 + 0.1).collect();
            let b: Vec<f32> = (0..len).map(|i| ((i + 1) as f32) * 0.12 + 0.2).collect();
            let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

            let euc_res = dispatcher.euclidean_distance_f32_bytes(&a, &b_bytes);
            let cos_res = dispatcher.cosine_distance_f32_bytes(&a, &b_bytes);
            let dot_res = dispatcher.dot_product_distance_f32_bytes(&a, &b_bytes);

            if euc_res.is_err() {
                continue;
            }

            if len == 0 {
                assert_eq!(euc_res.unwrap(), 0.0);
                assert_eq!(cos_res.unwrap(), 1.0);
                assert_eq!(dot_res.unwrap(), 0.0);
                continue;
            }

            let euc_d = euc_res.unwrap();
            let euc_s = euclidean_distance_f32_bytes_scalar(&a, &b_bytes);
            let euc_tol = 1e-3 * euc_s.abs().max(euc_d.abs()).max(1.0);
            assert!((euc_d - euc_s).abs() <= euc_tol);

            let cos_d = cos_res.unwrap();
            let cos_s = cosine_distance_f32_bytes_scalar(&a, &b_bytes);
            let cos_tol = 1e-4 * cos_s.abs().max(cos_d.abs()).max(1.0);
            assert!((cos_d - cos_s).abs() <= cos_tol);

            let dot_d = dot_res.unwrap();
            let dot_s = dot_product_f32_bytes_scalar(&a, &b_bytes);
            let dot_tol = 1e-3 * dot_s.abs().max(dot_d.abs()).max(1.0);
            assert!((dot_d - dot_s).abs() <= dot_tol);
        }
    });
}

#[test]
// ANCHOR[contextra-simd:TEST-REF-EXPLICIT-LENGTHS-U8]
// Prueft u8-Kernel fuer explizite Kantenlaengen 0, 1, 7, 8, 15, 16, 31, 33 auf bitgenaue Gleichheit.
fn test_explicit_lengths_u8_all_kernels() {
    run_across_isa_targets(|dispatcher| {
        for &len in &EXPLICIT_LENGTHS {
            let a: Vec<u8> = (0..len).map(|i| ((i * 17 + 5) % 256) as u8).collect();
            let b: Vec<u8> = (0..len).map(|i| ((i * 23 + 11) % 256) as u8).collect();

            let dot_res = dispatcher.dot_product_u8(&a, &b);
            if dot_res.is_err() {
                continue;
            }
            let dot_d = dot_res.unwrap();
            let dot_s = dot_product_u8_scalar(&a, &b);
            assert_eq!(dot_d, dot_s, "u8 dot mismatch at len {len}");

            let euc_d = dispatcher.euclidean_distance_sq_u8(&a, &b).unwrap();
            let euc_s = euclidean_distance_sq_u8_scalar(&a, &b);
            assert_eq!(euc_d, euc_s, "u8 euclidean sq mismatch at len {len}");

            let cos_d = dispatcher.cosine_similarity_parts_u8(&a, &b).unwrap();
            let cos_s = cosine_similarity_parts_u8_scalar(&a, &b);
            assert_eq!(cos_d, cos_s, "u8 cosine parts mismatch at len {len}");
        }
    });
}

#[test]
// ANCHOR[contextra-simd:TEST-REF-UNALIGNED-SLICES]
// Prueft unaligned Slices mit wechselnden Byte- und Element-Offsets (1..7) fuer alle Kantenlaengen.
fn test_unaligned_slices_all_lengths() {
    run_across_isa_targets(|dispatcher| {
        // Puffer mit Padding anlegen, um beliebige Offsets zu schneiden
        let buf_a: Vec<f32> = (0..200).map(|i| (i as f32) * 0.1 - 5.0).collect();
        let buf_b: Vec<f32> = (0..200).map(|i| ((i + 3) as f32) * 0.07 - 4.0).collect();
        let buf_b_bytes: Vec<u8> = buf_b.iter().flat_map(|x| x.to_le_bytes()).collect();

        let u8_buf_a: Vec<u8> = (0..200).map(|i| ((i * 13) % 256) as u8).collect();
        let u8_buf_b: Vec<u8> = (0..200).map(|i| ((i * 29) % 256) as u8).collect();

        for offset in 1..=7 {
            for &len in &EXPLICIT_LENGTHS {
                if len == 0 {
                    continue;
                }
                let a = &buf_a[offset..offset + len];
                let b = &buf_b[offset + 1..offset + 1 + len];
                let b_bytes = &buf_b_bytes[(offset + 1) * 4..(offset + 1 + len) * 4];

                let euc_res = dispatcher.euclidean_distance(a, b);
                if euc_res.is_err() {
                    continue;
                }

                // f32 L2
                let euc_d = euc_res.unwrap();
                let euc_s = euclidean_distance_scalar(a, b);
                assert!((euc_d - euc_s).abs() <= 1e-3 * euc_s.abs().max(euc_d.abs()).max(1.0));

                // f32 Cosine
                let cos_d = dispatcher.cosine_distance(a, b).unwrap();
                let cos_s = cosine_distance_scalar(a, b);
                assert!((cos_d - cos_s).abs() <= 1e-4 * cos_s.abs().max(cos_d.abs()).max(1.0));

                // f32 Dot
                let dot_d = dispatcher.dot_product_distance(a, b).unwrap();
                let dot_s = dot_product_scalar(a, b);
                assert!((dot_d - dot_s).abs() <= 1e-3 * dot_s.abs().max(dot_d.abs()).max(1.0));

                // f32_bytes
                let euc_b_d = dispatcher.euclidean_distance_f32_bytes(a, b_bytes).unwrap();
                let euc_b_s = euclidean_distance_f32_bytes_scalar(a, b_bytes);
                assert!((euc_b_d - euc_b_s).abs() <= 1e-3 * euc_b_s.abs().max(euc_b_d.abs()).max(1.0));

                // u8
                let u_a = &u8_buf_a[offset..offset + len];
                let u_b = &u8_buf_b[offset + 2..offset + 2 + len];
                assert_eq!(dispatcher.dot_product_u8(u_a, u_b).unwrap(), dot_product_u8_scalar(u_a, u_b));
                assert_eq!(dispatcher.euclidean_distance_sq_u8(u_a, u_b).unwrap(), euclidean_distance_sq_u8_scalar(u_a, u_b));
                assert_eq!(dispatcher.cosine_similarity_parts_u8(u_a, u_b).unwrap(), cosine_similarity_parts_u8_scalar(u_a, u_b));
            }
        }
    });
}

#[test]
// ANCHOR[contextra-simd:TEST-PROPTEST-FIXED-SEED-F32]
// Proptest mit festem deterministischem Seed fuer Zufallseingaben ueber variable Laengen.
fn test_proptest_fixed_seed_f32() {
    let fixed_seeds = [
        [0x42; 32],
        [0x99; 32],
        [0xAA; 32],
    ];

    for seed in fixed_seeds {
        let config = Config {
            cases: 100,
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..Config::default()
        };
        let mut runner = TestRunner::new_with_rng(config, TestRng::from_seed(proptest::test_runner::RngAlgorithm::ChaCha, &seed));

        let strategy = (0..=65usize).prop_flat_map(|len| {
            (
                prop::collection::vec(-10.0f32..10.0f32, len),
                prop::collection::vec(-10.0f32..10.0f32, len),
            )
        });

        runner
            .run(&strategy, |(a, b)| {
                let len = a.len();
                let dispatcher = Dispatcher::auto();
                let euc_d = dispatcher.euclidean_distance(&a, &b).unwrap();
                let cos_d = dispatcher.cosine_distance(&a, &b).unwrap();
                let dot_d = dispatcher.dot_product_distance(&a, &b).unwrap();

                if len == 0 {
                    prop_assert_eq!(euc_d, 0.0);
                    prop_assert_eq!(cos_d, 1.0);
                    prop_assert_eq!(dot_d, 0.0);
                    return Ok(());
                }

                let euc_s = euclidean_distance_scalar(&a, &b);
                let euc_tol = 1e-3 * euc_s.abs().max(euc_d.abs()).max(1.0);
                prop_assert!((euc_d - euc_s).abs() <= euc_tol);

                let cos_s = cosine_distance_scalar(&a, &b);
                let cos_tol = 1e-4 * cos_s.abs().max(cos_d.abs()).max(1.0);
                prop_assert!((cos_d - cos_s).abs() <= cos_tol);

                let dot_s = dot_product_scalar(&a, &b);
                let dot_tol = 1e-3 * dot_s.abs().max(dot_d.abs()).max(1.0);
                prop_assert!((dot_d - dot_s).abs() <= dot_tol);

                Ok(())
            })
            .unwrap();
    }
}

#[test]
// ANCHOR[contextra-simd:TEST-PROPTEST-FIXED-SEED-U8]
// Proptest mit festem deterministischem Seed fuer u8-Zufallseingaben.
fn test_proptest_fixed_seed_u8() {
    let fixed_seeds = [
        [0x17; 32],
        [0x55; 32],
        [0xFE; 32],
    ];

    for seed in fixed_seeds {
        let config = Config {
            cases: 100,
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..Config::default()
        };
        let mut runner = TestRunner::new_with_rng(config, TestRng::from_seed(proptest::test_runner::RngAlgorithm::ChaCha, &seed));

        let strategy = (0..=65usize).prop_flat_map(|len| {
            (
                prop::collection::vec(any::<u8>(), len),
                prop::collection::vec(any::<u8>(), len),
            )
        });

        runner
            .run(&strategy, |(a, b)| {
                let dispatcher = Dispatcher::auto();
                let dot_d = dispatcher.dot_product_u8(&a, &b).unwrap();
                let dot_s = dot_product_u8_scalar(&a, &b);
                prop_assert_eq!(dot_d, dot_s);

                let euc_d = dispatcher.euclidean_distance_sq_u8(&a, &b).unwrap();
                let euc_s = euclidean_distance_sq_u8_scalar(&a, &b);
                prop_assert_eq!(euc_d, euc_s);

                let cos_d = dispatcher.cosine_similarity_parts_u8(&a, &b).unwrap();
                let cos_s = cosine_similarity_parts_u8_scalar(&a, &b);
                prop_assert_eq!(cos_d, cos_s);

                Ok(())
            })
            .unwrap();
    }
}
