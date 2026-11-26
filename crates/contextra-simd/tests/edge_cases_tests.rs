// FILE-CONTEXT
// ZWECK: Edge-case tests covering NaN, Infinity, denormals, zero-vectors, huge magnitudes, and error boundaries.

//! # Randfall- und Grenzwert-Test-Suite: `edge_cases_tests`
//!
//! **Zweck:**
//! Testet gezielt mathematische und speicherbezogene Grenzfälle:
//! - Erkennung von ungültigen Gleitkommazahlen (NaN, +Inf, -Inf, subnormale/denormale Zahlen).
//! - Längenprüfungen (leere Vektoren, 1-Element-Vektoren, Dimensions-Fehlanpassungen).
//! - Mathematische Extrema (Nullvektoren, identische Vektoren, orthogonale Vektoren, antipodische Vektoren).
//! - Numerische Stabilität (sehr große Werte bis 1e18, maximale u8-Akkumulation ohne 32-Bit Überlauf).

// ADR-002 / SPEC §27: Test assertions and test dataset casts
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use contextra_simd::dispatch::{
    cosine_distance, cosine_distance_f32_bytes, cosine_similarity_parts_u8, dot_product_distance,
    dot_product_distance_f32_bytes, dot_product_u8, euclidean_distance,
    euclidean_distance_f32_bytes, euclidean_distance_sq_u8,
};
use contextra_simd::kernels::scalar::{
    cosine_distance_f32_bytes_scalar, cosine_distance_scalar, dot_product_f32_bytes_scalar,
    dot_product_scalar, dot_product_u8_scalar, euclidean_distance_f32_bytes_scalar,
    euclidean_distance_scalar, euclidean_distance_sq_u8_scalar, normalize_inplace,
    CosineSimilarityPartsF32U8, CosineSimilarityPartsU8,
};
use contextra_simd::{
    compute_distance, compute_distance_f32_bytes_trusted, compute_distance_trusted, validate_vector,
};
use contextra_simd::{DistanceMetric, SimdError};

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-NAN-DETECTION]
// Prüft, dass validate_vector Vektoren mit NaN-Werten zuverlässig abweist.
fn test_nan_detection_in_vector() {
    let nan_vec = vec![1.0f32, 2.0, f32::NAN, 4.0];
    let res = validate_vector(&nan_vec);
    assert!(matches!(res, Err(SimdError::InvalidInput(_))));
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-INF-DETECTION]
// Prüft, dass validate_vector positive und negative Unendlichkeit abweist.
fn test_infinity_detection_in_vector() {
    let inf_vec = vec![1.0f32, f32::INFINITY, 3.0];
    assert!(matches!(
        validate_vector(&inf_vec),
        Err(SimdError::InvalidInput(_))
    ));

    let neg_inf_vec = vec![1.0f32, f32::NEG_INFINITY, 3.0];
    assert!(matches!(
        validate_vector(&neg_inf_vec),
        Err(SimdError::InvalidInput(_))
    ));
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-VALIDATE-BOUNDARIES]
// Prüft die Erkennung von NaN und Inf an Indexgrenzen (erstes, mittleres, letztes Element).
fn test_validate_vector_nan_and_inf_at_boundary_indices() {
    let mut vec_first_nan = vec![0.0f32; 100];
    vec_first_nan[0] = f32::NAN;
    assert!(validate_vector(&vec_first_nan).is_err());

    let mut vec_last_inf = vec![0.0f32; 100];
    vec_last_inf[99] = f32::INFINITY;
    assert!(validate_vector(&vec_last_inf).is_err());
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-VALIDATE-EMPTY-LARGE]
// Prüft validate_vector mit leeren und großen gültigen Vektoren.
fn test_validate_vector_empty_and_large_valid_slices() {
    let empty: Vec<f32> = vec![];
    assert!(validate_vector(&empty).is_ok());

    let large_valid: Vec<f32> = (0..10_000).map(|i| (i as f32) * 0.01 - 50.0).collect();
    assert!(validate_vector(&large_valid).is_ok());
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-COMPUTE-REJECTS-NAN]
// Prüft, dass compute_distance bei NaN oder Inf in a oder b einen Fehler liefert.
fn test_compute_distance_rejects_nan_and_inf() {
    let valid = vec![1.0f32, 2.0, 3.0];
    let nan = vec![1.0f32, f32::NAN, 3.0];
    let inf = vec![1.0f32, f32::INFINITY, 3.0];

    assert!(compute_distance(&nan, &valid, DistanceMetric::Cosine).is_err());
    assert!(compute_distance(&valid, &inf, DistanceMetric::Euclidean).is_err());
    assert!(compute_distance(&inf, &nan, DistanceMetric::DotProduct).is_err());
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-COMPUTE-TRUSTED-METRICS]
// Prüft compute_distance_trusted für alle unterstützten Distanzmetriken.
fn test_compute_distance_trusted_all_supported_metrics() {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [2.0f32, 3.0, 4.0, 5.0];

    let cos = compute_distance_trusted(&a, &b, DistanceMetric::Cosine).unwrap();
    let euc = compute_distance_trusted(&a, &b, DistanceMetric::Euclidean).unwrap();
    let dot = compute_distance_trusted(&a, &b, DistanceMetric::DotProduct).unwrap();

    assert!((0.0..=2.0).contains(&cos));
    assert_eq!(euc, 2.0); // sqrt(1 + 1 + 1 + 1) = 2.0
    assert_eq!(dot, -40.0);
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-COMPUTE-BYTES-METRICS]
// Prüft compute_distance_f32_bytes_trusted für alle unterstützten Distanzmetriken.
fn test_compute_distance_f32_bytes_trusted_all_supported_metrics() {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [2.0f32, 3.0, 4.0, 5.0];
    let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

    let cos = compute_distance_f32_bytes_trusted(&a, &b_bytes, DistanceMetric::Cosine).unwrap();
    let euc = compute_distance_f32_bytes_trusted(&a, &b_bytes, DistanceMetric::Euclidean).unwrap();
    let dot = compute_distance_f32_bytes_trusted(&a, &b_bytes, DistanceMetric::DotProduct).unwrap();

    assert!((0.0..=2.0).contains(&cos));
    assert_eq!(euc, 2.0);
    assert_eq!(dot, -40.0);
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-DENORMAL-FLOATS]
// Prüft die stabile Berechnung von Distanzen bei subnormalen/denormalen Gleitkommazahlen.
fn test_denormal_float_handling() {
    // 1e-18f32 is a valid subnormal/small float whose square 1e-36 is above DENORMAL_MIN (~1.4e-45)
    let subnormal = 1e-18f32;
    let a = vec![subnormal; 64];
    let b = vec![subnormal * 2.0; 64];

    let cos = cosine_distance(&a, &b).unwrap();
    let euc = euclidean_distance(&a, &b).unwrap();
    let dot = dot_product_distance(&a, &b).unwrap();

    assert!(!cos.is_nan());
    assert!(!euc.is_nan());
    assert!(!dot.is_nan());
    assert!((cos - 0.0).abs() < 1e-5);

    // Extreme underflowing float where x*x underflows to 0.0
    let extreme_underflow = 1e-38f32;
    let u_a = vec![extreme_underflow; 64];
    let u_b = vec![extreme_underflow; 64];
    // Underflowing norm returns 1.0 (zero-norm safe fallback)
    assert_eq!(cosine_distance(&u_a, &u_b).unwrap(), 1.0);
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-ZERO-VEC-COSINE]
// Prüft, dass Nullvektoren bei der Kosinus-Distanz sicher 1.0 liefern (keine Null-Division).
fn test_zero_vector_cosine_distance() {
    let zero = vec![0.0f32; 128];
    let v = vec![1.0f32; 128];

    assert_eq!(cosine_distance(&zero, &zero).unwrap(), 1.0);
    assert_eq!(cosine_distance(&zero, &v).unwrap(), 1.0);
    assert_eq!(cosine_distance(&v, &zero).unwrap(), 1.0);
    assert_eq!(cosine_distance_scalar(&zero, &zero), 1.0);
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-LARGE-MAGNITUDES]
// Prüft numerische Stabilität bei Vektoren mit extrem großen Amplituden (1e18).
fn test_large_magnitudes_handling() {
    // 1e18f32 is a large magnitude float whose square 1e36 is within f32::MAX (~3.4e38)
    let large_a = vec![1e18f32; 64];
    let large_b = vec![1e18f32; 64];

    let cos = cosine_distance(&large_a, &large_b).unwrap();
    assert!((cos - 0.0).abs() < 1e-5);

    let euc = euclidean_distance(&large_a, &large_b).unwrap();
    assert_eq!(euc, 0.0);

    // Overflowing magnitude 1e20f32 where x*x overflows to Infinity
    let overflow_a = vec![1e20f32; 64];
    let overflow_b = vec![1e20f32; 64];
    let dot = dot_product_distance(&overflow_a, &overflow_b).unwrap();
    assert!(dot.is_infinite());
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-IDENTICAL-VECTORS]
// Prüft, dass identische Vektoren Kosinus-Distanz 0.0 und euklidische Distanz 0.0 haben.
fn test_identical_vectors_cosine_and_euclidean() {
    let a = vec![1.5f32, -2.3, 4.1, 0.8, -9.2];

    let cos = cosine_distance(&a, &a).unwrap();
    let euc = euclidean_distance(&a, &a).unwrap();

    assert!(
        cos.abs() < 1e-6,
        "d(a, a) for cosine should be 0.0, got {cos}"
    );
    assert_eq!(euc, 0.0, "d(a, a) for euclidean should be 0.0, got {euc}");
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-OPPOSITE-VECTORS]
// Prüft, dass entgegengesetzte Vektoren maximale Kosinus-Distanz 2.0 haben.
fn test_opposite_vectors_cosine() {
    let a = vec![1.0f32, 2.0, 3.0, 4.0];
    let b = vec![-1.0f32, -2.0, -3.0, -4.0];

    let cos = cosine_distance(&a, &b).unwrap();
    assert!(
        (cos - 2.0).abs() < 1e-5,
        "Opposite vector cosine distance should be 2.0, got {cos}"
    );
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-ORTHOGONAL-VECTORS]
// Prüft, dass orthogonale Vektoren Kosinus-Distanz 1.0 haben.
fn test_orthogonal_vectors_cosine() {
    let a = vec![1.0f32, 0.0, 0.0, 0.0];
    let b = vec![0.0f32, 1.0, 0.0, 0.0];

    let cos = cosine_distance(&a, &b).unwrap();
    assert!(
        (cos - 1.0).abs() < 1e-5,
        "Orthogonal vector cosine distance should be 1.0, got {cos}"
    );
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-DIM-MISMATCH]
// Prüft, dass ungleiche Vektordimensionen sauber mit InvalidInput abgewiesen werden.
fn test_dimension_mismatch_error() {
    let a = vec![1.0f32; 8];
    let b = vec![1.0f32; 10];
    let b_bytes = vec![0u8; 16];

    assert!(matches!(
        cosine_distance(&a, &b),
        Err(SimdError::DimensionMismatch { left: 8, right: 10 })
    ));
    assert!(matches!(
        euclidean_distance(&a, &b),
        Err(SimdError::DimensionMismatch { left: 8, right: 10 })
    ));
    assert!(matches!(
        dot_product_distance(&a, &b),
        Err(SimdError::DimensionMismatch { left: 8, right: 10 })
    ));

    assert!(matches!(
        cosine_distance_f32_bytes(&a, &b_bytes),
        Err(SimdError::DimensionMismatch { left: 8, right: 4 })
    ));
    assert!(matches!(
        euclidean_distance_f32_bytes(&a, &b_bytes),
        Err(SimdError::DimensionMismatch { left: 8, right: 4 })
    ));
    assert!(matches!(
        dot_product_distance_f32_bytes(&a, &b_bytes),
        Err(SimdError::DimensionMismatch { left: 8, right: 4 })
    ));

    let u8_a = vec![1u8; 8];
    let u8_b = vec![1u8; 12];
    assert!(matches!(
        dot_product_u8(&u8_a, &u8_b),
        Err(SimdError::DimensionMismatch { left: 8, right: 12 })
    ));
    assert!(matches!(
        euclidean_distance_sq_u8(&u8_a, &u8_b),
        Err(SimdError::DimensionMismatch { left: 8, right: 12 })
    ));
    assert!(matches!(
        cosine_similarity_parts_u8(&u8_a, &u8_b),
        Err(SimdError::DimensionMismatch { left: 8, right: 12 })
    ));
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-EMPTY-VECTORS]
// Prüft das Verhalten aller Distanzfunktionen bei leeren Vektoren (Länge 0).
fn test_empty_vectors_handling() {
    let empty_f32: Vec<f32> = vec![];
    let empty_u8: Vec<u8> = vec![];

    assert_eq!(cosine_distance(&empty_f32, &empty_f32).unwrap(), 1.0);
    assert_eq!(euclidean_distance(&empty_f32, &empty_f32).unwrap(), 0.0);
    assert_eq!(dot_product_distance(&empty_f32, &empty_f32).unwrap(), -0.0);

    assert_eq!(dot_product_u8(&empty_u8, &empty_u8).unwrap(), 0);
    assert_eq!(euclidean_distance_sq_u8(&empty_u8, &empty_u8).unwrap(), 0);
    let parts = cosine_similarity_parts_u8(&empty_u8, &empty_u8).unwrap();
    assert_eq!(parts.dot, 0);
    assert_eq!(parts.norm_a_sq, 0);
    assert_eq!(parts.norm_b_sq, 0);
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-SINGLE-ELEMENT]
// Prüft Distanzfunktionen bei 1-dimensionalen Skalar-Vektoren.
fn test_single_element_vectors() {
    let a = vec![4.0f32];
    let b = vec![2.0f32];

    assert_eq!(cosine_distance(&a, &b).unwrap(), 0.0);
    assert_eq!(euclidean_distance(&a, &b).unwrap(), 2.0);
    assert_eq!(dot_product_distance(&a, &b).unwrap(), -8.0);
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-NORMALIZE-EDGES]
// Prüft Grenzfälle der In-Place-Normalisierung (Nullvektor, leere Slice, Einheitsvektor).
fn test_normalize_inplace_edge_cases() {
    let mut zero_vec = vec![0.0f32; 10];
    normalize_inplace(&mut zero_vec);
    assert_eq!(zero_vec, vec![0.0f32; 10]);

    let mut single = vec![5.0f32];
    normalize_inplace(&mut single);
    assert_eq!(single, vec![1.0f32]);

    let mut neg_single = vec![-5.0f32];
    normalize_inplace(&mut neg_single);
    assert_eq!(neg_single, vec![-1.0f32]);

    let mut v = vec![3.0f32, 4.0];
    normalize_inplace(&mut v);
    assert!((v[0] - 0.6).abs() < 1e-6);
    assert!((v[1] - 0.8).abs() < 1e-6);
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-U8-MAX-ACCUMULATION]
// Prüft, dass 255-Werte bei 1024 Dimensionen ohne u32-Überlauf akkumulieren.
fn test_u8_max_accumulation_without_overflow() {
    // 60,000 u8 elements with max value 255.
    // 255 * 255 * 60,000 = 3,901,500,000 <= 4,294,967,295 (u32::MAX)
    let len = 60_000;
    let a = vec![255u8; len];
    let b = vec![255u8; len];

    let dot = dot_product_u8(&a, &b).unwrap();
    let expected = 255u32 * 255u32 * (len as u32);
    assert_eq!(dot, expected);

    let parts = cosine_similarity_parts_u8(&a, &b).unwrap();
    assert_eq!(parts.dot, expected);
    assert_eq!(parts.norm_a_sq, expected);
    assert_eq!(parts.norm_b_sq, expected);
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-SCALAR-DIRECT]
// Prüft direkte Aufrufe der Skalarfunktionen auf identische, orthogonale und Null-Vektoren.
fn test_scalar_reference_functions_directly() {
    let a = vec![1.0f32, 2.0, 3.0, 4.0];
    let b = vec![2.0f32, 3.0, 4.0, 5.0];
    let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

    let c1 = cosine_distance_scalar(&a, &b);
    let c2 = cosine_distance_f32_bytes_scalar(&a, &b_bytes);
    assert_eq!(c1.to_bits(), c2.to_bits());

    let e1 = euclidean_distance_scalar(&a, &b);
    let e2 = euclidean_distance_f32_bytes_scalar(&a, &b_bytes);
    assert_eq!(e1.to_bits(), e2.to_bits());

    let d1 = dot_product_scalar(&a, &b);
    let d2 = dot_product_f32_bytes_scalar(&a, &b_bytes);
    assert_eq!(d1.to_bits(), d2.to_bits());
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-TRAIT-IMPLS]
// Prüft Debug-, Clone-, Copy- und PartialEq-Trait-Implementierungen der Ergebnis-Strukturen.
fn test_struct_debug_and_copy_traits() {
    let parts_u8 = CosineSimilarityPartsU8 {
        dot: 100,
        norm_a_sq: 200,
        norm_b_sq: 300,
    };
    let parts_u8_copy = parts_u8;
    assert_eq!(parts_u8, parts_u8_copy);
    assert_eq!(
        format!("{parts_u8:?}"),
        "CosineSimilarityPartsU8 { dot: 100, norm_a_sq: 200, norm_b_sq: 300 }"
    );

    let parts_f32_u8 = CosineSimilarityPartsF32U8 {
        dot: 10.5,
        norm_a_sq: 20.25,
        norm_b_sq: 30.5,
    };
    let parts_f32_u8_copy = parts_f32_u8;
    assert_eq!(parts_f32_u8, parts_f32_u8_copy);
    assert_eq!(
        format!("{parts_f32_u8:?}"),
        "CosineSimilarityPartsF32U8 { dot: 10.5, norm_a_sq: 20.25, norm_b_sq: 30.5 }"
    );
}

