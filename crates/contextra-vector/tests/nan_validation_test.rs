// FILE-CONTEXT
// ZWECK: Strikte Validierung der NaN/Infinity-Abweisung an der Distanz-API-Grenze.

use contextra_vector::distance::ScalarDistance;
use contextra_vector::distance::contract::{DistanceFn, Metric};

#[test]
fn test_nan_infinity_strict_rejection() {
    let metrics = [Metric::Euclidean, Metric::Cosine, Metric::DotProduct];

    let valid = [1.0, 2.0, 3.0, 4.0];
    let nan_vec = [1.0, f32::NAN, 3.0, 4.0];
    let inf_vec = [1.0, 2.0, f32::INFINITY, 4.0];
    let neg_inf_vec = [f32::NEG_INFINITY, 2.0, 3.0, 4.0];

    for &metric in &metrics {
        let sd = ScalarDistance::new(metric);

        assert!(sd.compute(&valid, &nan_vec).is_err());
        assert!(sd.compute(&nan_vec, &valid).is_err());
        assert!(sd.compute(&valid, &inf_vec).is_err());
        assert!(sd.compute(&inf_vec, &valid).is_err());
        assert!(sd.compute(&valid, &neg_inf_vec).is_err());
        assert!(sd.compute(&neg_inf_vec, &valid).is_err());

        // DistanceFn contract returns NaN on invalid input
        assert!(sd.distance(&valid, &nan_vec).is_nan());
        assert!(sd.distance(&valid, &inf_vec).is_nan());
    }
}
