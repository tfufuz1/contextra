#![allow(dead_code)]
//! Gemeinsame Test-Hilfsmittel für die contextra-vector Qualitäts- und Integrationssuite.
//!
//! ANCHOR[contextra-vector:W10-COMMON-001]
//!
//! Beinhaltet deterministische PRNG-Generatoren, Vektorgenerierung, Ground-Truth-Distanzfunktionen
//! und Recall@k-Berechnungen.

use contextra_types::{DistanceMetric, DocId};
use std::collections::HashSet;

/// Deterministischer SplitMix64 PRNG für reproduzierbare Testvektoren.
pub struct TestRng {
    state: u64,
}

impl TestRng {
    /// Erzeugt einen neuen `TestRng` mit dem übergebenen Seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Liefert die nächste pseudozufällige 64-Bit-Ganzzahl.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Liefert eine f32-Zahl im Intervall [0.0, 1.0).
    pub fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 32) as f32) / 4294967296.0
    }

    /// Liefert eine symmetrische f32-Zahl im Intervall [-1.0, 1.0).
    pub fn next_signed_f32(&mut self) -> f32 {
        self.next_f32() * 2.0 - 1.0
    }

    /// Erzeugt einen f32-Vektor der Dimension `dim` skaliert mit `scale`.
    pub fn next_vector(&mut self, dim: usize, scale: f32) -> Vec<f32> {
        (0..dim).map(|_| self.next_signed_f32() * scale).collect()
    }

    /// Erzeugt normalverteilte f32-Zufallswerte mittels Box-Muller-Transformation.
    pub fn next_gaussian_f32(&mut self) -> f32 {
        let u1 = self.next_f32().clamp(1e-7, 1.0);
        let u2 = self.next_f32();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
    }

    /// Erzeugt einen normalverteilten f32-Vektor der Dimension `dim`.
    pub fn next_gaussian_vector(&mut self, dim: usize) -> Vec<f32> {
        (0..dim).map(|_| self.next_gaussian_f32()).collect()
    }
}

/// Unabhängige skalare Referenzimplementierung zur Distanzberechnung für Ground-Truth-Prüfungen.
pub fn reference_distance_scalar(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::Cosine => {
            let mut dot = 0.0f64;
            let mut norm_a = 0.0f64;
            let mut norm_b = 0.0f64;
            for (&x, &y) in a.iter().zip(b.iter()) {
                let x64 = x as f64;
                let y64 = y as f64;
                dot += x64 * y64;
                norm_a += x64 * x64;
                norm_b += y64 * y64;
            }
            if norm_a == 0.0 || norm_b == 0.0 {
                1.0
            } else {
                (1.0 - (dot / (norm_a.sqrt() * norm_b.sqrt()))) as f32
            }
        }
        DistanceMetric::Euclidean => {
            let mut sum = 0.0f64;
            for (&x, &y) in a.iter().zip(b.iter()) {
                let diff = (x - y) as f64;
                sum += diff * diff;
            }
            sum.sqrt() as f32
        }
        DistanceMetric::DotProduct => {
            let mut sum = 0.0f64;
            for (&x, &y) in a.iter().zip(b.iter()) {
                sum += (x as f64) * (y as f64);
            }
            (-sum) as f32
        }
    }
}

/// Exakter Brute-Force k-Nearest-Neighbor-Scan für Ground-Truth-Orakel.
pub fn brute_force_knn(
    query: &[f32],
    vectors: &[Vec<f32>],
    ids: &[DocId],
    k: usize,
    metric: DistanceMetric,
) -> Vec<DocId> {
    let mut scored: Vec<(DocId, f32)> = vectors
        .iter()
        .zip(ids.iter())
        .map(|(v, &id)| (id, reference_distance_scalar(query, v, metric)))
        .collect();

    scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(k).map(|(id, _)| id).collect()
}

/// Berechnet den Recall@k zwischen den Kandidaten und dem Ground-Truth-Ergebnis.
pub fn compute_recall_at_k(actual: &[DocId], ground_truth: &[DocId], k: usize) -> f64 {
    if ground_truth.is_empty() || k == 0 {
        return 0.0;
    }
    let gt_k: HashSet<DocId> = ground_truth.iter().copied().take(k).collect();
    let hits = actual.iter().take(k).filter(|id| gt_k.contains(id)).count();
    hits as f64 / gt_k.len() as f64
}
