// FILE-CONTEXT
// ZWECK: Edge-case tests covering NaN, Infinity, denormals, zero-vectors, huge magnitudes, and error boundaries.

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
use contextra_types::{ContextraError, DistanceMetric};

#[test]
fn test_nan_detection_in_vector() {
    let nan_vec = vec![1.0f32, 2.0, f32::NAN, 4.0];
    let res = validate_vector(&nan_vec);
    assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
}

#[test]
fn test_infinity_detection_in_vector() {
    let inf_vec = vec![1.0f32, f32::INFINITY, 3.0];
    assert!(matches!(
        validate_vector(&inf_vec),
        Err(ContextraError::InvalidInput(_))
    ));

    let neg_inf_vec = vec![1.0f32, f32::NEG_INFINITY, 3.0];
    assert!(matches!(
        validate_vector(&neg_inf_vec),
        Err(ContextraError::InvalidInput(_))
    ));
}

#[test]
fn test_validate_vector_nan_and_inf_at_boundary_indices() {
    let mut vec_first_nan = vec![0.0f32; 100];
    vec_first_nan[0] = f32::NAN;
    assert!(validate_vector(&vec_first_nan).is_err());

    let mut vec_last_inf = vec![0.0f32; 100];
    vec_last_inf[99] = f32::INFINITY;
    assert!(validate_vector(&vec_last_inf).is_err());
}

#[test]
fn test_validate_vector_empty_and_large_valid_slices() {
    let empty: Vec<f32> = vec![];
    assert!(validate_vector(&empty).is_ok());

    let large_valid: Vec<f32> = (0..10_000).map(|i| (i as f32) * 0.01 - 50.0).collect();
    assert!(validate_vector(&large_valid).is_ok());
}

#[test]
fn test_compute_distance_rejects_nan_and_inf() {
    let valid = vec![1.0f32, 2.0, 3.0];
    let nan = vec![1.0f32, f32::NAN, 3.0];
    let inf = vec![1.0f32, f32::INFINITY, 3.0];

    assert!(compute_distance(&nan, &valid, DistanceMetric::Cosine).is_err());
    assert!(compute_distance(&valid, &inf, DistanceMetric::Euclidean).is_err());
    assert!(compute_distance(&inf, &nan, DistanceMetric::DotProduct).is_err());
}

#[test]
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
fn test_zero_vector_cosine_distance() {
    let zero = vec![0.0f32; 128];
    let v = vec![1.0f32; 128];

    assert_eq!(cosine_distance(&zero, &zero).unwrap(), 1.0);
    assert_eq!(cosine_distance(&zero, &v).unwrap(), 1.0);
    assert_eq!(cosine_distance(&v, &zero).unwrap(), 1.0);
    assert_eq!(cosine_distance_scalar(&zero, &zero), 1.0);
}

#[test]
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
fn test_dimension_mismatch_error() {
    let a = vec![1.0f32; 8];
    let b = vec![1.0f32; 10];
    let b_bytes = vec![0u8; 16];

    assert!(matches!(
        cosine_distance(&a, &b),
        Err(ContextraError::EmbeddingDimensionMismatch {
            expected: 8,
            got: 10
        })
    ));
    assert!(matches!(
        euclidean_distance(&a, &b),
        Err(ContextraError::EmbeddingDimensionMismatch {
            expected: 8,
            got: 10
        })
    ));
    assert!(matches!(
        dot_product_distance(&a, &b),
        Err(ContextraError::EmbeddingDimensionMismatch {
            expected: 8,
            got: 10
        })
    ));

    assert!(matches!(
        cosine_distance_f32_bytes(&a, &b_bytes),
        Err(ContextraError::EmbeddingDimensionMismatch {
            expected: 8,
            got: 4
        })
    ));
    assert!(matches!(
        euclidean_distance_f32_bytes(&a, &b_bytes),
        Err(ContextraError::EmbeddingDimensionMismatch {
            expected: 8,
            got: 4
        })
    ));
    assert!(matches!(
        dot_product_distance_f32_bytes(&a, &b_bytes),
        Err(ContextraError::EmbeddingDimensionMismatch {
            expected: 8,
            got: 4
        })
    ));

    let u8_a = vec![1u8; 8];
    let u8_b = vec![1u8; 12];
    assert!(matches!(
        dot_product_u8(&u8_a, &u8_b),
        Err(ContextraError::EmbeddingDimensionMismatch {
            expected: 8,
            got: 12
        })
    ));
    assert!(matches!(
        euclidean_distance_sq_u8(&u8_a, &u8_b),
        Err(ContextraError::EmbeddingDimensionMismatch {
            expected: 8,
            got: 12
        })
    ));
    assert!(matches!(
        cosine_similarity_parts_u8(&u8_a, &u8_b),
        Err(ContextraError::EmbeddingDimensionMismatch {
            expected: 8,
            got: 12
        })
    ));
}

#[test]
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
fn test_single_element_vectors() {
    let a = vec![4.0f32];
    let b = vec![2.0f32];

    assert_eq!(cosine_distance(&a, &b).unwrap(), 0.0);
    assert_eq!(euclidean_distance(&a, &b).unwrap(), 2.0);
    assert_eq!(dot_product_distance(&a, &b).unwrap(), -8.0);
}

#[test]
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
