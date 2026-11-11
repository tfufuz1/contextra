// FILE-CONTEXT
// ZWECK: 8-Bit Skalare Quantisierung (SQ8) mit pro-Dimension Min/Max Skalierung.
// INVARIANTEN: Dividieren durch 0 geschützt (EPSILON Padding); Keine Panics bei unpassenden Eingaben; INV-SQ8-ERROR-BOUND-1.
// NICHT-OFFENSICHTLICH: try_train prüft Vektor-Dimensionen vor Rekalibrierung der Min/Max-Grenzen.
// HOTSPOTS: ScalarQuantizer
// STAND: 2026-11-12

//! Scalar Quantization (SQ8) for HNSW Index.
//!
//! # Error Bounds Guarantee (INV-SQ8-ERROR-BOUND-1)
//! For each dimension $i \in [0, D-1]$, given quantization interval $[\text{min}_i, \text{max}_i]$ with
//! range $\Delta_i = \text{max}_i - \text{min}_i$:
//! Rounding into 256 discrete levels ($u8 \in [0, 255]$) guarantees that for any in-distribution value $v_i \in [\text{min}_i, \text{max}_i]$,
//! the reconstruction error $e_i = |v_i - \hat{v}_i|$ satisfies:
//! $$e_i \le \frac{\Delta_i}{2 \cdot 255} = \frac{\Delta_i}{510}$$
//! For normalized vectors ($\|v\|_2 = 1$), the reconstructed Euclidean distance error is strictly bounded.

pub mod decode;
pub mod distance;
pub mod encode;
pub mod train;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU64;

// SECTION Quantisierungs-Konstanten

// ANCHOR[contextra-vector:SQ8-CONSTANTS]
/// Default low percentile for quantization scaling (0.5%).
pub const DEFAULT_P_LOW: f32 = 0.005;
/// Default high percentile for quantization scaling (99.5%).
pub const DEFAULT_P_HIGH: f32 = 0.995;

/// Theoretical maximum quantization error bound factor per dimension: $1 / (2 \cdot 255) = 1 / 510$.
pub const SQ8_MAX_ERROR_FACTOR_PER_DIM: f32 = 1.0 / 510.0;
// !SECTION

// SECTION ScalarQuantizer Struktur

// ANCHOR[contextra-vector:SQ8-QUANTIZER-STRUCT]
/// An 8-bit Scalar Quantizer (SQ8) with per-dimension scaling.
///
/// Quantization reduces the memory footprint of vector storage by 4x.
/// Per-dimension scaling improves recall by adapting to different value ranges.
#[derive(Debug, Serialize, Deserialize)]
pub struct ScalarQuantizer {
    pub(crate) mins: Vec<f32>,
    pub(crate) maxes: Vec<f32>,
    pub(crate) scales: Vec<f32>,
    pub(crate) inv_scales: Vec<f32>,
    pub(crate) dimension: usize,
    #[serde(skip, default)]
    pub(crate) total_queries: AtomicU64,
    #[serde(skip, default)]
    pub(crate) out_of_range_queries: AtomicU64,
}

impl Clone for ScalarQuantizer {
    fn clone(&self) -> Self {
        Self {
            mins: self.mins.clone(),
            maxes: self.maxes.clone(),
            scales: self.scales.clone(),
            inv_scales: self.inv_scales.clone(),
            dimension: self.dimension,
            total_queries: AtomicU64::new(
                self.total_queries
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            out_of_range_queries: AtomicU64::new(
                self.out_of_range_queries
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
        }
    }
}

impl ScalarQuantizer {
    /// Dimensionale Minima.
    pub fn mins(&self) -> &[f32] {
        &self.mins
    }

    /// Dimensionale Maxima.
    pub fn maxes(&self) -> &[f32] {
        &self.maxes
    }

    /// Dimensionale Skalierungsfaktoren.
    pub fn scales(&self) -> &[f32] {
        &self.scales
    }

    /// Dimensionale inverse Skalierungsfaktoren.
    pub fn inv_scales(&self) -> &[f32] {
        &self.inv_scales
    }

    /// Maximaler Rekonstruktionsfehler für eine gegebene Dimension.
    pub fn max_reconstruction_error_per_dim(&self, dim: usize) -> Option<f32> {
        if dim < self.dimension {
            let range = self.maxes[dim] - self.mins[dim];
            Some(range * SQ8_MAX_ERROR_FACTOR_PER_DIM)
        } else {
            None
        }
    }

    /// Vektordimension des Quantizers.
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}
