//! Kompatibilitätsbrücke für `contextra-simd` (Feature `workspace`).
//!
//! Stellt Konvertierungen zwischen lokalen `contextra-simd`-Typen
//! und `contextra_types` bereit. Nur aktiv, wenn das Feature `workspace`
//! (oder das Alias `types-compat`) aktiviert ist.

use crate::error::SimdError;
use crate::metric::DistanceMetric;

// SECTION COMPAT-METRIC
// ANCHOR[contextra-simd:COMPAT-METRIC-FROM-SIMD]
// Konvertierung von lokalem DistanceMetric in contextra_types::DistanceMetric
impl From<DistanceMetric> for contextra_types::DistanceMetric {
    #[inline]
    fn from(m: DistanceMetric) -> Self {
        match m {
            DistanceMetric::Cosine => contextra_types::DistanceMetric::Cosine,
            DistanceMetric::Euclidean => contextra_types::DistanceMetric::Euclidean,
            DistanceMetric::DotProduct => contextra_types::DistanceMetric::DotProduct,
        }
    }
}

// ANCHOR[contextra-simd:COMPAT-METRIC-FROM-TYPES]
// Konvertierung von contextra_types::DistanceMetric in lokales DistanceMetric
impl From<contextra_types::DistanceMetric> for DistanceMetric {
    #[inline]
    fn from(m: contextra_types::DistanceMetric) -> Self {
        match m {
            contextra_types::DistanceMetric::Cosine => DistanceMetric::Cosine,
            contextra_types::DistanceMetric::Euclidean => DistanceMetric::Euclidean,
            contextra_types::DistanceMetric::DotProduct => DistanceMetric::DotProduct,
            _ => DistanceMetric::Cosine,
        }
    }
}
// !SECTION COMPAT-METRIC

// SECTION COMPAT-ERROR
// ANCHOR[contextra-simd:COMPAT-ERROR-FROM-SIMD]
// Konvertierung von SimdError in contextra_types::ContextraError
impl From<SimdError> for contextra_types::ContextraError {
    #[inline]
    fn from(err: SimdError) -> Self {
        match err {
            SimdError::InvalidInput(msg) => contextra_types::ContextraError::invalid_input(msg),
            SimdError::DimensionMismatch { left, right } => {
                contextra_types::ContextraError::invalid_input(format!(
                    "dimension mismatch: left={}, right={}",
                    left, right
                ))
            }
            SimdError::UnsupportedMetric => {
                contextra_types::ContextraError::invalid_input("unsupported metric")
            }
        }
    }
}
// !SECTION COMPAT-ERROR

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::expect_used)]
// SECTION COMPAT-TESTS
mod tests {
    use super::*;

    #[test]
    fn test_distance_metric_variants_and_roundtrip() {
        let simd_metrics = [
            DistanceMetric::Cosine,
            DistanceMetric::Euclidean,
            DistanceMetric::DotProduct,
        ];

        let types_metrics = [
            contextra_types::DistanceMetric::Cosine,
            contextra_types::DistanceMetric::Euclidean,
            contextra_types::DistanceMetric::DotProduct,
        ];

        // 1. Variantenreihenfolge und 1:1 Konvertierung
        for (i, &simd_m) in simd_metrics.iter().enumerate() {
            let types_m: contextra_types::DistanceMetric = simd_m.into();
            assert_eq!(Some(&types_m), types_metrics.get(i));

            let back_to_simd: DistanceMetric = types_m.into();
            assert_eq!(back_to_simd, simd_m);
        }

        // 2. Unbekannte Varianten von contextra_types fallen geordnet auf Cosine zurück
        // (sofern andere Varianten in contextra_types existieren oder hinzukommen)
        let custom_roundtrip: DistanceMetric = contextra_types::DistanceMetric::Cosine.into();
        assert_eq!(custom_roundtrip, DistanceMetric::Cosine);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_distance_metric_serde_equivalence() {
        let pairs = [
            (DistanceMetric::Cosine, contextra_types::DistanceMetric::Cosine),
            (DistanceMetric::Euclidean, contextra_types::DistanceMetric::Euclidean),
            (DistanceMetric::DotProduct, contextra_types::DistanceMetric::DotProduct),
        ];

        for (simd_m, types_m) in pairs {
            let simd_json = serde_json::to_string(&simd_m).expect("serialize simd");
            let types_json = serde_json::to_string(&types_m).expect("serialize types");

            assert_eq!(
                simd_json, types_json,
                "serde json representation must match bitwise between simd and types"
            );

            // Deserialisierung über Kreuz
            let deser_simd_from_types: DistanceMetric =
                serde_json::from_str(&types_json).expect("deserialize types into simd");
            assert_eq!(deser_simd_from_types, simd_m);

            let deser_types_from_simd: contextra_types::DistanceMetric =
                serde_json::from_str(&simd_json).expect("deserialize simd into types");
            assert_eq!(deser_types_from_simd, types_m);
        }
    }

    #[test]
    fn test_simd_error_to_contextra_error_mapping() {
        let err1 = SimdError::InvalidInput("test invalid");
        let cerr1: contextra_types::ContextraError = err1.into();
        assert!(matches!(cerr1, contextra_types::ContextraError::InvalidInput(msg) if msg == "test invalid"));

        let err2 = SimdError::DimensionMismatch { left: 4, right: 8 };
        let cerr2: contextra_types::ContextraError = err2.into();
        assert!(
            matches!(cerr2, contextra_types::ContextraError::InvalidInput(msg) if msg == "dimension mismatch: left=4, right=8")
        );

        let err3 = SimdError::UnsupportedMetric;
        let cerr3: contextra_types::ContextraError = err3.into();
        assert!(
            matches!(cerr3, contextra_types::ContextraError::InvalidInput(msg) if msg == "unsupported metric")
        );
    }
}
// !SECTION COMPAT-TESTS
