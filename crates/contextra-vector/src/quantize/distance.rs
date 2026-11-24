// FILE-CONTEXT
// ZWECK: Asymmetrische und symmetrische Distanzberechnung für quantisierte Vektoren.
// INVARIANTEN: Zero unsafe code (#![forbid(unsafe_code)]); Zero allocations via Inline-Dequantisierung.
// HOTSPOTS: asymmetric_dist, symmetric_dist
// STAND: 2026-11-12

//! # Scalar Quantizer Distance Module
//!
//! Berechnet asymmetrische (exakt-zu-quantisiert) und symmetrische (quantisiert-zu-quantisiert) Distanzen.

use super::ScalarQuantizer;
use crate::distance::euclidean_distance_sq_f32_u8;
use crate::error::{Result, VectorError};
use crate::types::DistanceMetric;

impl ScalarQuantizer {
    /// Berechnet die asymmetrische Distanz zwischen einer exakten f32-Query und einem quantisierten u8-Vektor.
    // ANCHOR[contextra-vector:SQ8-ASYMMETRIC-DIST]
    pub fn asymmetric_dist(
        &self,
        query: &[f32],
        quantized: &[u8],
        metric: DistanceMetric,
    ) -> Result<f32> {
        if query.len() != quantized.len() {
            return Err(VectorError::invalid_input("Vector dimensions must match"));
        }

        let acc = match metric {
            DistanceMetric::Cosine => {
                let mut dot = 0.0;
                let mut norm_q_sq = 0.0;
                let mut norm_d_sq = 0.0;

                for i in 0..self.dimension {
                    let qi = query[i];
                    let di = f32::from(quantized[i]) * self.inv_scales[i] + self.mins[i];
                    dot += qi * di;
                    norm_q_sq += qi * qi;
                    norm_d_sq += di * di;
                }

                if norm_q_sq <= 0.0 || norm_d_sq <= 0.0 {
                    1.0
                } else {
                    let sim = dot / (norm_q_sq.sqrt() * norm_d_sq.sqrt());
                    (1.0 - sim).max(0.0)
                }
            }
            DistanceMetric::Euclidean => {
                let dist_sq =
                    euclidean_distance_sq_f32_u8(query, quantized, &self.inv_scales, &self.mins);
                dist_sq.sqrt()
            }
            DistanceMetric::DotProduct => {
                let mut dot = 0.0;
                for i in 0..self.dimension {
                    let qi = query[i];
                    let di = f32::from(quantized[i]) * self.inv_scales[i] + self.mins[i];
                    dot += qi * di;
                }
                -dot
            }
        };
        Ok(acc)
    }

    /// Berechnet die symmetrische Distanz zwischen zwei quantisierten u8-Vektoren.
    // ANCHOR[contextra-vector:SQ8-SYMMETRIC-DIST]
    pub fn symmetric_dist(&self, q1: &[u8], q2: &[u8], metric: DistanceMetric) -> Result<f32> {
        if q1.len() != q2.len() {
            return Err(VectorError::invalid_input("Vector dimensions must match"));
        }

        let mut dot = 0.0_f32;
        let mut norm_a_sq = 0.0_f32;
        let mut norm_b_sq = 0.0_f32;
        let mut dist_sq = 0.0_f32;

        for i in 0..self.dimension {
            let v1 = f32::from(q1[i]) * self.inv_scales[i] + self.mins[i];
            let v2 = f32::from(q2[i]) * self.inv_scales[i] + self.mins[i];
            dot += v1 * v2;
            norm_a_sq += v1 * v1;
            norm_b_sq += v2 * v2;
            dist_sq += (v1 - v2).powi(2);
        }

        let acc = match metric {
            DistanceMetric::Cosine => {
                if norm_a_sq <= 0.0 || norm_b_sq <= 0.0 {
                    1.0
                } else {
                    let sim = dot / (norm_a_sq.sqrt() * norm_b_sq.sqrt());
                    (1.0 - sim).max(0.0)
                }
            }
            DistanceMetric::Euclidean => dist_sq.sqrt(),
            DistanceMetric::DotProduct => -dot,
        };
        Ok(acc)
    }
}
