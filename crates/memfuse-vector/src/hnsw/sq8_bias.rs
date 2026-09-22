// FILE-CONTEXT
// ZWECK: Quantisierungs-Bias-Kalibrierung (Sq8Bias) für SQ8-Quantisierung im HNSW Index Header.
// INVARIANTEN: Zero-Panic Doctrine; Gemessener Quantisierungs-Bias im Index-Header.

//! SQ8 quantization bias calibration module.

use serde::{Deserialize, Serialize};

/// Measured quantization bias and variance statistics for SQ8 distance calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sq8Bias {
    /// Mean quantization error across distance computations.
    pub mean_bias: f32,
    /// Variance of quantization error across distance computations.
    pub variance_bias: f32,
    /// Number of sample pairs used during calibration.
    pub sample_count: u64,
}

impl Default for Sq8Bias {
    fn default() -> Self {
        Self {
            mean_bias: 0.0,
            variance_bias: 0.0,
            sample_count: 0,
        }
    }
}

impl Sq8Bias {
    /// Creates a new `Sq8Bias` instance with calibrated parameters.
    pub fn new(mean_bias: f32, variance_bias: f32, sample_count: u64) -> Self {
        Self {
            mean_bias,
            variance_bias,
            sample_count,
        }
    }

    /// Calibrates quantization error statistics across a representative batch of vectors.
    pub fn calibrate(
        batch: &[&[f32]],
        quantizer: &crate::quantize::ScalarQuantizer,
        metric: memfuse_core::DistanceMetric,
    ) -> Self {
        if batch.len() < 2 {
            return Self::default();
        }

        let sample_limit = batch.len().min(150);
        let mut diffs = Vec::with_capacity(sample_limit * (sample_limit - 1) / 2);

        for i in 0..sample_limit {
            let q_i = match quantizer.quantize(batch[i]) {
                Ok(q) => q,
                Err(_) => continue,
            };
            for j in (i + 1)..sample_limit {
                let exact_dist =
                    match crate::distance::compute_distance_trusted(batch[i], batch[j], metric) {
                        Ok(d) => d,
                        Err(_) => continue,
                    };
                let quant_dist = match quantizer.asymmetric_dist(batch[j], &q_i, metric) {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                diffs.push((exact_dist - quant_dist).abs());
            }
        }

        if diffs.is_empty() {
            return Self::default();
        }

        let count = diffs.len() as u64;
        let mean = diffs.iter().sum::<f32>() / count as f32;
        let variance = diffs.iter().map(|&d| (d - mean).powi(2)).sum::<f32>() / count as f32;

        Self {
            mean_bias: mean,
            variance_bias: variance,
            sample_count: count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantize::ScalarQuantizer;
    use memfuse_core::DistanceMetric;

    #[test]
    fn test_sq8_bias_calibration() {
        let v1 = vec![0.0f32, 0.0, 0.0, 0.0];
        let v2 = vec![1.0f32, 1.0, 1.0, 1.0];
        let v3 = vec![0.5f32, 0.5, 0.5, 0.5];
        let batch = vec![v1.as_slice(), v2.as_slice(), v3.as_slice()];

        let quantizer = ScalarQuantizer::train(&batch, 4);
        let bias = Sq8Bias::calibrate(&batch, &quantizer, DistanceMetric::Euclidean);

        assert!(bias.sample_count > 0);
        assert!(bias.mean_bias >= 0.0);
        assert!(bias.variance_bias >= 0.0);
    }
}