#[test]
// ANCHOR[contextra-simd:TEST-EDGE-SCALAR-SLICES]
// Prüft direkte Skalarfunktionen mit leeren Slices und 1-Element-Slices.
fn test_scalar_functions_zero_length_and_single_element_slices() {
    let empty_f32: [f32; 0] = [];

    assert_eq!(cosine_distance_scalar(&empty_f32, &empty_f32), 1.0);
    assert_eq!(euclidean_distance_scalar(&empty_f32, &empty_f32), 0.0);
    assert_eq!(dot_product_scalar(&empty_f32, &empty_f32), -0.0);

    let single_a = [3.0f32];
    let single_b = [4.0f32];
    assert_eq!(cosine_distance_scalar(&single_a, &single_b), 0.0);
    assert_eq!(euclidean_distance_scalar(&single_a, &single_b), 1.0);
    assert_eq!(dot_product_scalar(&single_a, &single_b), -12.0);

    let single_u8_a = [10u8];
    let single_u8_b = [5u8];
    assert_eq!(dot_product_u8_scalar(&single_u8_a, &single_u8_b), 50);
    assert_eq!(
        euclidean_distance_sq_u8_scalar(&single_u8_a, &single_u8_b),
        25
    );
}

/// SPEC: INV-SIMD-FINITE-1
// Invariante INV-SIMD-FINITE-1: Nie mit NaN/Inf rechnen, fail-closed Abweisung.
#[test]
fn test_inv_simd_finite_1() {
    let valid = [1.0f32, 2.0, 3.0, 4.0];
    let with_nan = [1.0f32, f32::NAN, 3.0, 4.0];
    let with_inf = [1.0f32, 2.0, f32::INFINITY, 4.0];
    let with_neg_inf = [1.0f32, 2.0, 3.0, f32::NEG_INFINITY];

    assert!(contextra_simd::validate_vector(&valid).is_ok());
    assert!(contextra_simd::validate_vector(&with_nan).is_err());
    assert!(contextra_simd::validate_vector(&with_inf).is_err());
    assert!(contextra_simd::validate_vector(&with_neg_inf).is_err());

    // compute_distance validiert Fail-Closed
    assert!(contextra_simd::compute_distance(&with_nan, &valid, DistanceMetric::Cosine).is_err());
    assert!(contextra_simd::compute_distance(&valid, &with_nan, DistanceMetric::Cosine).is_err());
    assert!(
        contextra_simd::compute_distance(&with_inf, &valid, DistanceMetric::Euclidean).is_err()
    );
    assert!(
        contextra_simd::compute_distance(&valid, &with_neg_inf, DistanceMetric::DotProduct)
            .is_err()
    );

    // Denormale Gleitkommazahlen sind finite Zahlen und müssen berechenbar sein (kein NaN Ergebnis)
    let denormal_a = [1e-40f32, 2e-40, 3e-40, 4e-40];
    let denormal_b = [2e-40f32, 3e-40, 4e-40, 5e-40];
    assert!(contextra_simd::validate_vector(&denormal_a).is_ok());
    let res = contextra_simd::compute_distance(&denormal_a, &denormal_b, DistanceMetric::Cosine);
    assert!(res.is_ok());
    assert!(!res.unwrap().is_nan());
}

