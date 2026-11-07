//! Eigenständiges Beispiel zur isolierten Nutzung von `contextra-simd`.
//!
//! Zeigt Vektor-Validierung, Distanzberechnung und die Nutzung des injizierbaren Dispatchers.

use contextra_simd::dispatch::{Dispatcher, SimdTarget};
use contextra_simd::{compute_distance, validate_vector};
use contextra_types::DistanceMetric;

fn main() {
    println!("=== contextra-simd standalone building block example ===");

    // 1. Vektorvalidierung (Fail-Closed gegen NaN / Inf)
    let query = [0.1f32, 0.2, 0.3, 0.4, 0.5];
    let candidate = [0.2f32, 0.1, 0.4, 0.3, 0.6];

    if let Err(err) = validate_vector(&query) {
        eprintln!("Query vector validation failed: {err}");
        return;
    }
    println!("Vector validation successful: finite float values confirmed.");

    // 2. High-Level Distanzberechnung
    match compute_distance(&query, &candidate, DistanceMetric::Cosine) {
        Ok(dist) => println!("Cosine distance (auto dispatch): {dist:.6}"),
        Err(err) => eprintln!("Distance calculation failed: {err}"),
    }

    // 3. Injizierbarer Dispatcher (Design by Contract, z. B. erzwungener Skalarpfad für Orakel)
    let scalar_dispatcher = Dispatcher::new(SimdTarget::Scalar);
    let auto_dispatcher = Dispatcher::auto();

    let scalar_dist = scalar_dispatcher
        .euclidean_distance(&query, &candidate)
        .unwrap_or(0.0);
    let auto_dist = auto_dispatcher
        .euclidean_distance(&query, &candidate)
        .unwrap_or(0.0);

    println!("Euclidean distance (Scalar): {scalar_dist:.6}");
    println!("Euclidean distance (Auto SIMD): {auto_dist:.6}");
    println!("Numerical delta: {:.2e}", (scalar_dist - auto_dist).abs());

    println!("=== example completed successfully ===");
}
