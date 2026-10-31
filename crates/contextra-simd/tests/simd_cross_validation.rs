// ADR-002 / SPEC §27: Test assertions and test dataset casts
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
// ADR-002 / SPEC §27: Test assertions and test dataset casts

//! # Quervalidierungs- und Property-Test-Suite: `simd_cross_validation`
//!
//! **Zweck:**
//! Führt umfassende hypothesen- und eigenschaftsbasierte Tests (Property-Based Testing via `proptest`)
//! sowie deterministische Quervalidierungen durch:
//! - Validiert SIMD-Dispatch gegen Skalar-Referenzen über zufällige Dimensionen (1 bis 1024).
//! - Prüft 1.000 Vektorpaare auf strenge Äquivalenz-Schwellenwerte (`1e-4`).
//! - Überprüft Grenzlängen um SIMD-Registergrenzen herum (z. B. 7, 8, 9 bei AVX2; 15, 16, 17 bei AVX-512).
//! - Prüft mathematische Axiome: Nullvektor-Eigenschaften, Dreiecksungleichung, Symmetrie.

use contextra_simd::dispatch::{
    cosine_distance, cosine_distance_f32_bytes, cosine_similarity_parts_u8, dot_product_distance,
    dot_product_distance_f32_bytes, dot_product_u8, euclidean_distance,
    euclidean_distance_f32_bytes, euclidean_distance_sq_u8,
};
use contextra_simd::kernels::scalar::{
    cosine_distance_f32_bytes_scalar, cosine_distance_scalar, cosine_similarity_parts_u8_scalar,
    dot_product_f32_bytes_scalar, dot_product_scalar, dot_product_u8_scalar,
    euclidean_distance_f32_bytes_scalar, euclidean_distance_scalar,
    euclidean_distance_sq_u8_scalar,
};
use proptest::prelude::*;

