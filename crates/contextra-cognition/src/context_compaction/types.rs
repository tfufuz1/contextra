//! # Typen für die Kontext-Kompaktierung (`context_compaction/types.rs`)
//!
//! Definiert Datenstrukturen, Strategie-Enums und Repräsentationen
//! für die Reduktion des Token-Verbrauchs in Kontextfenstern.
//!
//! ## Ebene und Einordnung
//! Teil der Kognitionsschicht (Ebene E2). Verwendet Basistypen aus `contextra-types`.
//
// FILE-CONTEXT
// ZWECK: Datenmodelle und Strategien für Token-Budgetierung und Kontext-Kompaktierung.
// INVARIANTEN: Zero-Panic; deterministische Datenstrukturen; exakte Provenienz über source_doc_ids.
// STAND: 2026-11-11

use crate::types_local::{ContextChunk, DocId};

// =========================================================================
// SECTION: Strategie-Definitionen
// =========================================================================

// ANCHOR[contextra-cognition:COMP-TYPE-001]
/// Strategie für Context Compaction.
#[derive(Debug, Clone, PartialEq, Eq)]
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
// !SECTION: Strategie-Definitionen

// =========================================================================
// SECTION: Kompaktierungsergebnis & Status-Token
// =========================================================================

// ANCHOR[contextra-cognition:COMP-TYPE-002]
/// Kompaktierter Kontext für LLM-Übergabe.
#[derive(Debug, Clone)]
pub struct CompactedContext {
    /// Beibehaltene Chunks (innerhalb Budget).
    pub retained_chunks: Vec<ContextChunk>,
    /// Status-Token für kompaktierte Chunks.
    pub status_tokens: Vec<StatusToken>,
    /// Verbrauchte Tokens.
    pub tokens_used: usize,
    /// Ursprüngliche Quell-Dokument-IDs zur Wahrung der Provenienz.
    pub source_doc_ids: Vec<DocId>,
}

// ANCHOR[contextra-cognition:COMP-TYPE-003]
/// Kompakter Stellvertreter für einen oder mehrere kompaktierte Chunks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusToken {
    /// Kompakter Beschreibungstext (z. B. "Tool-Output: DB-Abfrage lieferte 42 Ergebnisse").
    pub summary: String,
    /// Anzahl der ersetzten originalen Tokens.
    pub replaced_tokens: usize,
    /// Referenz auf die ersetzten Chunk-IDs.
    pub replaced_doc_ids: Vec<DocId>,
}
// !SECTION: Kompaktierungsergebnis & Status-Token
