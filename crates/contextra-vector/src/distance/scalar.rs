// FILE-CONTEXT
// ZWECK: Skalare Referenzimplementierung aller Distanzmetriken (bitgleich zum SIMD-Orakel).
// INVARIANTEN: Zero unsafe code (#![forbid(unsafe_code)]); Exakt reproduzierbare Akkumulation.
// HOTSPOTS: ScalarKernel, ScalarDistance, cosine_distance, dot_product_distance, euclidean_distance
// STAND: 2026-11-12

//! # Scalar Distance Module
//!
//! Sichere, allokationsfreie skalare Referenzimplementierung des Distanz-Kernels.

use super::kernel::{CosineSimilarityPartsU8, DistanceKernel};
use super::validate::{validate_bytes, validate_slices};
use crate::contract::{DistanceFn, Error as ContractError, Metric as ContractMetric};
use crate::error::{Result, VectorError};
use crate::types::DistanceMetric;

// SECTION Skalare Referenz-Implementierung (ScalarKernel)

// ANCHOR[contextra-vector:DIST-SCALAR-KERNEL]
/// Vollständige skalare Referenz-Implementierung mit sicheren Iteratoren.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScalarKernel;

impl ScalarKernel {
    /// Inhärente Distanzberechnung mit Unterstützung für alle Typen, die `Into<DistanceMetric>` implementieren.
    #[inline]
    pub fn distance<M: Into<DistanceMetric>>(
        &self,
        metric: M,
        a: &[f32],
        b: &[f32],
    ) -> Result<f32> {
        let metric = metric.into();
        validate_slices(a, b)?;
        self.distance_trusted(metric, a, b)
    }

    /// Inhärente Distanzberechnung auf Byte-Slices mit flexibler Metrik.
    #[inline]
    pub fn distance_f32_bytes<M: Into<DistanceMetric>>(
        &self,
        metric: M,
        a: &[f32],
        b_bytes: &[u8],
    ) -> Result<f32> {
        let metric = metric.into();
        validate_bytes(a, b_bytes)?;
        self.distance_f32_bytes_trusted(metric, a, b_bytes)
    }

    /// Inhärente Distanzberechnung auf vorvalidierten Slices.
    #[inline]
    pub fn distance_trusted<M: Into<DistanceMetric>>(
        &self,
        metric: M,
        a: &[f32],
        b: &[f32],
    ) -> Result<f32> {
        let metric = metric.into();
        if a.len() != b.len() {
            return Err(VectorError::invalid_input(format!(
                "Vector dimensions must match: {} != {}",
                a.len(),
                b.len()
            )));
        }
        match metric {
            DistanceMetric::Cosine => self.cosine_distance(a, b),
            DistanceMetric::Euclidean => self.euclidean_distance(a, b),
            DistanceMetric::DotProduct => self.dot_product_distance(a, b),
        }
    }

    /// Inhärente Distanzberechnung auf vorvalidierten Byte-Slices.
    #[inline]
    pub fn distance_f32_bytes_trusted<M: Into<DistanceMetric>>(
        &self,
        metric: M,
        a: &[f32],
        b_bytes: &[u8],
    ) -> Result<f32> {
        let metric = metric.into();
        if b_bytes.len() % 4 != 0 || b_bytes.len() / 4 != a.len() {
            return Err(VectorError::invalid_input(format!(
                "Byte alignment or length mismatch: b_bytes len {}, a len {}",
                b_bytes.len(),
                a.len()
            )));
        }
        match metric {
            DistanceMetric::Cosine => {
                let mut dot = 0.0f32;
                let mut norm_a = 0.0f32;
                let mut norm_b = 0.0f32;

                for (x, chunk) in a.iter().zip(b_bytes.chunks_exact(4)) {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(chunk);
                    let y = f32::from_le_bytes(arr);
                    dot += x * y;
                    norm_a += x * x;
                    norm_b += y * y;
                }

                if norm_a == 0.0 || norm_b == 0.0 {
                    return Ok(1.0);
                }
                let similarity = dot / (norm_a.sqrt() * norm_b.sqrt());
                Ok(1.0 - similarity.clamp(-1.0, 1.0))
            }
            DistanceMetric::Euclidean => {
                let mut sum_sq = 0.0f32;
                for (x, chunk) in a.iter().zip(b_bytes.chunks_exact(4)) {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(chunk);
                    let y = f32::from_le_bytes(arr);
                    let diff = x - y;
                    sum_sq += diff * diff;
                }
                Ok(sum_sq.sqrt())
            }
            DistanceMetric::DotProduct => {
                let mut dot = 0.0f32;
                for (x, chunk) in a.iter().zip(b_bytes.chunks_exact(4)) {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(chunk);
                    let y = f32::from_le_bytes(arr);
                    dot += x * y;
                }
                Ok(-dot)
            }
        }
    }

