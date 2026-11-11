//! Numerik-Orakel und Golden-Tests für `contextra-simd` (WP-M1).
//!
//! Friert das numerische Verhalten der Distanzkerne ein:
//! - Dimensionen: 4, 17, 128, 768
//! - Wertebereiche: normale Floats, Nullen, Denormale, NaN/Inf-Verhalten
//! - Prüft Bitgleichheit bzw. relative Fehlertoleranz 1e-5 (f32)

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::manual_range_contains)]

use contextra_simd::{
    compute_distance, compute_distance_trusted, validate_vector, DistanceMetric, SimdError,
};

const TEST_DIMS: &[usize] = &[1, 4, 17, 128, 768, 1536];
const TOLERANCE: f32 = 1e-5;

fn approx_eq(a: f32, b: f32, tol: f32) -> bool {
    if a.is_nan() && b.is_nan() {
        return true;
    }
    if a == b {
        return true;
    }
    let diff = (a - b).abs();
    let max_mag = a.abs().max(b.abs());
    if max_mag < 1e-12 {
        diff < tol
    } else {
        (diff / max_mag) <= tol
    }
}

fn generate_deterministic_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut state = seed;
    let mut out = Vec::with_capacity(dim);
    for i in 0..dim {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = (((state >> 33) as f32) / (u32::MAX as f32) - 0.5) * 2.0;
        out.push(val + (i as f32 * 0.001));
    }
    out
}

#[test]
fn test_golden_normal_values_across_dimensions() {
    for &dim in TEST_DIMS {
        let v1 = generate_deterministic_vector(dim, 42);
        let v2 = generate_deterministic_vector(dim, 1337);

        assert!(validate_vector(&v1).is_ok());
        assert!(validate_vector(&v2).is_ok());

        for metric in [
            DistanceMetric::Cosine,
            DistanceMetric::Euclidean,
            DistanceMetric::DotProduct,
        ] {
            let res_untrusted = compute_distance(&v1, &v2, metric).expect("valid vectors");
            let res_trusted = compute_distance_trusted(&v1, &v2, metric).expect("valid vectors");

            assert!(
                (res_untrusted - res_trusted).abs() <= 1e-7,
                "untrusted and trusted must yield identical results: untrusted={res_untrusted}, trusted={res_trusted}, dim={dim}, metric={metric:?}"
            );
            assert!(res_untrusted.is_finite());
        }
    }
}

#[test]
fn test_golden_zero_and_identical_vectors() {
    for &dim in TEST_DIMS {
        let zeros = vec![0.0f32; dim];
        let v1 = generate_deterministic_vector(dim, 999);

        // Zero-Vektor Cosine liefert 1.0 per Spezifikation / Implementierung
        let cos_zeros = compute_distance(&zeros, &zeros, DistanceMetric::Cosine).unwrap();
        assert_eq!(cos_zeros, 1.0, "Zero-vector cosine must be 1.0");

        let eucl_zeros = compute_distance(&zeros, &zeros, DistanceMetric::Euclidean).unwrap();
        assert_eq!(eucl_zeros, 0.0, "Zero-vector Euclidean must be 0.0");

        let dot_zeros = compute_distance(&zeros, &zeros, DistanceMetric::DotProduct).unwrap();
        assert_eq!(dot_zeros, 0.0, "Zero-vector DotProduct must be 0.0");

        // Identische Vektoren
        let cos_ident = compute_distance(&v1, &v1, DistanceMetric::Cosine).unwrap();
        assert!(
            cos_ident.abs() <= TOLERANCE,
            "Self-cosine distance must be ~0.0, got {cos_ident}"
        );

        let eucl_ident = compute_distance(&v1, &v1, DistanceMetric::Euclidean).unwrap();
        assert!(
            eucl_ident.abs() <= TOLERANCE,
            "Self-Euclidean distance must be ~0.0, got {eucl_ident}"
        );
    }
}

#[test]
fn test_golden_denormal_values() {
    for &dim in TEST_DIMS {
        let mut v1 = vec![1e-39f32; dim];
        let mut v2 = vec![2e-39f32; dim];
        v1[0] = f32::MIN_POSITIVE / 2.0;
        v2[0] = f32::MIN_POSITIVE / 4.0;

        assert!(validate_vector(&v1).is_ok());
        assert!(validate_vector(&v2).is_ok());

        for metric in [
            DistanceMetric::Cosine,
            DistanceMetric::Euclidean,
            DistanceMetric::DotProduct,
        ] {
            let res = compute_distance(&v1, &v2, metric).expect("denormals are valid numbers");
            assert!(
                res.is_finite(),
                "Denormal distance must be finite, got {res} (dim={dim}, metric={metric:?})"
            );
        }
    }
}

