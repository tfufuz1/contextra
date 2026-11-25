//! Autarke Typen für Dokument-IDs und Chunks im Kontext von Ingestion (B1, B3).
//!
//! # Zweck & Architektur
//! Dieses Modul definiert Standard-Typen für `DocId` und `ContextChunk`,
//! sodass `contextra-ingest` vollkommen autark ohne Kopplung an `contextra-types` agiert.

// FILE-CONTEXT
// ZWECK: Autarke Typen DocId, ContextChunk und Link für contextra-ingest (SPEC v4.0 B1/B3).
// INVARIANTEN: Zero-Panic, deterministische Serialisierung.
// STAND: TS:2026-11-08T00:00:00Z

use serde::{Deserialize, Serialize};
use std::fmt;

/// Eindeutige Kennung eines Dokuments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocId(pub u64);

impl DocId {
    /// Erstellt eine neue `DocId`.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den numerischen Wert zurück.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for DocId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

/// Verweis auf einen anderen Chunk oder ein referenziertes Dokument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextChunkLink {
    /// Typ der Verknüpfung (z. B. "parent", "child", "cross_reference")
    pub kind: String,
    /// Ziel-Identifier
    pub target: String,
}

/// Finalisierter Text-Chunk eines Dokuments für Indexierung und Retrieval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextChunk {
    /// Zugehörige Dokument-ID
    pub doc_id: DocId,
    /// Extrahierter Textinhalt des Chunks
    pub content: String,
    /// Heuristische Relevanzbewertung
    pub relevance: f32,
    /// Geschätzte oder gezählte Token-Anzahl
    pub token_count: usize,
    /// Strukturierte Metadaten des Chunks im JSON-Format
    pub metadata: Option<serde_json::Value>,
    /// Optionaler Kontext-Präfix
    pub contextual_prefix: Option<String>,
    /// Verknüpfungen zu anderen Chunks
    pub links: Vec<ContextChunkLink>,
}