    /// Kosinus-Distanz zwischen zwei f32-Slices.
    #[inline]
    pub fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(VectorError::invalid_input("Vector dimensions must match"));
        }

        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;

        for (x, y) in a.iter().zip(b.iter()) {
            dot += x * y;
            norm_a += x * x;
            norm_b += y * y;
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(1.0);
        }

        let denom = norm_a.sqrt() * norm_b.sqrt();
        let similarity = dot / denom;
        Ok(1.0 - similarity.clamp(-1.0, 1.0))
    }

    /// Dot-Produkt-Distanz (invertiertes Skalarprodukt) zwischen zwei f32-Slices.
    #[inline]
    pub fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(VectorError::invalid_input("Vector dimensions must match"));
        }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        Ok(-dot)
    }

    /// Euklidische Distanz zwischen zwei f32-Slices.
    #[inline]
    pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(VectorError::invalid_input("Vector dimensions must match"));
        }
        let sum_sq: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum();
        Ok(sum_sq.sqrt())
    }

    /// Quadratische Euklidische Distanz zwischen zwei u8-Slices.
    #[inline]
    pub fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> u32 {
        let mut sum = 0u32;
        let len = a.len().min(b.len());
        for i in 0..len {
            let diff = a[i] as i32 - b[i] as i32;
            sum += (diff * diff) as u32;
        }
        sum
    }

    /// Gemischte f32/u8 quadratische Euklidische Distanz mit Rekonstruktionsparametern.
    #[inline]
    pub fn euclidean_distance_sq_f32_u8(
        &self,
        a: &[f32],
        b: &[u8],
        alphas: &[f32],
        mins: &[f32],
    ) -> f32 {
        let mut sum = 0.0f32;
        let len = a.len().min(b.len()).min(alphas.len()).min(mins.len());
        for i in 0..len {
            let dequant = mins[i] + (b[i] as f32) * alphas[i];
            let diff = a[i] - dequant;
            sum += diff * diff;
        }
        sum
    }

    /// Ganzzahliges Skalarprodukt zwischen zwei u8-Byte-Slices.
    #[inline]
    pub fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> u32 {
        let mut dot = 0u32;
        let len = a.len().min(b.len());
        for i in 0..len {
            dot += (a[i] as u32) * (b[i] as u32);
        }
        dot
    }

    /// Gemischtes f32/u8 Skalarprodukt.
    #[inline]
    pub fn dot_product_f32_u8(&self, a: &[f32], b: &[u8]) -> f32 {
        let mut dot = 0.0f32;
        let len = a.len().min(b.len());
        for i in 0..len {
            dot += a[i] * (b[i] as f32);
        }
        dot
    }

    /// Rohkomponenten für u8 Kosinus-Ähnlichkeit.
    #[inline]
    pub fn cosine_similarity_parts_u8(&self, a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
        let mut dot = 0u32;
        let mut norm_a = 0u32;
        let mut norm_b = 0u32;
        let len = a.len().min(b.len());

        for i in 0..len {
            let val_a = a[i] as u32;
            let val_b = b[i] as u32;
            dot += val_a * val_b;
            norm_a += val_a * val_a;
            norm_b += val_b * val_b;
        }

        CosineSimilarityPartsU8 {
            dot,
            norm_a_sq: norm_a,
            norm_b_sq: norm_b,
        }
    }

    /// Normalisiert einen Vektor in-place auf Einheitslänge.
    #[inline]
    pub fn normalize_inplace(&self, v: &mut [f32]) {
        let norm_sq: f32 = v.iter().map(|x| x * x).sum();
        if norm_sq > 0.0 {
            let inv_norm = 1.0 / norm_sq.sqrt();
            for x in v.iter_mut() {
                *x *= inv_norm;
            }
        }
    }
}