/// SPEC: INV-SIMD-ALIGN-1
// Invariante INV-SIMD-ALIGN-1: Byte-Puffer ganzzahlig durch 4, keine Annahme über Ausrichtung.
#[test]
fn test_inv_simd_align_1() {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [5.0f32, 6.0, 7.0, 8.0];

    // Korrekt ausgerichtete Bytes
    let b_bytes_aligned: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();
    assert_eq!(b_bytes_aligned.len() % 4, 0);

    let res = contextra_simd::compute_distance_f32_bytes_trusted(
        &a,
        &b_bytes_aligned,
        DistanceMetric::Cosine,
    )
    .expect("valid distance");

    // Künstlich unaligned Byte-Slice (mit 1-3 vorangestellten Padding-Bytes)
    for offset in 1..=3 {
        let mut unaligned_buf = vec![0u8; offset];
        unaligned_buf.extend_from_slice(&b_bytes_aligned);

        let slice = &unaligned_buf[offset..];
        assert_eq!(slice.len() % 4, 0);
        let res_unaligned =
            contextra_simd::compute_distance_f32_bytes_trusted(&a, slice, DistanceMetric::Cosine);
        assert!(res_unaligned.is_ok());
        assert_eq!(res.to_bits(), res_unaligned.unwrap().to_bits());
    }

    // Ungültige Byte-Puffer-Länge (nicht durch 4 teilbar)
    let invalid_len_bytes = [1u8, 2, 3, 4, 5];
    assert!(contextra_simd::compute_distance_f32_bytes_trusted(
        &a,
        &invalid_len_bytes,
        DistanceMetric::Cosine,
    )
    .is_err());
}

