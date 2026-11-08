//! Lokale Grundtypen für autarke Volltextsuche (Klasse A / SPEC §4.3).

use serde::{Deserialize, Serialize};

#[cfg(not(feature = "docid-128"))]
/// Standard 64-Bit Dokument-Identifikator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct DocId(pub u64);

#[cfg(feature = "docid-128")]
/// 128-Bit Dokument-Identifikator bei aktiviertem `docid-128` Feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(C, align(16))]
pub struct DocId(pub u128);

impl DocId {
    /// Erzeugt eine neue `DocId`.
    #[cfg(not(feature = "docid-128"))]
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Erzeugt eine neue `DocId` (128-Bit).
    #[cfg(feature = "docid-128")]
    #[inline]
    pub const fn new(id: u128) -> Self {
        Self(id)
    }

    /// Liefert den primitiven Identifikator.
    #[cfg(not(feature = "docid-128"))]
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Liefert den primitiven Identifikator (128-Bit).
    #[cfg(feature = "docid-128")]
    #[inline]
    pub const fn inner(self) -> u128 {
        self.0
    }

    /// Kompatibilitäts-Alias für `inner()`.
    #[cfg(not(feature = "docid-128"))]
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[cfg(not(feature = "docid-128"))]
impl From<u64> for DocId {
    #[inline]
    fn from(id: u64) -> Self {
        Self(id)
    }
}

#[cfg(feature = "docid-128")]
impl From<u128> for DocId {
    #[inline]
    fn from(id: u128) -> Self {
        Self(id)
    }
}

#[cfg(feature = "types-compat")]
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

#[cfg(feature = "types-compat")]
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

/// Ergebnisdokument mit Relevanz-Score für BM25-Suchen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextHit {
    /// Eindeutige ID des Dokuments.
    pub doc_id: DocId,
    /// Berechneter BM25-Relevanz-Score.
    pub score: f32,
}

#[cfg(feature = "types-compat")]
impl From<TextHit> for contextra_types::ScoredDocument {
    fn from(hit: TextHit) -> Self {
        Self {
            doc_id: hit.doc_id.into(),
            score: hit.score,
        }
    }
}

#[cfg(feature = "types-compat")]
impl From<contextra_types::ScoredDocument> for TextHit {
    fn from(doc: contextra_types::ScoredDocument) -> Self {
        Self {
            doc_id: doc.doc_id.into(),
            score: doc.score,
        }
    }
}

/// Maximale Anzahl von Suchtreffern pro Anfrage (SPEC §11.3).
pub const MAX_SEARCH_K: usize = 10_000;
