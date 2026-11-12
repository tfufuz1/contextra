// FILE-CONTEXT
// ZWECK: Dekodierung (u8 -> f32) für ScalarQuantizer (SQ8).
// INVARIANTEN: Zero unsafe code (#![forbid(unsafe_code)]); Exakte Rekonstruktion mit inv_scales.
// HOTSPOTS: dequantize
// STAND: 2026-11-12

//! # Scalar Quantizer Decoding Module
//!
//! Rekonstruiert Fließkommawerte aus 8-Bit quantisierten Bytes.

use super::ScalarQuantizer;
use crate::error::{Result, VectorError};

impl ScalarQuantizer {
    /// Rekonstruiert einen `u8` Vektor zurück in `f32`.
    // ANCHOR[contextra-vector:SQ8-DEQUANTIZE]
    pub fn dequantize(&self, vector: &[u8]) -> Result<Vec<f32>> {
        if self.inv_scales.len() < self.dimension || self.mins.len() < self.dimension {
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

        let mut dequantized = Vec::with_capacity(self.dimension);
        for (i, &v) in vector.iter().enumerate().take(self.dimension) {
            let inv_scale_v = self.inv_scales.get(i).copied().ok_or_else(|| {
                VectorError::invalid_input("Quantizer inv_scales index out of bounds")
            })?;
            let min_v =
                self.mins.get(i).copied().ok_or_else(|| {
                    VectorError::invalid_input("Quantizer mins index out of bounds")
                })?;
            dequantized.push(f32::from(v) * inv_scale_v + min_v);
        }

        Ok(dequantized)
    }
}