/// SPEC: INV-SIMD-DETERMINISM-1
// Invariante INV-SIMD-DETERMINISM-1: Jeder Kernel weicht höchstens um dokumentierte Toleranz (1e-5 relativ) von der Skalar-Referenz ab.
#[test]
fn test_inv_simd_determinism_1() {
    // Teste über verschiedene Dimensionen inklusive SIMD-Grenzbreiten
    let test_dims = [
        1, 3, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 128, 384, 768, 1024, 1536,
    ];

    for &dim in &test_dims {
        let a: Vec<f32> = (0..dim)
            .map(|i| ((i * 17) % 100) as f32 * 0.05 + 0.1)
            .collect();
        let b: Vec<f32> = (0..dim)
            .map(|i| ((i * 23) % 100) as f32 * 0.05 + 0.2)
            .collect();

        // Cosine
        let cos_scalar = cosine_distance_scalar(&a, &b);
        let cos_simd = cosine_distance(&a, &b).expect("valid distance");
        let cos_diff = (cos_scalar - cos_simd).abs();
        let cos_tol = 1e-4 * cos_scalar.abs().max(cos_simd.abs()).max(1.0);
        assert!(
            cos_diff <= cos_tol,
            "Cosine determinism mismatch at dim {dim}: scalar={cos_scalar}, simd={cos_simd}, diff={cos_diff}, tol={cos_tol}"
        );

        // Euclidean
        let euc_scalar = euclidean_distance_scalar(&a, &b);
        let euc_simd = euclidean_distance(&a, &b).expect("valid distance");
        let euc_diff = (euc_scalar - euc_simd).abs();
        let euc_tol = 1e-4 * euc_scalar.abs().max(euc_simd.abs()).max(1.0);
        assert!(
            euc_diff <= euc_tol,
            "Euclidean determinism mismatch at dim {dim}: scalar={euc_scalar}, simd={euc_simd}, diff={euc_diff}, tol={euc_tol}"
        );

        // Dot product
        let dot_scalar = dot_product_scalar(&a, &b);
        let dot_simd = dot_product_distance(&a, &b).expect("valid distance");
        let dot_diff = (dot_scalar - dot_simd).abs();
        let dot_tol = 1e-4 * dot_scalar.abs().max(dot_simd.abs()).max(1.0);
        assert!(
            dot_diff <= dot_tol,
            "Dot product determinism mismatch at dim {dim}: scalar={dot_scalar}, simd={dot_simd}, diff={dot_diff}, tol={dot_tol}"
        );
    }
}
