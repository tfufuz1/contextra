// FILE-CONTEXT
// ZWECK: Validierung der autarken Baustein-Architektur und Schnittstellen-Integrität für contextra-simd.
// INVARIANTEN: Deterministische Differential-Konsistenz, Safe API, Fail-Closed Fehlerbehandlung.

use contextra_simd::{
    compute_distance, compute_distance_f32_bytes_trusted, compute_distance_trusted, validate_vector,
};
use contextra_types::{ContextraError, DistanceMetric};

#[test]
fn test_building_block_safe_api_and_validation() -> Result<(), ContextraError> {
    let a = [1.0f32, 0.0, 0.0];
    let b = [0.0f32, 1.0, 0.0];

    assert!(validate_vector(&a).is_ok());
    assert!(validate_vector(&b).is_ok());

    let invalid = [1.0f32, f32::NAN, 0.0];
    assert!(validate_vector(&invalid).is_err());

    let dist = compute_distance(&a, &b, DistanceMetric::Cosine)?;
    assert_eq!(dist, 1.0);

    let euc = compute_distance(&a, &b, DistanceMetric::Euclidean)?;
    assert!((euc - std::f32::consts::SQRT_2).abs() < 1e-6);

    let dot = compute_distance(&a, &b, DistanceMetric::DotProduct)?;
    assert_eq!(dot, 0.0);

    Ok(())
}

#[test]
fn test_building_block_byte_slice_and_trusted_fast_path() -> Result<(), ContextraError> {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [2.0f32, 3.0, 4.0, 5.0];
    let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

    let cos_val = compute_distance_trusted(&a, &b, DistanceMetric::Cosine)?;
    let cos_bytes = compute_distance_f32_bytes_trusted(&a, &b_bytes, DistanceMetric::Cosine)?;
    assert_eq!(cos_val.to_bits(), cos_bytes.to_bits());

    let euc_val = compute_distance_trusted(&a, &b, DistanceMetric::Euclidean)?;
    let euc_bytes = compute_distance_f32_bytes_trusted(&a, &b_bytes, DistanceMetric::Euclidean)?;
    assert_eq!(euc_val.to_bits(), euc_bytes.to_bits());

    let dot_val = compute_distance_trusted(&a, &b, DistanceMetric::DotProduct)?;
    let dot_bytes = compute_distance_f32_bytes_trusted(&a, &b_bytes, DistanceMetric::DotProduct)?;
    assert_eq!(dot_val.to_bits(), dot_bytes.to_bits());

    Ok(())
}
