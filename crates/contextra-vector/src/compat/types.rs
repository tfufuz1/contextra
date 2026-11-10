//! Konvertierungen für Typen zwischen  und  (SPEC-v2 §4.3).

use crate::types::{DistanceMetric, DocId, ScoredDocument, TxId};

impl From<DocId> for contextra_types::DocId {
    fn from(d: DocId) -> Self {
        contextra_types::DocId::new(d.0)
    }
}

impl From<contextra_types::DocId> for DocId {
    fn from(d: contextra_types::DocId) -> Self {
        Self(d.inner())
    }
}

impl From<TxId> for contextra_types::TxId {
    fn from(t: TxId) -> Self {
        contextra_types::TxId(t.0)
    }
}

impl From<contextra_types::TxId> for TxId {
    fn from(t: contextra_types::TxId) -> Self {
        Self(t.0)
    }
}

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
            doc_id: sd.doc_id.into(),
            score: sd.score,
        }
    }
}

impl From<contextra_types::ScoredDocument> for ScoredDocument {
    fn from(sd: contextra_types::ScoredDocument) -> Self {
        ScoredDocument {
            doc_id: sd.doc_id.into(),
            score: sd.score,
            distance: 1.0 - sd.score,
        }
    }
}
