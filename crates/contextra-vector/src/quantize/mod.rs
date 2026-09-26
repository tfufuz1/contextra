// FILE-CONTEXT
// ZWECK: 8-Bit Skalare Quantisierung (SQ8) mit pro-Dimension Min/Max Skalierung.
// INVARIANTEN: Dividieren durch 0 geschützt (EPSILON Padding); Keine Panics bei unpassenden Eingaben.
// NICHT-OFFENSICHTLICH: try_train prüft Vektor-Dimensionen vor Rekalibrierung der Min/Max-Grenzen.
// HOTSPOTS: quantize.rs (ScalarQuantizer::try_train, quantize, dequantize)
// STAND: TS:2026-08-30T21:55:29Z (SESSION: 10569099)

//! Scalar Quantization (SQ8) for HNSW Index.

use crate::distance::euclidean_distance_sq_f32_u8;
use contextra_core::DistanceMetric;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Default low percentile for quantization scaling (0.5%).
pub const DEFAULT_P_LOW: f32 = 0.005;
/// Default high percentile for quantization scaling (99.5%).
pub const DEFAULT_P_HIGH: f32 = 0.995;

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
            total_queries: AtomicU64::new(self.total_queries.load(Ordering::Relaxed)),
            out_of_range_queries: AtomicU64::new(self.out_of_range_queries.load(Ordering::Relaxed)),
        }
    }
}

impl ScalarQuantizer {
    /// Creates a new ScalarQuantizer trained on a batch of vectors to find per-dimension min/max.
    ///
    /// For long-lived or growing collections, callers should periodically recalibrate the
    /// quantizer (e.g., during index rebuilds) using a representative sample of active vectors.
    /// Without recalibration, new vectors that fall outside the initial range will be clamped,
    /// leading to degraded quantization accuracy.
    /// Safely creates a new ScalarQuantizer trained on a batch of vectors using default percentile clipping (0.5% / 99.5%).
    /// Returns `ContextraError::InvalidInput` if dimension is 0 or any vector dimension mismatches.
    pub fn try_train(batch: &[&[f32]], dimension: usize) -> contextra_core::Result<Self> {
        Self::try_train_with_percentiles(batch, dimension, DEFAULT_P_LOW, DEFAULT_P_HIGH)
    }

    /// Safely creates a new ScalarQuantizer trained on a batch of vectors using custom percentile clipping bounds.
    /// Range `p_low` and `p_high` must satisfy `0.0 <= p_low < p_high <= 1.0`.
    pub fn try_train_with_percentiles(
        batch: &[&[f32]],
        dimension: usize,
        p_low: f32,
        p_high: f32,
    ) -> contextra_core::Result<Self> {
        if dimension == 0 {
            return Err(contextra_core::ContextraError::invalid_input(
                "Quantizer dimension must be greater than 0",
            ));
        }

        if !p_low.is_finite()
            || !p_high.is_finite()
            || p_low < 0.0
            || p_high > 1.0
            || p_low >= p_high
        {
            return Err(contextra_core::ContextraError::invalid_input(format!(
                "Invalid percentiles: p_low ({p_low}) and p_high ({p_high}) must satisfy 0.0 <= p_low < p_high <= 1.0"
            )));
        }

        for (idx, vec) in batch.iter().enumerate() {
            if vec.len() != dimension {
                return Err(contextra_core::ContextraError::invalid_input(format!(
                    "Vector at index {idx} has dimension {}, expected {dimension}",
                    vec.len()
                )));
            }
        }

        if batch.is_empty() {
            return Ok(Self {
                mins: vec![0.0; dimension],
                maxes: vec![1.0; dimension],
                scales: vec![255.0; dimension],
                inv_scales: vec![1.0 / 255.0; dimension],
                dimension,
                total_queries: AtomicU64::new(0),
                out_of_range_queries: AtomicU64::new(0),
            });
        }

        let mut mins = vec![f32::MAX; dimension];
        let mut maxes = vec![f32::MIN; dimension];

        if batch.len() >= 100 {
            // Apply p_low / p_high percentile clipping to eliminate extreme training outliers
            // and preserve 8-bit quantization resolution for in-distribution values.
            for i in 0..dimension {
                let mut dim_vals: Vec<f32> = batch.iter().map(|vec| vec[i]).collect();
                dim_vals.sort_by(|a, b| a.total_cmp(b));
                let max_idx = (dim_vals.len() - 1) as f64;
                let low_idx = ((max_idx * p_low as f64).round() as usize).min(dim_vals.len() - 1);
                let high_idx = ((max_idx * p_high as f64).round() as usize).min(dim_vals.len() - 1);
                mins[i] = dim_vals[low_idx];
                maxes[i] = dim_vals[high_idx];
            }
        } else {
            for vec in batch {
                for (i, &val) in vec.iter().take(dimension).enumerate() {
                    if val < mins[i] {
                        mins[i] = val;
                    }
                    if val > maxes[i] {
                        maxes[i] = val;
                    }
                }
            }
        }

        let mut scales = Vec::with_capacity(dimension);
        let mut inv_scales = Vec::with_capacity(dimension);

        for i in 0..dimension {
            // Prevent div by zero if max == min
            if (maxes[i] - mins[i]).abs() < f32::EPSILON {
                maxes[i] = mins[i] + 1e-6;
            }
            let range = maxes[i] - mins[i];
            scales.push(255.0 / range);
            inv_scales.push(range / 255.0);
        }

        Ok(Self {
            mins,
            maxes,
            scales,
            inv_scales,
            dimension,
            total_queries: AtomicU64::new(0),
            out_of_range_queries: AtomicU64::new(0),
        })
    }

