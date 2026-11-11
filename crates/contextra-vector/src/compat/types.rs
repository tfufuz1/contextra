//! Konvertierungen für Typen zwischen contextra-vector und contextra-types (SPEC-v2 §4.3).

use crate::types::{DistanceMetric, ScoredDocument};

impl From<DistanceMetric> for contextra_types::DistanceMetric {
    fn from(m: DistanceMetric) -> Self {
        match m {
            DistanceMetric::Cosine => contextra_types::DistanceMetric::Cosine,
            DistanceMetric::Euclidean => contextra_types::DistanceMetric::Euclidean,
            DistanceMetric::DotProduct => contextra_types::DistanceMetric::DotProduct,
        }
    }
}

impl From<contextra_types::DistanceMetric> for DistanceMetric {
    fn from(m: contextra_types::DistanceMetric) -> Self {
        match m {
            contextra_types::DistanceMetric::Cosine => DistanceMetric::Cosine,
            contextra_types::DistanceMetric::Euclidean => DistanceMetric::Euclidean,
            contextra_types::DistanceMetric::DotProduct => DistanceMetric::DotProduct,
            _ => DistanceMetric::Cosine,
        }
    }
}

impl From<ScoredDocument> for contextra_types::ScoredDocument {
    fn from(sd: ScoredDocument) -> Self {
        contextra_types::ScoredDocument {
            doc_id: sd.doc_id,
            score: sd.score,
        }
    }
}

impl From<contextra_types::ScoredDocument> for ScoredDocument {
    fn from(sd: contextra_types::ScoredDocument) -> Self {
        ScoredDocument {
            doc_id: sd.doc_id,
            score: sd.score,
            distance: 1.0 - sd.score,
        }
    }
}
