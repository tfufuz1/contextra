// FILE-CONTEXT
// ZWECK: Trainingslogik und Min/Max/Scale-Berechnung für ScalarQuantizer (SQ8).
// INVARIANTEN: Dividieren durch 0 geschützt (EPSILON Padding); DIM-Validierung; Outlier-Percentile Clipping.
// HOTSPOTS: try_train_with_percentiles, calculate_vector_drift
// STAND: 2026-11-12

//! # Scalar Quantizer Training Module
//!
//! Berechnet die dimensionalen Minima, Maxima, Skalierungsfaktoren und Drift-Messwerte.

use super::ScalarQuantizer;
use crate::error::{Result, VectorError};
use std::sync::atomic::{AtomicU64, Ordering};

impl ScalarQuantizer {
    /// Versucht das Training eines `ScalarQuantizer` mit Standard-Perzentilen.
    pub fn try_train(batch: &[&[f32]], dimension: usize) -> Result<Self> {
        Self::try_train_with_percentiles(
            batch,
            dimension,
            super::DEFAULT_P_LOW,
            super::DEFAULT_P_HIGH,
        )
    }

    /// Trainiert einen `ScalarQuantizer` unter Berücksichtigung von Quantil-Grenzen.
    // ANCHOR[contextra-vector:SQ8-TRAIN]
    pub fn try_train_with_percentiles(
        batch: &[&[f32]],
        dimension: usize,
        p_low: f32,
        p_high: f32,
    ) -> Result<Self> {
        if dimension == 0 {
            return Err(VectorError::invalid_input(
                "Quantizer dimension must be greater than 0",
            ));
        }

        if !p_low.is_finite()
            || !p_high.is_finite()
            || p_low < 0.0
            || p_high > 1.0
            || p_low >= p_high
        {
            return Err(VectorError::invalid_input(format!(
                "Invalid percentiles: p_low ({p_low}) and p_high ({p_high}) must satisfy 0.0 <= p_low < p_high <= 1.0"
            )));
        }

        for (idx, vec) in batch.iter().enumerate() {
            if vec.len() != dimension {
                return Err(VectorError::invalid_input(format!(
                    "Vector at index {idx} has dimension {}, expected {dimension}",
                    vec.len()
                )));
            }
            for &val in *vec {
                if !val.is_finite() {
                    return Err(VectorError::invalid_input(
                        "training data contains NaN or infinite values",
                    ));
                }
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

    /// Panik-freie Trainingsmethode für Tests und Legacy-Pfade.
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

    /// Trainiert den Quantizer mit Standard-Parametern.
    pub fn train(batch: &[&[f32]], dimension: usize) -> Self {
        Self::train_with_percentiles(
            batch,
            dimension,
            super::DEFAULT_P_LOW,
            super::DEFAULT_P_HIGH,
        )
    }

    /// Berechnet die relative Abweichung eines Vektors vom gelernten Wertebereich.
    // ANCHOR[contextra-vector:SQ8-CALC-DRIFT]
    pub fn calculate_vector_drift(&self, vector: &[f32]) -> f32 {
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

    /// Prüft den Drift und aktualisiert interne Metriken.
    #[deprecated(since = "0.1.0", note = "Use `calculate_vector_drift` instead")]
    pub fn check_drift(&self, vector: &[f32]) -> f32 {
        self.calculate_vector_drift(vector)
    }

    /// Gibt das Verhältnis der Anfragen außerhalb des trainierten Bereichs zurück.
    pub fn drift_ratio(&self) -> f32 {
        let total = self.total_queries.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let out = self.out_of_range_queries.load(Ordering::Relaxed);
        out as f32 / total as f32
    }

    /// Prüft, ob eine Neukalibrierung empfohlen wird.
    pub fn is_rebuild_required(&self, threshold: f32) -> bool {
        let total = self.total_queries.load(Ordering::Relaxed);
        if total < 20 {
            return false;
        }
        self.drift_ratio() >= threshold
    }
}
