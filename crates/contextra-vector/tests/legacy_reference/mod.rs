// FILE-CONTEXT
// HERKUNFT: Stand commit f92caabaa (crates/contextra-vector/src/distance.rs)
// ZWECK: Unveränderte Legacy-Referenzimplementierung für skalare Vektordistanzen.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyDistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

pub fn legacy_cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let norm = (norm_a * norm_b).sqrt();
    let sim = dot / norm;
    let sim_clamped = sim.clamp(-1.0, 1.0);
    1.0 - sim_clamped
}

pub fn legacy_euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        let diff = x - y;
        sum += diff * diff;
    }
    sum.sqrt()
}

pub fn legacy_dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        sum += x * y;
    }
    -sum
}

pub fn legacy_compute_distance(a: &[f32], b: &[f32], metric: LegacyDistanceMetric) -> Result<f32, String> {
    if a.len() != b.len() {
        return Err(format!("Dimension mismatch: {} vs {}", a.len(), b.len()));
    }
    for &val in a.iter().chain(b.iter()) {
        if !val.is_finite() {
            return Err("Vector contains non-finite values".into());
        }
    }
    match metric {
        LegacyDistanceMetric::Cosine => Ok(legacy_cosine_distance_scalar(a, b)),
        LegacyDistanceMetric::Euclidean => Ok(legacy_euclidean_distance_scalar(a, b)),
        LegacyDistanceMetric::DotProduct => Ok(legacy_dot_product_scalar(a, b)),
    }
}
