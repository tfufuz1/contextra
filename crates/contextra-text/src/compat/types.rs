//! Typkonvertierungen zwischen lokalen Typen und `contextra-types`.
// ANCHOR[contextra-text:COMPAT-TYPES-001]

use crate::types::{DocId, TextHit};

impl From<contextra_types::DocId> for DocId {
    #[inline]
    fn from(id: contextra_types::DocId) -> Self {
        #[cfg(not(feature = "docid-128"))]
        {
            Self(id.inner())
        }
        #[cfg(feature = "docid-128")]
        {
            Self(id.inner() as u128)
        }
    }
}

impl From<DocId> for contextra_types::DocId {
    #[inline]
    fn from(id: DocId) -> Self {
        #[cfg(not(feature = "docid-128"))]
        {
            contextra_types::DocId::new(id.inner() as _)
        }
        #[cfg(feature = "docid-128")]
        {
            contextra_types::DocId::new(id.inner() as _)
        }
    }
}

impl From<TextHit> for contextra_types::ScoredDocument {
    fn from(hit: TextHit) -> Self {
        Self {
            doc_id: hit.doc_id.into(),
            score: hit.score,
        }
    }
}

impl From<contextra_types::ScoredDocument> for TextHit {
    fn from(doc: contextra_types::ScoredDocument) -> Self {
        Self {
            doc_id: doc.doc_id.into(),
            score: doc.score,
        }
    }
}
