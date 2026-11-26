// FILE-CONTEXT
// ZWECK: Validierung der autarken Baustein-Architektur und Schnittstellen-Integrität für contextra-simd.
// INVARIANTEN: Deterministische Differential-Konsistenz, Safe API, Fail-Closed Fehlerbehandlung.

use contextra_simd::{
    compute_distance, compute_distance_f32_bytes_trusted, compute_distance_trusted, validate_vector,
};
use contextra_simd::{DistanceMetric, SimdError};

#[test]
fn test_building_block_safe_api_and_validation() -> Result<(), SimdError> {
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
fn test_building_block_injectable_dispatcher_contract() -> Result<(), SimdError> {
    use contextra_simd::dispatch::{Dispatcher, SimdTarget};

    let auto_disp = Dispatcher::auto();
    let scalar_disp = Dispatcher::scalar();

    assert_eq!(scalar_disp.target(), SimdTarget::Scalar);

    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [2.0f32, 3.0, 4.0, 5.0];

    let cos_auto = auto_disp.cosine_distance(&a, &b)?;
    let cos_scalar = scalar_disp.cosine_distance(&a, &b)?;
    assert!((cos_auto - cos_scalar).abs() < 1e-4);

    let euc_auto = auto_disp.euclidean_distance(&a, &b)?;
    let euc_scalar = scalar_disp.euclidean_distance(&a, &b)?;
    assert!((euc_auto - euc_scalar).abs() < 1e-4);

    let dot_auto = auto_disp.dot_product_distance(&a, &b)?;
    let dot_scalar = scalar_disp.dot_product_distance(&a, &b)?;
    assert!((dot_auto - dot_scalar).abs() < 1e-4);

    let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();
    let cos_b = auto_disp.cosine_distance_f32_bytes(&a, &b_bytes)?;
    assert!((cos_b - cos_auto).abs() < 1e-5);

    let u8_a = [10u8, 20, 30, 40];
    let u8_b = [5u8, 15, 25, 35];
    let dot_u8 = auto_disp.dot_product_u8(&u8_a, &u8_b)?;
    let euc_u8 = auto_disp.euclidean_distance_sq_u8(&u8_a, &u8_b)?;
    let parts_u8 = auto_disp.cosine_similarity_parts_u8(&u8_a, &u8_b)?;
    assert_eq!(dot_u8, parts_u8.dot);
    assert_eq!(euc_u8, 100);

    Ok(())
}

#[test]
fn test_building_block_byte_slice_and_trusted_fast_path() -> Result<(), SimdError> {
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