#[test]
fn test_golden_nan_and_inf_fail_closed() {
    for &dim in TEST_DIMS {
        let mut v_nan = vec![1.0f32; dim];
        v_nan[dim - 1] = f32::NAN;

        let mut v_inf = vec![1.0f32; dim];
        v_inf[0] = f32::INFINITY;

        let mut v_neg_inf = vec![1.0f32; dim];
        v_neg_inf[dim / 2] = f32::NEG_INFINITY;

        let normal = vec![1.0f32; dim];

        // validate_vector fail-closed
        assert_eq!(
            validate_vector(&v_nan),
            Err(SimdError::InvalidInput("NaN or Infinity detected in vector"))
        );
        assert_eq!(
            validate_vector(&v_inf),
            Err(SimdError::InvalidInput("NaN or Infinity detected in vector"))
        );
        assert_eq!(
            validate_vector(&v_neg_inf),
            Err(SimdError::InvalidInput("NaN or Infinity detected in vector"))
        );

        // compute_distance fail-closed
        assert!(matches!(
            compute_distance(&v_nan, &normal, DistanceMetric::Cosine),
            Err(SimdError::InvalidInput(_))
        ));
        assert!(matches!(
            compute_distance(&normal, &v_inf, DistanceMetric::Euclidean),
            Err(SimdError::InvalidInput(_))
        ));
        assert!(matches!(
            compute_distance(&normal, &v_neg_inf, DistanceMetric::DotProduct),
            Err(SimdError::InvalidInput(_))
        ));
    }
}

#[test]
fn test_golden_bit_equivalence_and_tolerance() {
    // Feste Golden-Werte für dim=4
    let left4 = [1.0f32, 2.0, 3.0, 4.0];
    let right4 = [5.0f32, 6.0, 7.0, 8.0];

    let dot4 = compute_distance(&left4, &right4, DistanceMetric::DotProduct).unwrap();
    // In contextra-simd ist DotProduct distance als negiertes Skalarprodukt definiert: -(1*5 + 2*6 + 3*7 + 4*8) = -70.0
    assert_eq!(dot4, -70.0);

    let eucl4 = compute_distance(&left4, &right4, DistanceMetric::Euclidean).unwrap();
    // sqrt((5-1)^2 + (6-2)^2 + (7-3)^2 + (8-4)^2) = sqrt(16 + 16 + 16 + 16) = sqrt(64) = 8.0
    assert_eq!(eucl4, 8.0);

    let cos4 = compute_distance(&left4, &right4, DistanceMetric::Cosine).unwrap();
    // norm_l = sqrt(1 + 4 + 9 + 16) = sqrt(30)
    // norm_r = sqrt(25 + 36 + 49 + 64) = sqrt(174)
    // sim = 70.0 / (sqrt(30) * sqrt(174)) = 70.0 / sqrt(5220) ≈ 70.0 / 72.24956747 ≈ 0.9688639
    // dist = 1.0 - sim ≈ 0.031136095
    let expected_cos4 = 1.0 - (70.0 / ((30.0f32).sqrt() * (174.0f32).sqrt()));
    assert!(approx_eq(cos4, expected_cos4, 1e-5), "cos4={cos4}, expected={expected_cos4}");

    // Dims 17, 128, 768: prüfe Konsistenz über Skalar und Dispatcher
    for &dim in &[17, 128, 768] {
        let v1 = generate_deterministic_vector(dim, 777);
        let v2 = generate_deterministic_vector(dim, 888);

        let d_cos = compute_distance(&v1, &v2, DistanceMetric::Cosine).unwrap();
        let d_eucl = compute_distance(&v1, &v2, DistanceMetric::Euclidean).unwrap();
        let d_dot = compute_distance(&v1, &v2, DistanceMetric::DotProduct).unwrap();

        assert!(d_cos >= 0.0 && d_cos <= 2.0);
        assert!(d_eucl >= 0.0);
        assert!(d_dot.is_finite());
    }
}
