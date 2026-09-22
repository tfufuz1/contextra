use serde::{Deserialize, Serialize};

/// Stub or reference structure for SQ8 Bias Calibration.
///
/// Once Prompt 3.2 is merged, `Sq8Bias` will be imported directly from `memfuse_vector::quantize::Sq8Bias`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sq8Bias {
    /// Dimension-wise bias offset vector computed from quantization error.
    pub dimension_biases: Vec<f32>,
    /// Mean quantization error across all dimensions.
    pub mean_error: f32,
    /// Variance of quantization error.
    pub error_variance: f32,
}

impl Sq8Bias {
    /// Calibrates SQ8 bias from sample original vectors and their quantized/dequantized counterparts.
    pub fn calibrate(original: &[Vec<f32>], dequantized: &[Vec<f32>]) -> Self {
        if original.is_empty() || original.len() != dequantized.len() {
            return Self {
                dimension_biases: Vec::new(),
                mean_error: 0.0,
                error_variance: 0.0,
            };
        }

        let dim = original[0].len();
        let num_samples = original.len() as f32;
        let mut dim_sums = vec![0.0f32; dim];
        let mut total_error_sum = 0.0f32;

        for (orig, deq) in original.iter().zip(dequantized.iter()) {
            for d in 0..dim {
                let err = orig[d] - deq[d];
                dim_sums[d] += err;
                total_error_sum += err.abs();
            }
        }

        let dimension_biases: Vec<f32> = dim_sums.into_iter().map(|s| s / num_samples).collect();
        let mean_error = total_error_sum / (num_samples * dim as f32);

        let mut variance_sum = 0.0f32;
        for (orig, deq) in original.iter().zip(dequantized.iter()) {
            for d in 0..dim {
                let err = orig[d] - deq[d];
                let diff = err.abs() - mean_error;
                variance_sum += diff * diff;
            }
        }
        let error_variance = variance_sum / (num_samples * dim as f32);

        Self {
            dimension_biases,
            mean_error,
            error_variance,
        }
    }
}

#[test]
#[ignore = "blocked on Sq8Bias impl"]
fn test_sq8_bias_calibration_and_persistence() {
    let dim = 4;
    let original = vec![
        vec![1.0, 2.0, 3.0, 4.0],
        vec![0.5, 1.5, 2.5, 3.5],
        vec![2.0, 3.0, 4.0, 5.0],
    ];

    // Simulated quantized-dequantized vectors with artificial quantization bias
    let dequantized = vec![
        vec![0.9, 2.1, 2.9, 4.2],
        vec![0.4, 1.6, 2.4, 3.7],
        vec![1.9, 3.1, 3.9, 5.2],
    ];

    let bias = Sq8Bias::calibrate(&original, &dequantized);

    assert_eq!(bias.dimension_biases.len(), dim);
    assert!(bias.mean_error > 0.0);
    assert!(bias.error_variance >= 0.0);

    // Test serialization / header reconstruction
    let serialized = bincode::serialize(&bias).expect("Serialization failed");
    let deserialized: Sq8Bias = bincode::deserialize(&serialized).expect("Deserialization failed");

    assert_eq!(bias, deserialized);
}
