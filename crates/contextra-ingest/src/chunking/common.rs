//! Gemeinsame Datenstrukturen und Hilfsfunktionen fuer Chunks.
//!
//! # Zweck & Architektur
//! Gehoert zur Segmentierungsschicht (Ring 4) in `contextra-ingest`.
//! Zerlegt die hierarchische `DocumentIr` in semantisch zusammenhaengende,
//! budgetierte Einheiten (Chunks) fuer Embeddings und Volltext-Indexierung.
//!
// FILE-CONTEXT
// ZWECK: Hilfsfunktionen für Token-Zählung, Window-Splitting und Finalisierung von ContextChunk (WP M3).
// INVARIANTEN: Zero-Panic, UTF-8-sicher, Reagiert auf ResourceBudget/Limits.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::CanonicalText;
use crate::chunking::config::ChunkingConfig;
use crate::chunking::meta::build_chunk_metadata;
use crate::chunking::ChunkDraft;
use crate::error::IngestError;
use crate::ir::DocumentIr;
use crate::limits::IngestLimits;
use contextra_engine::chunker::{chunk_text_with_overlap, estimate_tokens};
use contextra_types::{ContextChunk, DocId};
use std::ops::Range;

/// Schätzt die Token-Anzahl eines Textes via engine estimate_tokens.
// ANCHOR[contextra-ingest:CHK-0201]
// fn `token_count` fuer Gemeinsame Datenstrukturen und Hilfsfunktionen fuer Chunks.
pub fn token_count(text: &str) -> usize {
    estimate_tokens(text)
}

/// UTF-8-sicheres Window-Splitting mit Overlap (Span-Mapping).
// ANCHOR[contextra-ingest:CHK-0202]
// fn `window_split` fuer Gemeinsame Datenstrukturen und Hilfsfunktionen fuer Chunks.
pub fn window_split(text: &str, window_chars: usize, overlap_chars: usize) -> Vec<Range<usize>> {
    if text.is_empty() || window_chars == 0 {
        return Vec::new();
    }

    let slices = chunk_text_with_overlap(text, window_chars, overlap_chars);
    let mut spans = Vec::with_capacity(slices.len());
    let base_ptr = text.as_ptr() as usize;

    for slice in slices {
        let slice_ptr = slice.as_ptr() as usize;
        let start = slice_ptr - base_ptr;
        let end = start + slice.len();
        spans.push(start..end);
    }

    spans
}

/// Wandelt ChunkDrafts in fertige `ContextChunk`-Instanzen um. Verwirft leere Drafts
/// und erzwingt das Limit `max_chunks_per_document`.
// ANCHOR[contextra-ingest:CHK-0203]
// fn `finalize` fuer Gemeinsame Datenstrukturen und Hilfsfunktionen fuer Chunks.
pub fn finalize(
    drafts: Vec<ChunkDraft>,
    ir: &DocumentIr,
    _canon: &CanonicalText,
    cfg: &ChunkingConfig,
    doc_id: DocId,
    limits: &IngestLimits,
) -> Result<Vec<ContextChunk>, IngestError> {
    let canonical_hash = ir.canonical_hash();
    let cfg_fingerprint = cfg.fingerprint();
    let strategy = cfg.strategy.unwrap_or_else(|| ir.format.default_strategy());

    // Leere oder Whitespace-only Drafts filtern
    let valid_drafts: Vec<_> = drafts
        .into_iter()
        .filter(|d| !d.text.trim().is_empty())
        .collect();

    let chunk_total_u64 = valid_drafts.len() as u64;
    let max_chunks_u64 = u64::from(limits.max_chunks_per_document);

    if chunk_total_u64 > max_chunks_u64 {
        return Err(IngestError::LimitExceeded {
            kind: crate::error::LimitKind::Chunks,
            limit: max_chunks_u64,
            observed: chunk_total_u64,
        });
    }

    let chunk_total = u32::try_from(valid_drafts.len()).unwrap_or(u32::MAX);
    let mut chunks = Vec::with_capacity(valid_drafts.len());

    for (index, draft) in valid_drafts.into_iter().enumerate() {
        let chunk_index = u32::try_from(index).unwrap_or(u32::MAX);
        let tok_count = token_count(&draft.text);

        let metadata = build_chunk_metadata(
            &canonical_hash,
            &cfg_fingerprint,
            strategy,
            &draft,
            chunk_index,
            chunk_total,
            ir,
        );

        chunks.push(ContextChunk {
            doc_id,
            content: draft.text,
            relevance: 1.0,
            token_count: tok_count,
            metadata: Some(metadata),
            contextual_prefix: None,
            links: Vec::new(),
        });
    }

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_split_spans() {
        let text = "Hello world! This is a test for span splitting.";
        let spans = window_split(text, 15, 5);
        assert!(!spans.is_empty());

        for span in spans {
            let slice = &text[span];
            assert!(!slice.is_empty());
        }
    }
}
