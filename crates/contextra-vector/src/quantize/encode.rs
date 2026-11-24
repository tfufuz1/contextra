// FILE-CONTEXT
// ZWECK: Quantisierung (f32 -> u8) für ScalarQuantizer (SQ8).
// INVARIANTEN: Zero unsafe code (#![forbid(unsafe_code)]); Clamp auf [0, 255].
// HOTSPOTS: quantize
// STAND: 2026-11-12

//! # Scalar Quantizer Encoding Module
//!
//! Bildet Fließkommavektoren auf diskrete 8-Bit Ganzzahlen ab.

use super::ScalarQuantizer;
use crate::error::{Result, VectorError};
use std::sync::atomic::Ordering;

impl ScalarQuantizer {
    /// Quantisiert einen `f32` Vektor in ein `u8` Slice.
    // ANCHOR[contextra-vector:SQ8-QUANTIZE]
    pub fn quantize(&self, vector: &[f32]) -> Result<Vec<u8>> {
        if self.mins.len() < self.dimension
            || self.maxes.len() < self.dimension
            || self.scales.len() < self.dimension
        {
            return Err(VectorError::invalid_input(
                "Quantizer internal state is corrupted or inconsistent with dimension",
            ));
        }

        if vector.len() != self.dimension {
            return Err(VectorError::invalid_input(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.len()
            )));
        }

        let mut is_out = false;
        for (i, &v) in vector.iter().enumerate().take(self.dimension) {
            let min_v =
                self.mins.get(i).copied().ok_or_else(|| {
                    VectorError::invalid_input("Quantizer mins index out of bounds")
                })?;
            let max_v =
                self.maxes.get(i).copied().ok_or_else(|| {
                    VectorError::invalid_input("Quantizer maxes index out of bounds")
                })?;
            if v < min_v || v > max_v {
                is_out = true;
                break;
            }
        }

        let total = self.total_queries.fetch_add(1, Ordering::Relaxed) + 1;
        let out_cnt = if is_out {
            self.out_of_range_queries.fetch_add(1, Ordering::Relaxed) + 1
        } else {
            self.out_of_range_queries.load(Ordering::Relaxed)
        };

        if total >= 20 && (out_cnt as f64 / total as f64) > 0.05 {
            let ratio = (out_cnt as f64 / total as f64) * 100.0;
            tracing::warn!(
                out_of_range_ratio = %format!("{:.1}%", ratio),
                total_queries = total,
                "Over 5% of quantized queries are outside the trained range — ScalarQuantizer recalibration is recommended."
            );
        }

        let mut quantized = Vec::with_capacity(self.dimension);
        for (i, &v) in vector.iter().enumerate().take(self.dimension) {
            let min_v =
                self.mins.get(i).copied().ok_or_else(|| {
                    VectorError::invalid_input("Quantizer mins index out of bounds")
                })?;
            let max_v =
                self.maxes.get(i).copied().ok_or_else(|| {
                    VectorError::invalid_input("Quantizer maxes index out of bounds")
                })?;
            let scale_v = self.scales.get(i).copied().ok_or_else(|| {
                VectorError::invalid_input("Quantizer scales index out of bounds")
            })?;
            let clamped = v.clamp(min_v, max_v);
            let byte_val = ((clamped - min_v) * scale_v).round().clamp(0.0, 255.0) as u8;
            quantized.push(byte_val);
        }

        Ok(quantized)
    }
}