impl DistanceKernel for ScalarKernel {
    #[inline]
    fn distance(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32> {
        self.distance(metric, a, b)
    }

    #[inline]
    fn distance_f32_bytes(&self, metric: DistanceMetric, a: &[f32], b_bytes: &[u8]) -> Result<f32> {
        self.distance_f32_bytes(metric, a, b_bytes)
    }

    #[inline]
    fn distance_trusted(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32> {
        self.distance_trusted(metric, a, b)
    }

    #[inline]
    fn distance_f32_bytes_trusted(
        &self,
        metric: DistanceMetric,
        a: &[f32],
        b_bytes: &[u8],
    ) -> Result<f32> {
        self.distance_f32_bytes_trusted(metric, a, b_bytes)
    }

    #[inline]
    fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        self.cosine_distance(a, b)
    }

    #[inline]
    fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        self.dot_product_distance(a, b)
    }

    #[inline]
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        self.euclidean_distance(a, b)
    }

    #[inline]
    fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> u32 {
        self.euclidean_distance_sq_u8(a, b)
    }

    #[inline]
    fn euclidean_distance_sq_f32_u8(
        &self,
        a: &[f32],
        b: &[u8],
        alphas: &[f32],
        mins: &[f32],
    ) -> f32 {
        self.euclidean_distance_sq_f32_u8(a, b, alphas, mins)
    }

    #[inline]
    fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> u32 {
        self.dot_product_u8(a, b)
    }

    #[inline]
    fn dot_product_f32_u8(&self, a: &[f32], b: &[u8]) -> f32 {
        self.dot_product_f32_u8(a, b)
    }

    #[inline]
    fn cosine_similarity_parts_u8(&self, a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
        self.cosine_similarity_parts_u8(a, b)
    }

    #[inline]
    fn normalize_inplace(&self, v: &mut [f32]) {
        self.normalize_inplace(v);
    }
}

// ANCHOR[contextra-vector:DIST-SCALAR-DISTANCE]
/// Schnittstellenadapter für [`DistanceFn`] über [`ScalarKernel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScalarDistance {
    pub metric: ContractMetric,
}

impl ScalarDistance {
    /// Erzeugt eine neue skalare Distanzfunktion für die angegebene Metrik.
    #[must_use]
    pub const fn new(metric: ContractMetric) -> Self {
        Self { metric }
    }

    /// Berechnet die Distanz zwischen zwei Vektoren und liefert bei ungültigen Eingaben Fehler.
    pub fn compute(&self, a: &[f32], b: &[f32]) -> core::result::Result<f32, ContractError> {
        if a.len() != b.len() {
            return Err(ContractError::InvalidInput(
                "Vektordimensionen stimmen nicht überein",
            ));
        }
        if a.iter().any(|v| !v.is_finite()) || b.iter().any(|v| !v.is_finite()) {
            return Err(ContractError::InvalidInput(
                "Vektor enthält NaN oder Unendlichkeit",
            ));
        }

        let metric = match self.metric {
            ContractMetric::Cosine => DistanceMetric::Cosine,
            ContractMetric::Euclidean => DistanceMetric::Euclidean,
            ContractMetric::DotProduct => DistanceMetric::DotProduct,
        };

        ScalarKernel
            .distance(metric, a, b)
            .map_err(|_| ContractError::InvalidInput("Distanzberechnung fehlgeschlagen"))
    }
}

impl DistanceFn for ScalarDistance {
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        self.compute(a, b).unwrap_or(f32::NAN)
    }
}

// Bitgleiche skalare Referenzfunktionen für Hilfskonstrukte und Tests

/// Skalare Kosinus-Distanz als Referenz.
#[inline]
pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    ScalarKernel.cosine_distance(a, b).unwrap_or(1.0)
}

/// Skalares Dot-Produkt (invertiert) als Referenz.
#[inline]
pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    ScalarKernel.dot_product_distance(a, b).unwrap_or(0.0)
}

/// Skalare Euklidische Distanz als Referenz.
#[inline]
pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    ScalarKernel.euclidean_distance(a, b).unwrap_or(0.0)
}
