//! Suchstrom, Trefferstrukturen und Reciprocal Rank Fusion nach SPEC §5.5, §7.2, D-62, D-124.

use contextra_types::{DocHandle, DocTag};

/// Quelle eines Suchtreffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitSource {
    /// BM25-Volltextindex.
    Text,
    /// Vektorindex.
    Vector,
    /// RRF-Fusion.
    Hybrid,
}

/// Einzelner Treffer in einer Suchseite.
#[derive(Debug, Clone)]
pub struct Hit {
    /// Opaque Dokumentenreferenz.
    pub handle: DocHandle,
    /// Mandantenweiter Dokument-Tag.
    pub doc_tag: DocTag,
    /// Chunk-Index innerhalb des Dokuments.
    pub chunk: u32,
    /// Relevanz- oder Fusions-Score.
    pub score: f32,
    /// Signalquelle.
    pub source: HitSource,
}

/// Seite eines Suchergebnisses mit Erschöpfungsanzeige (D-62).
#[derive(Debug, Clone)]
pub struct SearchPage {
    /// Liste von Treffern.
    pub hits: Vec<Hit>,
    /// Gibt an, ob der Kandidatenpool vollständig erschöpft ist.
    pub exhausted: bool,
}
