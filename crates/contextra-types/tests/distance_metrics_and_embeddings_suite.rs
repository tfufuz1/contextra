//! Distance metrics, embeddings, and vector math property test suite for `contextra-types`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_types::*;
use proptest::prelude::*;

#[test]
/// Prüft Kosinus-Distanzwerte für identische (0.0), entgegengesetzte (2.0) und orthogonale (1.0) Vektoren.
fn test_cosine_distance_identical_and_opposite_vectors() {
    let a = vec![1.0, 2.0, 3.0, 4.0];
    let identical = vec![1.0, 2.0, 3.0, 4.0];
    let opposite = vec![-1.0, -2.0, -3.0, -4.0];
    let orthogonal = vec![2.0, -1.0, 0.0, 0.0];

    // Cosine distance of identical vectors is 0.0
    let dist_id = DistanceMetric::Cosine.compute(&a, &identical).unwrap();
    assert!((dist_id - 0.0).abs() < 1e-6);

    // Cosine distance of opposite vectors is 2.0
    let dist_opp = DistanceMetric::Cosine.compute(&a, &opposite).unwrap();
    assert!((dist_opp - 2.0).abs() < 1e-6);

    // Cosine distance of orthogonal vectors is 1.0
    let dist_orth = DistanceMetric::Cosine.compute(&a, &orthogonal).unwrap();
    assert!((dist_orth - 1.0).abs() < 1e-6);
}

#[test]
/// Prüft die euklidische Distanzberechnung und deren mathematische Symmetrie.
fn test_euclidean_distance_properties() {
    let a = vec![0.0, 0.0, 0.0];
    let b = vec![3.0, 4.0, 0.0];

    let dist = DistanceMetric::Euclidean.compute(&a, &b).unwrap();
    assert_eq!(dist, 5.0);

    // Symmetry: dist(a, b) == dist(b, a)
    let dist_rev = DistanceMetric::Euclidean.compute(&b, &a).unwrap();
    assert_eq!(dist, dist_rev);
}

#[test]
/// Prüft die Distanzberechnung per invertiertem Skalarprodukt (-dot).
fn test_dot_product_distance_properties() {
    let a = vec![1.0, 2.0, 3.0];
    let b = vec![4.0, 5.0, 6.0];

    // dot = 1*4 + 2*5 + 3*6 = 32 -> negated: -32.0
    let dist = DistanceMetric::DotProduct.compute(&a, &b).unwrap();
    assert_eq!(dist, -32.0);
}

#[test]
/// Prüft L2-Normierung und Dimensionstreue von Vektor-Embeddings.
fn test_embedding_normalization_invariants() {
    let v = vec![1.0, 1.0, 1.0, 1.0];
    let emb = Embedding::new(v.clone());
    assert_eq!(emb.dim(), 4);
    assert_eq!(emb.as_slice(), &v[..]);

    let norm = emb.l2_norm();
    assert_eq!(norm, 2.0);

    let normalized = emb.normalize();
    assert!((normalized.l2_norm() - 1.0).abs() < 1e-6);
    assert_eq!(normalized.dim(), 4);
}

#[test]
/// Stellt sicher, dass nicht-endliche Werte (NaN, Inf) bei allen Distanzmetriken abgewiesen werden.
fn test_distance_metrics_non_finite_inputs_rejected() {
    let normal = vec![1.0, 2.0];
    let nan_vec = vec![f32::NAN, 2.0];
    let inf_vec = vec![1.0, f32::INFINITY];

    for metric in [
        DistanceMetric::Cosine,
        DistanceMetric::Euclidean,
        DistanceMetric::DotProduct,
    ] {
        assert!(metric.compute(&normal, &nan_vec).is_err());
        assert!(metric.compute(&nan_vec, &normal).is_err());
        assert!(metric.compute(&normal, &inf_vec).is_err());
    }
}

proptest! {
    #[test]
        /// Property-Test: Symmetrie und Nicht-Negativität euklidischer und Kosinus-Distanzen.
    fn prop_distance_metric_symmetry_and_non_negativity(
        vx in proptest::collection::vec(-100.0f32..100.0f32, 16),
        vy in proptest::collection::vec(-100.0f32..100.0f32, 16)
    ) {
        let dist_xy = DistanceMetric::Euclidean.compute(&vx, &vy).unwrap();
        let dist_yx = DistanceMetric::Euclidean.compute(&vy, &vx).unwrap();

        prop_assert!(dist_xy >= 0.0);
        prop_assert!((dist_xy - dist_yx).abs() < 1e-4);

        let cos_xy = DistanceMetric::Cosine.compute(&vx, &vy).unwrap();
        prop_assert!(cos_xy >= 0.0);
        prop_assert!(cos_xy <= 2.0);
    }

    #[test]
        /// Property-Test: Mehrfache L2-Vektornormierung ist idempotent.
    fn prop_embedding_normalize_is_idempotent(
        v in proptest::collection::vec(-50.0f32..50.0f32, 8)
    ) {
        let emb = Embedding::new(v);
        let norm1 = emb.normalize();
        let norm2 = norm1.normalize();

        if emb.l2_norm() >= 1e-12 {
            prop_assert!((norm1.l2_norm() - 1.0).abs() < 1e-4);
            prop_assert!((norm2.l2_norm() - 1.0).abs() < 1e-4);
        }
    }
}