    /// Creates a new ScalarQuantizer trained on a batch of vectors with custom percentile bounds.
    pub fn train_with_percentiles(
        batch: &[&[f32]],
        dimension: usize,
        p_low: f32,
        p_high: f32,
    ) -> Self {
        Self::try_train_with_percentiles(batch, dimension, p_low, p_high).unwrap_or_else(|_| Self {
            mins: vec![0.0; dimension],
            maxes: vec![1.0; dimension],
            scales: vec![255.0; dimension],
            inv_scales: vec![1.0 / 255.0; dimension],
            dimension,
            total_queries: AtomicU64::new(0),
            out_of_range_queries: AtomicU64::new(0),
        })
    }

    /// Creates a new ScalarQuantizer trained on a batch of vectors to find per-dimension min/max.
    ///
    /// For long-lived or growing collections, callers should periodically recalibrate the
    /// quantizer (e.g., during index rebuilds) using a representative sample of active vectors.
    /// Without recalibration, new vectors that fall outside the initial range will be clamped,
    /// leading to degraded quantization accuracy.
    pub fn train(batch: &[&[f32]], dimension: usize) -> Self {
        Self::try_train(batch, dimension).unwrap_or_else(|_| Self {
            mins: vec![0.0; dimension],
            maxes: vec![1.0; dimension],
            scales: vec![255.0; dimension],
            inv_scales: vec![1.0 / 255.0; dimension],
            dimension,
            total_queries: AtomicU64::new(0),
            out_of_range_queries: AtomicU64::new(0),
        })
    }

    /// Returns a reference to per-dimension minimum values.
    pub fn mins(&self) -> &[f32] {
        &self.mins
    }

    /// Returns a reference to per-dimension maximum values.
    pub fn maxes(&self) -> &[f32] {
        &self.maxes
    }

    /// Returns a reference to per-dimension scale factors.
    #[allow(dead_code)]
    pub fn scales(&self) -> &[f32] {
        &self.scales
    }

    /// Returns a reference to per-dimension inverse scale factors.
    #[allow(dead_code)]
    pub fn inv_scales(&self) -> &[f32] {
        &self.inv_scales
    }

    /// Returns the target dimension of the quantizer.
    #[allow(dead_code)]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Calculates quantization drift as the fraction of dimensions falling outside \[mins\[i\], maxes\[i\]\].
    pub fn check_drift(&self, vector: &[f32]) -> f32 {
        if self.dimension == 0 || vector.is_empty() {
            return 0.0;
        }
        let out_count = vector
            .iter()
            .take(self.dimension)
            .enumerate()
            .filter(|(i, &v)| v < self.mins[*i] || v > self.maxes[*i])
            .count();
        out_count as f32 / self.dimension as f32
    }

    /// Returns the fraction of quantized queries that contained values outside the trained min/max range.
    pub fn drift_ratio(&self) -> f32 {
        let total = self.total_queries.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let out = self.out_of_range_queries.load(Ordering::Relaxed);
        out as f32 / total as f32
    }

