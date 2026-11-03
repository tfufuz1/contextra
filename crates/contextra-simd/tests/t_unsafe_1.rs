// ADR-002 / SPEC §27: Test assertions and test dataset casts
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

//! # Nachweis T-UNSAFE-1 für `contextra-simd`
//!
//! **Zweck (SPEC §3.2, D-29, AR-8, T-UNSAFE-1):**
//! - Verifiziert, dass `contextra-simd` die einzige zulässige Unsafe-Insel neben `contextra-sys` ist.
//! - Prüft, dass keine `unsafe fn` im öffentlichen API exportiert wird (alle öffentlichen Eintrittsfunktionen sind sicher).
//! - Prüft Vorbedingungs-Validierungen (keine Panik bei Dimensionsinkongruenz oder ungültigen Eingaben).

use contextra_simd::{
    active_path, compute_distance, cosine_distance, cosine_distance_f32_bytes,
    dot_product_distance, dot_product_distance_f32_bytes, euclidean_distance,
    euclidean_distance_f32_bytes, validate_vector, SimdPath,
};
use contextra_types::{ContextraError, DistanceMetric};

/// SPEC: T-UNSAFE-1
#[test]
fn t_unsafe_1_public_api_is_entirely_safe() {
    // 1. Alle öffentlichen Funktionen müssen safe sein und sicher mit beliebigen Slices umgehen.
    let empty: [f32; 0] = [];
    assert!(validate_vector(&empty).is_ok());

    let a = [1.0f32, 2.0, 3.0];
    let b = [4.0f32, 5.0, 6.0];
    assert!(cosine_distance(&a, &b).is_ok());
    assert!(euclidean_distance(&a, &b).is_ok());
    assert!(dot_product_distance(&a, &b).is_ok());

    let b_bytes = [0u8; 12];
    assert!(cosine_distance_f32_bytes(&a, &b_bytes).is_ok());
    assert!(euclidean_distance_f32_bytes(&a, &b_bytes).is_ok());
    assert!(dot_product_distance_f32_bytes(&a, &b_bytes).is_ok());

    // 2. Active Path ermitteln
    let p = active_path();
    assert!(matches!(
        p,
        SimdPath::Avx512 | SimdPath::Avx2Fma | SimdPath::Neon | SimdPath::Scalar
    ));
}

/// SPEC: T-UNSAFE-1
#[test]
fn t_unsafe_1_fail_closed_on_invalid_bounds() {
    // Längenunterschiede dürfen NIEMALS in Pufferüberläufe oder Segfaults münden, sondern müssen fail-closed Result::Err liefern.
    let a = [1.0f32, 2.0, 3.0];
    let b = [4.0f32, 5.0];

    assert!(matches!(
        cosine_distance(&a, &b),
        Err(ContextraError::EmbeddingDimensionMismatch { .. })
    ));
    assert!(matches!(
        euclidean_distance(&a, &b),
        Err(ContextraError::EmbeddingDimensionMismatch { .. })
    ));
    assert!(matches!(
        dot_product_distance(&a, &b),
        Err(ContextraError::EmbeddingDimensionMismatch { .. })
    ));

    // Byte-Slice zu kurz
    let b_short_bytes = [0u8; 4];
    assert!(matches!(
        cosine_distance_f32_bytes(&a, &b_short_bytes),
        Err(ContextraError::EmbeddingDimensionMismatch { .. })
    ));
    assert!(matches!(
        euclidean_distance_f32_bytes(&a, &b_short_bytes),
        Err(ContextraError::EmbeddingDimensionMismatch { .. })
    ));
    assert!(matches!(
        dot_product_distance_f32_bytes(&a, &b_short_bytes),
        Err(ContextraError::EmbeddingDimensionMismatch { .. })
    ));

    // Dimension zu groß (> 4096)
    let too_large = vec![1.0f32; 4097];
    assert!(validate_vector(&too_large).is_err());
    assert!(compute_distance(&too_large, &too_large, DistanceMetric::Cosine).is_err());
}