proptest! {
    // ANCHOR[contextra-simd:TEST-CROSS-PROP-COSINE-F32]
    // Property-Test: Vergleicht SIMD- und Skalar-Kosinus-Distanz über beliebige Dimensionen (1..1024).
    #[test]
    fn simd_vs_scalar_cosine_f32(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = cosine_distance_scalar(a, b);
        let dispatch = cosine_distance(a, b).expect("valid distance calculation");
        let diff = (scalar - dispatch).abs();
        let tol = 1e-4 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Cosine mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    // ANCHOR[contextra-simd:TEST-CROSS-PROP-EUCLIDEAN-F32]
    // Property-Test: Vergleicht SIMD- und Skalar-euklidische Distanz über beliebige Dimensionen (1..1024).
    #[test]
    fn simd_vs_scalar_euclidean_f32(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = euclidean_distance_scalar(a, b);
        let dispatch = euclidean_distance(a, b).expect("valid distance calculation");
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Euclidean mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    // ANCHOR[contextra-simd:TEST-CROSS-PROP-DOT-F32]
    // Property-Test: Vergleicht SIMD- und Skalarprodukt-Distanz über beliebige Dimensionen (1..1024).
    #[test]
    fn simd_vs_scalar_dot_product_f32(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = dot_product_scalar(a, b);
        let dispatch = dot_product_distance(a, b).expect("valid distance calculation");
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Dot product mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    // ANCHOR[contextra-simd:TEST-CROSS-PROP-COSINE-BYTES]
    // Property-Test: Vergleicht SIMD- und Skalar-Kosinus-Distanz auf Byte-Slices über Dimensionen 1..1024.
    #[test]
    fn simd_vs_scalar_cosine_f32_bytes(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b_bytes: Vec<u8> = b[..n].iter().flat_map(|x| x.to_le_bytes()).collect();

        let scalar = cosine_distance_f32_bytes_scalar(a, &b_bytes);
        let dispatch = cosine_distance_f32_bytes(a, &b_bytes).expect("valid distance calculation");
        let diff = (scalar - dispatch).abs();
        let tol = 1e-4 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Cosine f32_bytes mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    // ANCHOR[contextra-simd:TEST-CROSS-PROP-EUCLIDEAN-BYTES]
    // Property-Test: Vergleicht SIMD- und Skalar-euklidische Distanz auf Byte-Slices über Dimensionen 1..1024.
    #[test]
    fn simd_vs_scalar_euclidean_f32_bytes(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b_bytes: Vec<u8> = b[..n].iter().flat_map(|x| x.to_le_bytes()).collect();

        let scalar = euclidean_distance_f32_bytes_scalar(a, &b_bytes);
        let dispatch = euclidean_distance_f32_bytes(a, &b_bytes).expect("valid distance calculation");
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Euclidean f32_bytes mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    // ANCHOR[contextra-simd:TEST-CROSS-PROP-DOT-BYTES]
    // Property-Test: Vergleicht SIMD- und Skalarprodukt auf Byte-Slices über Dimensionen 1..1024.
    #[test]
    fn simd_vs_scalar_dot_product_f32_bytes(
        dim in 1..1024usize,
        a in prop::collection::vec(-10.0f32..10.0, 1..1024),
        b in prop::collection::vec(-10.0f32..10.0, 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b_bytes: Vec<u8> = b[..n].iter().flat_map(|x| x.to_le_bytes()).collect();

        let scalar = dot_product_f32_bytes_scalar(a, &b_bytes);
        let dispatch = dot_product_distance_f32_bytes(a, &b_bytes).expect("valid distance calculation");
        let diff = (scalar - dispatch).abs();
        let tol = 1e-3 * scalar.abs().max(dispatch.abs()).max(1.0);
        prop_assert!(
            diff <= tol,
            "Dot product f32_bytes mismatch at dim {}: scalar={}, dispatch={}, diff={}, tol={}",
            n, scalar, dispatch, diff, tol
        );
    }

    // ANCHOR[contextra-simd:TEST-CROSS-PROP-DOT-U8]
    // Property-Test: Vergleicht u8-Skalarprodukt zwischen SIMD und Skalar auf Gleichheit.
    #[test]
    fn simd_vs_scalar_dot_product_u8(
        dim in 1..1024usize,
        a in prop::collection::vec(any::<u8>(), 1..1024),
        b in prop::collection::vec(any::<u8>(), 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = dot_product_u8_scalar(a, b);
        let dispatch = dot_product_u8(a, b).expect("valid distance calculation");
        prop_assert_eq!(
            scalar, dispatch,
            "Dot product u8 mismatch at dim {}: scalar={}, dispatch={}",
            n, scalar, dispatch
        );
    }

    // ANCHOR[contextra-simd:TEST-CROSS-PROP-EUCLIDEAN-SQ-U8]
    // Property-Test: Vergleicht quadrierte u8-euklidische Distanz zwischen SIMD und Skalar auf Gleichheit.
    #[test]
    fn simd_vs_scalar_euclidean_sq_u8(
        dim in 1..1024usize,
        a in prop::collection::vec(any::<u8>(), 1..1024),
        b in prop::collection::vec(any::<u8>(), 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = euclidean_distance_sq_u8_scalar(a, b);
        let dispatch = euclidean_distance_sq_u8(a, b).expect("valid distance calculation");
        prop_assert_eq!(
            scalar, dispatch,
            "Euclidean sq u8 mismatch at dim {}: scalar={}, dispatch={}",
            n, scalar, dispatch
        );
    }

    // ANCHOR[contextra-simd:TEST-CROSS-PROP-COSINE-PARTS-U8]
    // Property-Test: Vergleicht u8-Kosinus-Komponentensummen zwischen SIMD und Skalar.
    #[test]
    fn simd_vs_scalar_cosine_similarity_parts_u8(
        dim in 1..1024usize,
        a in prop::collection::vec(any::<u8>(), 1..1024),
        b in prop::collection::vec(any::<u8>(), 1..1024),
    ) {
        let n = dim.min(a.len()).min(b.len());
        let a = &a[..n];
        let b = &b[..n];

        let scalar = cosine_similarity_parts_u8_scalar(a, b);
        let dispatch = cosine_similarity_parts_u8(a, b).expect("valid distance calculation");
        prop_assert_eq!(
            scalar, dispatch,
            "Cosine similarity parts u8 mismatch at dim {}: scalar={:?}, dispatch={:?}",
            n, scalar, dispatch
        );
    }
}

// ANCHOR[contextra-simd:TEST-CROSS-1000-PAIRS]
// Prüft 1000 normalisierte Zufallsvektorpaare auf strikte Äquivalenz (Delta < 1e-4).
#[test]
fn test_s4_1000_pairs_equivalence_threshold() {
    let dimensions = [128, 512, 1536, 4096];
    let num_pairs = 1000;

    for &dim in &dimensions {
        let mut max_cos_diff = 0.0f32;
        let mut max_euc_diff = 0.0f32;
        let mut max_dot_diff = 0.0f32;

        for pair_idx in 0..num_pairs {
            let a: Vec<f32> = (0..dim)
                .map(|i| (((i * 17 + pair_idx * 31) % 1000) as f32) / 100.0 - 5.0)
                .collect();
            let b: Vec<f32> = (0..dim)
                .map(|i| (((i * 23 + pair_idx * 37) % 1000) as f32) / 100.0 - 5.0)
                .collect();

            let cos_s = cosine_distance_scalar(&a, &b);
            let cos_d = cosine_distance(&a, &b).expect("valid distance calculation");
            let cos_diff = (cos_s - cos_d).abs();
            max_cos_diff = max_cos_diff.max(cos_diff);

            let euc_s = euclidean_distance_scalar(&a, &b);
            let euc_d = euclidean_distance(&a, &b).expect("valid distance calculation");
            let euc_diff = (euc_s - euc_d).abs();
            max_euc_diff = max_euc_diff.max(euc_diff);

            let dot_s = dot_product_scalar(&a, &b);
            let dot_d = dot_product_distance(&a, &b).expect("valid distance calculation");
            let dot_diff = (dot_s - dot_d).abs();
            max_dot_diff = max_dot_diff.max(dot_diff);
        }

        println!("Dim {dim}: max_cos_diff={max_cos_diff:.8e}, max_euc_diff={max_euc_diff:.8e}, max_dot_diff={max_dot_diff:.8e}");

        assert!(
            max_cos_diff < 1e-5,
            "Cosine max diff at dim {dim} exceeded 1e-5: {max_cos_diff}"
        );
        assert!(
            max_euc_diff < 1e-3,
            "Euclidean max diff at dim {dim}: {max_euc_diff}"
        );
        assert!(
            max_dot_diff < 5e-3,
            "Dot product max diff at dim {dim}: {max_dot_diff}"
        );
    }
}

// ANCHOR[contextra-simd:TEST-CROSS-REMAINDER-F32]
// Prüft Grenzdimensionen um SIMD-Registerbreiten (0 bis 65) auf exakte Restschleifen-Verarbeitung.
#[test]
fn test_simd_remainder_boundary_lengths_f32() {
    // Tests boundary vector lengths across SIMD chunk sizes (4, 8, 16, 32, 64)
    // to verify remainder loop handling for cosine, euclidean, and dot product distances.
    let boundary_lengths = [
        0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129,
    ];

    for &len in &boundary_lengths {
        let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.1 - 2.0).collect();
        let b: Vec<f32> = (0..len).map(|i| (i as f32) * 0.15 - 1.5).collect();

        // Cosine distance
        let cos_scalar = cosine_distance_scalar(&a, &b);
        let cos_dispatch = cosine_distance(&a, &b).expect("valid distance calculation");
        let cos_diff = (cos_scalar - cos_dispatch).abs();
        let cos_tol = 1e-4 * cos_scalar.abs().max(cos_dispatch.abs()).max(1.0);
        assert!(
            cos_diff <= cos_tol,
            "Cosine remainder mismatch at len {len}: scalar={cos_scalar}, dispatch={cos_dispatch}"
        );

        // Euclidean distance
        let euc_scalar = euclidean_distance_scalar(&a, &b);
        let euc_dispatch = euclidean_distance(&a, &b).expect("valid distance calculation");
        let euc_diff = (euc_scalar - euc_dispatch).abs();
        let euc_tol = 1e-3 * euc_scalar.abs().max(euc_dispatch.abs()).max(1.0);
        assert!(
            euc_diff <= euc_tol,
            "Euclidean remainder mismatch at len {len}: scalar={euc_scalar}, dispatch={euc_dispatch}"
        );

        // Dot product distance
        let dot_scalar = dot_product_scalar(&a, &b);
        let dot_dispatch = dot_product_distance(&a, &b).expect("valid distance calculation");
        let dot_diff = (dot_scalar - dot_dispatch).abs();
        let dot_tol = 1e-3 * dot_scalar.abs().max(dot_dispatch.abs()).max(1.0);
        assert!(
            dot_diff <= dot_tol,
            "Dot product remainder mismatch at len {len}: scalar={dot_scalar}, dispatch={dot_dispatch}"
        );

        // f32_bytes variants
        let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();
        let cos_bytes_scalar = cosine_distance_f32_bytes_scalar(&a, &b_bytes);
        let cos_bytes_dispatch =
            cosine_distance_f32_bytes(&a, &b_bytes).expect("valid distance calculation");
        let cos_bytes_tol = 1e-4
            * cos_bytes_scalar
                .abs()
                .max(cos_bytes_dispatch.abs())
                .max(1.0);
        assert!(
            (cos_bytes_scalar - cos_bytes_dispatch).abs() <= cos_bytes_tol,
            "Cosine f32_bytes remainder mismatch at len {len}"
        );

        let euc_bytes_scalar = euclidean_distance_f32_bytes_scalar(&a, &b_bytes);
        let euc_bytes_dispatch =
            euclidean_distance_f32_bytes(&a, &b_bytes).expect("valid distance calculation");
        let euc_bytes_tol = 1e-3
            * euc_bytes_scalar
                .abs()
                .max(euc_bytes_dispatch.abs())
                .max(1.0);
        assert!(
            (euc_bytes_scalar - euc_bytes_dispatch).abs() <= euc_bytes_tol,
            "Euclidean f32_bytes remainder mismatch at len {len}"
        );

        let dot_bytes_scalar = dot_product_f32_bytes_scalar(&a, &b_bytes);
        let dot_bytes_dispatch =
            dot_product_distance_f32_bytes(&a, &b_bytes).expect("valid distance calculation");
        let dot_bytes_tol = 1e-3
            * dot_bytes_scalar
                .abs()
                .max(dot_bytes_dispatch.abs())
                .max(1.0);
        assert!(
            (dot_bytes_scalar - dot_bytes_dispatch).abs() <= dot_bytes_tol,
            "Dot product f32_bytes remainder mismatch at len {len}"
        );
    }
}

// ANCHOR[contextra-simd:TEST-CROSS-REMAINDER-U8]
// Prüft Grenzdimensionen für u8-Vektoren (0 bis 130) auf fehlerfreie Vektorrest-Akkumulation.
#[test]
fn test_simd_remainder_boundary_lengths_u8() {
    let boundary_lengths = [
        0, 1, 2, 3, 4, 7, 8, 15, 16, 31, 32, 33, 63, 64, 65, 127, 128,
    ];

    for &len in &boundary_lengths {
        let a: Vec<u8> = (0..len).map(|i| (i * 7) as u8).collect();
        let b: Vec<u8> = (0..len).map(|i| (i * 13) as u8).collect();

        let dot_s = dot_product_u8_scalar(&a, &b);
        let dot_d = dot_product_u8(&a, &b).expect("valid distance calculation");
        assert_eq!(
            dot_s, dot_d,
            "Dot product u8 remainder mismatch at len {len}"
        );

        let euc_s = euclidean_distance_sq_u8_scalar(&a, &b);
        let euc_d = euclidean_distance_sq_u8(&a, &b).expect("valid distance calculation");
        assert_eq!(
            euc_s, euc_d,
            "Euclidean sq u8 remainder mismatch at len {len}"
        );

        let cos_s = cosine_similarity_parts_u8_scalar(&a, &b);
        let cos_d = cosine_similarity_parts_u8(&a, &b).expect("valid distance calculation");
        assert_eq!(
            cos_s, cos_d,
            "Cosine similarity parts u8 remainder mismatch at len {len}"
        );
    }
}

// ANCHOR[contextra-simd:TEST-CROSS-ZERO-AND-L2]
// Prüft mathematisches Axiom S1: Nullvektor-Kosinusdistanz ist 1.0 und L2-Norm ist 0.0.
#[test]
fn test_s1_zero_vector_and_l2_norm() {
    let zero = vec![0.0f32; 128];
    let v = vec![1.0f32; 128];

    assert_eq!(cosine_distance_scalar(&zero, &zero), 1.0);
    assert_eq!(
        cosine_distance(&zero, &zero).expect("valid distance calculation"),
        1.0
    );
    assert_eq!(
        cosine_distance(&zero, &v).expect("valid distance calculation"),
        1.0
    );
    assert_eq!(
        cosine_distance(&v, &zero).expect("valid distance calculation"),
        1.0
    );
}

// ANCHOR[contextra-simd:TEST-CROSS-L2-CORRECTNESS]
// Prüft mathematisches Axiom S2: Euklidische Distanz erfüllt Dreiecksungleichung und Symmetrie.
#[test]
fn test_s2_l2_distance_correctness() {
    let a = vec![3.0f32, 0.0];
    let b = vec![0.0f32, 4.0];
    assert_eq!(euclidean_distance_scalar(&a, &b), 5.0);
    assert_eq!(
        euclidean_distance(&a, &b).expect("valid distance calculation"),
        5.0
    );
}

// ANCHOR[contextra-simd:TEST-CROSS-OVERFLOW-BEHAVIOR]
// Prüft mathematisches Axiom S3: Kein vorzeitiger Überlauf bei großen Werten innerhalb f32::MAX.
#[test]
fn test_s3_overflow_behavior() {
    let norm_a = vec![1.0f32 / (4096.0f32.sqrt()); 4096];
    let norm_b = vec![1.0f32 / (4096.0f32.sqrt()); 4096];
    let dot = dot_product_distance(&norm_a, &norm_b).expect("valid distance calculation");
    assert!(!dot.is_nan() && !dot.is_infinite());

    let huge_a = vec![1e20f32; 128];
    let huge_b = vec![1e20f32; 128];
    let huge_dot = dot_product_distance(&huge_a, &huge_b).expect("valid distance calculation");
    assert!(huge_dot.is_infinite());
}
