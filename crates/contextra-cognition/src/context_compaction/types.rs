use contextra_core::{ContextChunk, DocId};

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

/// Kompaktierter Kontext für LLM-Übergabe.
#[derive(Debug, Clone)]
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
pub struct StatusToken {
    /// Kompakter Beschreibungstext (z. B. "Tool-Output: DB-Abfrage lieferte 42 Ergebnisse").
    pub summary: String,
    /// Anzahl der ersetzten originalen Tokens.
    pub replaced_tokens: usize,
    /// Referenz auf die ersetzten Chunk-IDs.
    pub replaced_doc_ids: Vec<DocId>,
}
