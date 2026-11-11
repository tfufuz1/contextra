//! Eigenständiges Beispiel zur isolierten Nutzung von `contextra-simd`.
//!
//! Zeigt zwei Vektoren und drei Metriken (Cosine, Euclidean, DotProduct).

use contextra_simd::{compute_distance, DistanceMetric};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [5.0f32, 6.0, 7.0, 8.0];

    let cos = compute_distance(&a, &b, DistanceMetric::Cosine)?;
    let euc = compute_distance(&a, &b, DistanceMetric::Euclidean)?;
    let dot = compute_distance(&a, &b, DistanceMetric::DotProduct)?;

    println!("Cosine: {cos:.6}, Euclidean: {euc:.6}, Dot: {dot:.6}");
    Ok(())
}
