use crate::error::{ContextraError, Result};
use crate::types::domain::DocId;
use serde::{Deserialize, Serialize};

/// Distance metric for vector comparison.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[non_exhaustive]
pub enum DistanceMetric {
    /// Cosine distance (`1.0 - cos(angle)`).
    #[default]
    Cosine,
    /// Euclidean (L2) distance.
    Euclidean,
    /// Dot product distance (negated inner product for f32).
    DotProduct,
}

impl DistanceMetric {
    /// Computes the distance between two f32 vectors using this metric.
    pub fn compute(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(ContextraError::invalid_input(
                "Vector dimensions must match",
            ));
        }

        for val in a.iter().chain(b.iter()) {
            if !val.is_finite() {
                return Err(ContextraError::InvalidInput(
                    "Input vector contains non-finite values (NaN or Inf)".into(),
                ));
            }
        }

        let dist = match self {
            Self::Cosine => {
                let mut dot = 0.0;
                let mut norm_a = 0.0;
                let mut norm_b = 0.0;
                for (x, y) in a.iter().zip(b.iter()) {
                    dot += x * y;
                    norm_a += x * x;
                    norm_b += y * y;
                }
                if norm_a == 0.0 || norm_b == 0.0 {
                    1.0
                } else {
                    // Floating-point rounding errors on nearly parallel or identical vectors can cause dot / (norm_a.sqrt() * norm_b.sqrt())
                    // to slightly exceed 1.0, producing negative distances.
                    // Cosine distance is mathematically restricted to [0.0, 2.0] as cosine similarity ∈ [-1.0, 1.0].
                    let dist = 1.0 - (dot / (norm_a.sqrt() * norm_b.sqrt()));
                    dist.clamp(0.0, 2.0)
                }
            }
            Self::Euclidean => {
                let mut sum = 0.0;
                for (x, y) in a.iter().zip(b.iter()) {
                    let diff = x - y;
                    sum += diff * diff;
                }
                sum.max(0.0).sqrt()
            }
            Self::DotProduct => {
                let mut dot = 0.0;
                for (x, y) in a.iter().zip(b.iter()) {
                    dot += x * y;
                }
                -dot // Negative dot product for distance
            }
        };

        if !dist.is_finite() {
            return Err(ContextraError::InvalidInput(
                "Distance computation resulted in a non-finite value (NaN or Inf)".into(),
            ));
        }

        Ok(dist)
    }

    /// Computes the distance between two u8 vectors using this metric.
    ///
    /// Returns a `u32` distance value where smaller values indicate higher similarity / smaller distance
    /// for ALL metric variants.
    ///
    /// # Metric Semantics & Precision
    /// - **Cosine**: Scaled by `1_000_000` for fixed-point ranking (`[0, 2_000_000]`).
    /// - **Euclidean**: Computes `sqrt(Σ diff²)` in `f64` precision before rounding to nearest integer (`round()`)
    ///   and casting/saturating to `u32`. This matches `compute()`'s f32 square-root behavior.
    /// - **DotProduct**: Inverts the sign convention using `u32::MAX - dot` (saturated at `u32::MAX`), matching
    ///   `compute()`'s `-dot` convention so that smaller return values consistently mean higher similarity.
    ///
    /// # Cross-Crate Caller Dependency Notice
    /// Callers in `contextra-index` (such as `hnsw.rs` or `quantize.rs`) expecting raw squared Euclidean distance or
    /// non-inverted raw dot products must account for this unified "smaller = closer" distance metric semantics.
    // DECISION-REF: AGT-CORE-001 — Overflow-Schutz in compute_u8() bereits implementiert.
    // KONTEXT: Alle drei Zweige akkumulieren in u64 (Euclidean: diff²-Summe, DotProduct: Produkt-Summe)
    //          bzw. f64 (Cosine) und sättigen per .min(u32::MAX as u64). Kein Overflow möglich.
    //          Regressionstest: test_distance_metrics_u8_overflow (100_000 Elemente à 255).
    // ID: AGT-CORE-001
    pub fn compute_u8(&self, a: &[u8], b: &[u8]) -> Result<u32> {
        if a.len() != b.len() {
            return Err(ContextraError::invalid_input(
                "Vector dimensions must match",
            ));
        }

        match self {
            Self::Cosine => {
                // FIND-COR-002: Correct cosine distance using f64 arithmetic
                // cos_dist = 1.0 - (dot(a,b) / (||a|| * ||b||))
                let mut dot = 0f64;
                let mut norm_a = 0f64;
                let mut norm_b = 0f64;
                for (&x, &y) in a.iter().zip(b.iter()) {
                    let xf = x as f64;
                    let yf = y as f64;
                    dot += xf * yf;
                    norm_a += xf * xf;
                    norm_b += yf * yf;
                }
                let denom = norm_a.sqrt() * norm_b.sqrt();
                let dist = if denom == 0.0 { 1.0 } else { 1.0 - dot / denom };
                // Scale to u32 fixed-point (×1_000_000) for ranking
                Ok((dist.clamp(0.0, 2.0) * 1_000_000.0) as u32)
            }
            Self::Euclidean => {
                let mut sum = 0u64;
                for (&x, &y) in a.iter().zip(b.iter()) {
                    let diff = (x as i64) - (y as i64);
                    sum += (diff * diff) as u64;
                }
                let dist_f64 = (sum as f64).sqrt();
                let dist_rounded = dist_f64.round();
                Ok(dist_rounded.min(u32::MAX as f64) as u32)
            }
            Self::DotProduct => {
                // Inverted dot product (u32::MAX - dot) so smaller distance = higher similarity.
                let mut dot = 0u64;
                for (&x, &y) in a.iter().zip(b.iter()) {
                    dot += (x as u64) * (y as u64);
                }
                let dot_clamped = dot.min(u32::MAX as u64) as u32;
                Ok(u32::MAX - dot_clamped)
            }
        }
    }
}

/// Vector embedding representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    /// Raw vector data elements.
    pub data: Vec<f32>,
}

impl Embedding {
    /// Creates a new vector embedding from `f32` slice or vector.
    pub fn new(data: Vec<f32>) -> Self {
        Self { data }
    }

    /// Returns the dimension (length) of the embedding vector.
    #[inline]
    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// Returns the vector data as an `f32` slice.
    #[inline]
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    /// Computes the Euclidean L2 norm of the vector.
    pub fn l2_norm(&self) -> f32 {
        self.data.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Returns an L2-unit-normalized clone of this embedding.
    ///
    /// Guarded against zero and subnormal/near-zero vector norms (`norm < 1e-12`).
    /// Vectors with `norm < 1e-12` are returned unchanged to avoid division by subnormals
    /// resulting in `Inf` or `NaN`.
    pub fn normalize(&self) -> Self {
        let norm = self.l2_norm();
        // Threshold 1e-12 is chosen to safely catch subnormal or near-zero floats
        // across high-dimensional embeddings while preventing Inf/NaN after division.
        if norm < 1e-12 {
            return self.clone();
        }
        Self::new(self.data.iter().map(|x| x / norm).collect())
    }
}

/// A scored search result.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScoredDocument {
    /// Identifier of the scored document.
    pub doc_id: DocId,
    /// Relevance or similarity score.
    pub score: f32,
}

impl ScoredDocument {
    /// Creates a new `ScoredDocument` with document ID and score.
    pub fn new(doc_id: DocId, score: f32) -> Self {
        Self { doc_id, score }
    }
}
