// FILE-CONTEXT
// ZWECK: Bit-Determinismus-Test für skalare Distanzen (INV-SIMD-DETERMINISM-1).

use contextra_vector::distance::ScalarDistance;
use contextra_vector::distance::contract::{DistanceFn, Metric};

#[test]
fn test_distance_bit_determinism_repetitions() {
    let dims = [4, 16, 64, 128];
    let metrics = [Metric::Euclidean, Metric::Cosine, Metric::DotProduct];

    for &dim in &dims {
        let v1: Vec<f32> = (0..dim).map(|i| ((i + 1) as f32 * 0.13).sin()).collect();
        let v2: Vec<f32> = (0..dim).map(|i| ((i + 3) as f32 * 0.29).cos()).collect();

        for &metric in &metrics {
            let sd = ScalarDistance::new(metric);
            let first_res = sd.distance(&v1, &v2);

            for _ in 0..50 {
                let rep_res = sd.distance(&v1, &v2);
                assert_eq!(
                    first_res.to_bits(),
                    rep_res.to_bits(),
                    "Bit-level determinism mismatch for {:?} at dim {}",
                    metric,
                    dim
                );
            }
        }
    }
}
