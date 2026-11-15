//! Konformitätstest für `DistanceMetric` (R4-Ausnahme).
//! Prüft identische Varianten-Reihenfolge und serde-Serialisierung zwischen
//! `acta_vec::types::DistanceMetric` und `acta_base::DistanceMetric`.
// FILE-CONTEXT
// ZWECK: Konformitätstest der Distanzmetriken über Ports- und Typgrenzen hinweg.
// INVARIANTEN: Identische Distanzergebnisse unabhängig vom Interface-Pfad.
// HOTSPOTS: 15-35
// STAND: 2026-11-12

#![cfg(feature = "ports-compat")]

use acta_vec::types::DistanceMetric as LocalMetric;
use acta_base::DistanceMetric as TypesMetric; // TODO(acta-hr3): Hub-Import (ex types/ports) nach HR-2/HR-3 aufloesen

#[test]
fn test_distance_metric_conformance() {
    let local_variants = [
        LocalMetric::Cosine,
        LocalMetric::Euclidean,
        LocalMetric::DotProduct,
    ];

    let types_variants = [
        TypesMetric::Cosine,
        TypesMetric::Euclidean,
        TypesMetric::DotProduct,
    ];

    for (l, t) in local_variants.iter().zip(types_variants.iter()) {
        let l_json = serde_json::to_string(l).expect("serialize local");
        let t_json = serde_json::to_string(t).expect("serialize types");
        assert_eq!(l_json, t_json, "serde representation must match exactly");

        let converted_to_types: TypesMetric = (*l).into();
        assert_eq!(&converted_to_types, t, "conversion to types failed");

        let converted_to_local: LocalMetric = (*t).into();
        assert_eq!(&converted_to_local, l, "conversion to local failed");
    }
}