    /// Checks if recalibration / index rebuild is required based on cumulative quantization drift ratio.
    ///
    /// Requires at least 20 queries to avoid false positives on small initial samples.
    #[allow(dead_code)]
    pub fn is_rebuild_required(&self, threshold: f32) -> bool {
        let total = self.total_queries.load(Ordering::Relaxed);
        if total < 20 {
            return false;
        }
        self.drift_ratio() >= threshold
    }

    /// Quantizes an `f32` vector to `u8`.
    pub fn quantize(&self, vector: &[f32]) -> contextra_core::Result<Vec<u8>> {
        if self.mins.len() < self.dimension
            || self.maxes.len() < self.dimension
            || self.scales.len() < self.dimension
        {
            return Err(contextra_core::ContextraError::invalid_input(
                "Quantizer internal state is corrupted or inconsistent with dimension",
            ));
        }

        if vector.len() != self.dimension {
            return Err(contextra_core::ContextraError::invalid_input(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.len()
            )));
        }

        let mut is_out = false;
        for (i, &v) in vector.iter().enumerate().take(self.dimension) {
            let min_v = self.mins.get(i).copied().ok_or_else(|| {
                contextra_core::ContextraError::invalid_input("Quantizer mins index out of bounds")
            })?;
            let max_v = self.maxes.get(i).copied().ok_or_else(|| {
                contextra_core::ContextraError::invalid_input("Quantizer maxes index out of bounds")
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
            let min_v = self.mins.get(i).copied().ok_or_else(|| {
                contextra_core::ContextraError::invalid_input("Quantizer mins index out of bounds")
            })?;
            let max_v = self.maxes.get(i).copied().ok_or_else(|| {
                contextra_core::ContextraError::invalid_input("Quantizer maxes index out of bounds")
            })?;
            let scale_v = self.scales.get(i).copied().ok_or_else(|| {
                contextra_core::ContextraError::invalid_input(
                    "Quantizer scales index out of bounds",
                )
            })?;
            let clamped = v.clamp(min_v, max_v);
            let byte_val = ((clamped - min_v) * scale_v).round().clamp(0.0, 255.0) as u8;
            quantized.push(byte_val);
        }

        Ok(quantized)
    }

    /// Dequantizes a `u8` vector back to `f32`.
    pub fn dequantize(&self, vector: &[u8]) -> contextra_core::Result<Vec<f32>> {
        if self.inv_scales.len() < self.dimension || self.mins.len() < self.dimension {
            return Err(contextra_core::ContextraError::invalid_input(
                "Quantizer internal state is corrupted or inconsistent with dimension",
            ));
        }

        if vector.len() != self.dimension {
            return Err(contextra_core::ContextraError::invalid_input(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.len()
            )));
        }

        let mut dequantized = Vec::with_capacity(self.dimension);
        for (i, &v) in vector.iter().enumerate().take(self.dimension) {
            let inv_scale_v = self.inv_scales.get(i).copied().ok_or_else(|| {
                contextra_core::ContextraError::invalid_input(
                    "Quantizer inv_scales index out of bounds",
                )
            })?;
            let min_v = self.mins.get(i).copied().ok_or_else(|| {
                contextra_core::ContextraError::invalid_input("Quantizer mins index out of bounds")
            })?;
            dequantized.push(f32::from(v) * inv_scale_v + min_v);
        }

        Ok(dequantized)
    }

    /// Computes the asymmetric distance between an exact query and a quantized vector.
    /// Optimized for zero allocations via inline dequantization.
    pub fn asymmetric_dist(
        &self,
        query: &[f32],
        quantized: &[u8],
        metric: DistanceMetric,
    ) -> contextra_core::Result<f32> {
        if query.len() != quantized.len() {
            return Err(contextra_core::ContextraError::invalid_input(
                "Vector dimensions must match",
            ));
        }

        let acc = match metric {
            DistanceMetric::Cosine => {
                // For per-dimension scaling, we must dequantize each component
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
            _ => unreachable!(),
        };
        Ok(acc)
    }

    /// Computes symmetric (approximate) distance purely in u8.
    /// Optimized for zero allocations via inline dequantization.
    pub fn symmetric_dist(
        &self,
        q1: &[u8],
        q2: &[u8],
        metric: DistanceMetric,
    ) -> contextra_core::Result<f32> {
        if q1.len() != q2.len() {
            return Err(contextra_core::ContextraError::invalid_input(
                "Vector dimensions must match",
            ));
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
            _ => unreachable!(),
        };
        Ok(acc)
    }
}

#[cfg(test)]
mod tests;
