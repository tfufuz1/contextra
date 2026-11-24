// FILE-CONTEXT
// ZWECK: Gemeinsame Test-Helfer (Vektorgenerierung, Brute-Force-Oracle, Recall, Tempfile).
// INVARIANTEN: Reiner Test-Code, deterministische Seeds (kein thread_rng), stabiler Tie-Break nach DocId.
// HOTSPOTS: 25-140
// STAND: 2026-11-11

//! Gemeinsame Hilfsfunktionen für die Integrationstests von `contextra-vector`.

use contextra_vector::types::{DistanceMetric, DocId};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::Ordering;

// ANCHOR[contextra-vector:W09-COMMON-001]
/// Deterministischer Vektorgenerator mit festem Seed.
pub fn generate_deterministic_vectors(count: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::with_capacity(count);
    for _ in 0..count {
        let mut v = Vec::with_capacity(dim);
        for _ in 0..dim {
            v.push(rng.gen_range(-1.0..1.0));
        }
        vectors.push(v);
    }
    vectors
}

// ANCHOR[contextra-vector:W09-COMMON-002]
/// Normalisiert einen Vektor auf Einheitslänge (für Cosine-Distanz).
pub fn normalize_vector(v: &mut [f32]) {
    let norm_sq: f32 = v.iter().map(|x| x * x).sum();
    let norm = norm_sq.sqrt();
    if norm > 1e-12 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

// ANCHOR[contextra-vector:W09-COMMON-003]
/// Berechnet die exakte Distanz zwischen zwei Vektoren gemäß Distanzmetrik.
pub fn compute_exact_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::Euclidean => {
            let mut sum = 0.0f32;
            for (x, y) in a.iter().zip(b.iter()) {
                let diff = *x - *y;
                sum += diff * diff;
            }
            sum.sqrt()
        }
        DistanceMetric::Cosine => {
            let mut dot = 0.0f32;
            let mut norm_a = 0.0f32;
            let mut norm_b = 0.0f32;
            for (x, y) in a.iter().zip(b.iter()) {
                dot += *x * *y;
                norm_a += *x * *x;
                norm_b += *y * *y;
            }
            let denom = (norm_a * norm_b).sqrt();
            if denom <= 1e-12 {
                1.0
            } else {
                1.0 - (dot / denom).clamp(-1.0, 1.0)
            }
        }
        DistanceMetric::DotProduct => {
            let mut dot = 0.0f32;
            for (x, y) in a.iter().zip(b.iter()) {
                dot += *x * *y;
            }
            -dot
        }
        _ => panic!("Nicht unterstützte Distanzmetrik im Test-Oracle"),
    }
}

// ANCHOR[contextra-vector:W09-COMMON-004]
/// Exakter Brute-Force k-NN Oracle mit deterministischem Tie-Break nach DocId.
pub fn brute_force_knn(
    dataset: &[(DocId, Vec<f32>)],
    query: &[f32],
    k: usize,
    metric: DistanceMetric,
) -> Vec<(DocId, f32)> {
    let mut scored: Vec<(DocId, f32)> = dataset
        .iter()
        .map(|(id, vec)| (*id, compute_exact_distance(query, vec, metric)))
        .collect();

    scored.sort_by(|a, b| {
        a.1.partial_cmp(&b.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    scored.truncate(k);
    scored
}

// ANCHOR[contextra-vector:W09-COMMON-005]
/// Berechnet den Recall@k zwischen den Ground-Truth IDs und den tatsächlichen Resultaten.
pub fn calculate_recall_at_k(ground_truth: &[DocId], actual: &[DocId]) -> f64 {
    if ground_truth.is_empty() {
        return 1.0;
    }
    let gt_set: std::collections::BTreeSet<DocId> = ground_truth.iter().copied().collect();
    let matches = actual.iter().filter(|id| gt_set.contains(id)).count();
    matches as f64 / ground_truth.len() as f64
}
