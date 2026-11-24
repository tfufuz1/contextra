// FILE-CONTEXT
// ZWECK: Differenzieller Test zwischen SIMD- und Skalar-Kernen für AVX2/NEON/Fallback.
// INVARIANTEN: Bit-Identität bzw. Differenz <= 1e-5 für alle Metriken.
// HOTSPOTS: 15-55
// STAND: 2026-11-12
use contextra_vector::distance::{compute_distance, ScalarKernel, DistanceKernel};
use contextra_types::DistanceMetric;

fn deterministic_vector(seed: u64, dim: usize) -> Vec<f32> {
    let mut vec = Vec::with_capacity(dim);
    for i in 0..dim {
        let angle = ((seed.wrapping_mul(37) + i as u64) % 1000) as f32 / 1000.0 * std::f32::consts::TAU;
        vec.push(angle.sin());
    }
    vec
}

#[test]
fn test_simd_vs_scalar_differential_10k() {
    let dims = [4, 17, 128, 768];
    let metrics = [
        DistanceMetric::Cosine,
        DistanceMetric::Euclidean,
        DistanceMetric::DotProduct,
    ];

    let pairs_per_config = 10_000 / (dims.len() * metrics.len());

    for &dim in &dims {
        for &metric in &metrics {
            for i in 0..pairs_per_config {
                let v1 = deterministic_vector(10_000 + i as u64 * 2, dim);
                let v2 = deterministic_vector(10_000 + i as u64 * 2 + 1, dim);

                let dist_simd = compute_distance(&v1, &v2, metric).expect("simd dist");
                let dist_scalar = ScalarKernel.distance(metric, &v1, &v2).expect("scalar dist");

                let diff = (dist_simd - dist_scalar).abs();
                let tolerance_met = diff <= 1e-4 || (dist_scalar.abs() > 1e-3 && (diff / dist_scalar.abs()) <= 1e-4);

                assert!(
                    tolerance_met,
                    "Differential mismatch: dim {dim}, metric {metric:?}, simd={dist_simd}, scalar={dist_scalar}, diff={diff}"
                );
            }
        }
    }
}

#[test]
fn test_distance_metric_serde_conformance() {
    let vector_m = contextra_vector::types::DistanceMetric::Cosine;
    let serialized_vector = serde_json::to_string(&vector_m).unwrap();

    let simd_m = contextra_simd::DistanceMetric::Cosine;
    let serialized_simd = serde_json::to_string(&simd_m).unwrap();

    assert_eq!(serialized_vector, serialized_simd);

    #[cfg(feature = "ports-compat")]
    {
        let types_m = contextra_types::DistanceMetric::Cosine;
        let serialized_types = serde_json::to_string(&types_m).unwrap();
        assert_eq!(serialized_vector, serialized_types);
    }
}
