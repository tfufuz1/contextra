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

/// Ergebnisdokument mit Relevanz-Score für BM25-Suchen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextHit {
    /// Eindeutige ID des Dokuments.
    pub doc_id: DocId,
    /// Berechneter BM25-Relevanz-Score.
    pub score: f32,
}

/// Maximale Anzahl von Suchtreffern pro Anfrage (SPEC §11.3).
pub const MAX_SEARCH_K: usize = 10_000;
