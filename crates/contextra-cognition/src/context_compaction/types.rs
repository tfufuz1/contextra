//! # Typen für die Kontext-Kompaktierung (`context_compaction/types.rs`)
//!
//! Definiert Datenstrukturen, Strategie-Enums und Repräsentationen
//! für die Reduktion des Token-Verbrauchs in Kontextfenstern.
//!
//! ## Ring-Zugehörigkeit
//! Teil der Kognitionsschicht (Ring 3). Verwendet Basistypen aus `contextra-types`.

use contextra_types::{ContextChunk, DocId};

/// Strategie für Context Compaction.
#[derive(Debug, Clone, PartialEq, Eq)]
// ANCHOR[contextra-cognition:COMP-TYPE-001]
/// Strategie für die Kontext-Kompaktierung.
pub enum CompactionStrategy {
    /// Einfachste Strategie: Chunks über Token-Limit werden weggelassen.
    Truncate,
    /// Summarisierung via LLM (erfordert externen Summarizer-Trait).
    Summarize,
    /// Ersetze Tool-Outputs durch kompakte Status-Token.
    StatusToken,
    /// LLM-Zusammenfassung veralteter Chunks mit konfigurierbarem Batch-Limit.
    LlmSummarize {
        /// Maximale Anzahl von Chunks, die pro LLM-Aufruf zusammengefasst werden.
        max_input_chunks: usize,
    },
}

/// Kompaktierter Kontext für LLM-Übergabe.
#[derive(Debug, Clone)]
// ANCHOR[contextra-cognition:COMP-TYPE-002]
/// Ergebnis eines Kompaktierungslaufs.
pub struct CompactedContext {
    /// Beibehaltene Chunks (innerhalb Budget).
    pub retained_chunks: Vec<ContextChunk>,
    /// Status-Token für kompaktierte Chunks.
    pub status_tokens: Vec<StatusToken>,
    /// Verbrauchte Tokens.
    pub tokens_used: usize,
    /// Ursprüngliche Quell-Dokument-IDs.
    pub source_doc_ids: Vec<DocId>,
}

/// Kompakter Stellvertreter für einen oder mehrere kompaktierte Chunks.
#[derive(Debug, Clone, PartialEq, Eq)]
// ANCHOR[contextra-cognition:COMP-TYPE-003]
/// Status-Token als kompakter Stellvertreter für komprimierte Chunks.
pub struct StatusToken {
    /// Kompakter Beschreibungstext (z. B. "Tool-Output: DB-Abfrage lieferte 42 Ergebnisse").
    pub summary: String,
    /// Anzahl der ersetzten originalen Tokens.
    pub replaced_tokens: usize,
    /// Referenz auf die ersetzten Chunk-IDs.
    pub replaced_doc_ids: Vec<DocId>,
}
