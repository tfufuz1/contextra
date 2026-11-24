// FILE-CONTEXT
// ZWECK: Fehlerschranken-, Serialisierungs- und Stabilitätstests für SQ8 (WP-VEC-06).
// INVARIANTEN: Rekonstruktionsfehler <= Delta / 510 je Dimension; Bincode/Serde Roundtrip.
// STAND: 2026-11-12

use contextra_vector::quantize::{ScalarQuantizer, SQ8_MAX_ERROR_FACTOR_PER_DIM};
use proptest::prelude::*;

#[test]
fn test_sq8_constant_dimension() {
    let batch: Vec<Vec<f32>> = vec![
        vec![5.0, 10.0, -2.0],
        vec![5.0, 20.0, -2.0],
        vec![5.0, 15.0, -2.0],
    ];
    let slice_batch: Vec<&[f32]> = batch.iter().map(|v| v.as_slice()).collect();

    let quantizer = ScalarQuantizer::try_train(&slice_batch, 3).unwrap();
    assert!(quantizer.scales()[0] > 0.0);
    assert!(quantizer.inv_scales()[0] > 0.0);
    assert!(quantizer.scales()[2] > 0.0);

    let test_vec = [5.0, 15.0, -2.0];
    let encoded = quantizer.quantize(&test_vec).unwrap();
    let decoded = quantizer.dequantize(&encoded).unwrap();

    assert_eq!(decoded[0], 5.0);
    assert_eq!(decoded[2], -2.0);
    assert!((decoded[1] - 15.0).abs() <= (20.0 - 10.0) / 255.0);
}

#[test]
fn test_sq8_extreme_ranges_and_non_finite_rejection() {
    let batch: Vec<Vec<f32>> = vec![vec![-1e20, 1e20], vec![0.0, 0.0], vec![1e20, -1e20]];
    let slice_batch: Vec<&[f32]> = batch.iter().map(|v| v.as_slice()).collect();
    let quantizer = ScalarQuantizer::try_train(&slice_batch, 2).unwrap();

    let q = quantizer.quantize(&[0.0, 0.0]).unwrap();
    assert_eq!(q.len(), 2);

    let nan_batch: Vec<Vec<f32>> = vec![vec![1.0, f32::NAN], vec![2.0, 3.0]];
    let nan_slices: Vec<&[f32]> = nan_batch.iter().map(|v| v.as_slice()).collect();
    assert!(ScalarQuantizer::try_train(&nan_slices, 2).is_err());

    let inf_batch: Vec<Vec<f32>> = vec![vec![1.0, f32::INFINITY], vec![2.0, 3.0]];
    let inf_slices: Vec<&[f32]> = inf_batch.iter().map(|v| v.as_slice()).collect();
    assert!(ScalarQuantizer::try_train(&inf_slices, 2).is_err());
}

#[test]
fn test_sq8_serde_roundtrip() {
    let batch: Vec<Vec<f32>> = vec![
        vec![1.0, 2.0, 3.0, 4.0],
        vec![4.0, 3.0, 2.0, 1.0],
        vec![2.0, 2.5, 3.5, 2.0],
    ];
    let slice_batch: Vec<&[f32]> = batch.iter().map(|v| v.as_slice()).collect();
    let quantizer = ScalarQuantizer::try_train(&slice_batch, 4).unwrap();

    let serialized = bincode::serialize(&quantizer).unwrap();
    let deserialized: ScalarQuantizer = bincode::deserialize(&serialized).unwrap();

    assert_eq!(quantizer.dimension(), deserialized.dimension());
    assert_eq!(quantizer.mins(), deserialized.mins());
    assert_eq!(quantizer.maxes(), deserialized.maxes());
    assert_eq!(quantizer.scales(), deserialized.scales());
    assert_eq!(quantizer.inv_scales(), deserialized.inv_scales());

    let test_vec = [2.0, 2.5, 3.0, 3.5];
    assert_eq!(
        quantizer.quantize(&test_vec).unwrap(),
        deserialized.quantize(&test_vec).unwrap()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_sq8_error_bound(
        dim in 4..16usize,
        min_vals in prop::collection::vec(-50.0..0.0f32, 16),
        ranges in prop::collection::vec(0.1..50.0f32, 16)
    ) {
        let dimension = dim;
        let mut batch = Vec::new();
        for _ in 0..10 {
            let mut vec = Vec::with_capacity(dimension);
            for d in 0..dimension {
                vec.push(min_vals[d] + (ranges[d] * 0.5));
            }
            batch.push(vec);
        }
        for d in 0..dimension {
            batch[0][d] = min_vals[d];
            batch[1][d] = min_vals[d] + ranges[d];
        }

        let slice_batch: Vec<&[f32]> = batch.iter().map(|v| v.as_slice()).collect();
        let quantizer = ScalarQuantizer::try_train_with_percentiles(&slice_batch, dimension, 0.0, 1.0).unwrap();

        for vec in &batch {
            let encoded = quantizer.quantize(vec).unwrap();
            let decoded = quantizer.dequantize(&encoded).unwrap();

            for d in 0..dimension {
                let err = (vec[d] - decoded[d]).abs();
                let max_bound = (quantizer.maxes()[d] - quantizer.mins()[d]) * SQ8_MAX_ERROR_FACTOR_PER_DIM + 1e-5;
                assert!(
                    err <= max_bound,
                    "Reconstruction error {err} exceeds bound {max_bound} on dim {d}"
                );
            }
        }
    }
}
