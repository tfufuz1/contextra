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
use contextra_types::{ContextChunk, DocId};
use std::ops::Range;

/// Schätzt die Token-Anzahl eines Textes mit heuristischen Regeln (autark ohne Engine-Kopplung).
// ANCHOR[contextra-ingest:CHK-0201]
// fn `token_count` fuer Gemeinsame Datenstrukturen und Hilfsfunktionen fuer Chunks.
pub fn token_count(text: &str) -> usize {
    estimate_tokens(text)
}

/// Autarke heuristische Token-Schätzung für Fließtext, Codeblöcke und Tabellen.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let mut tokens: f64 = 0.0;
    let mut in_fence = false;
    let code_multiplier = 1.8f64;

    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            tokens += 1.0;
            continue;
        }

        let multiplier = if in_fence { code_multiplier } else { 1.0 };

        let mut char_iter = line.chars().peekable();
        while let Some(c) = char_iter.next() {
            match c {
                '\u{4E00}'..='\u{9FFF}'
                | '\u{3400}'..='\u{4DBF}'
                | '\u{20000}'..='\u{2A6DF}'
                | '\u{3040}'..='\u{309F}'
                | '\u{30A0}'..='\u{30FF}' => {
                    tokens += 1.0 * multiplier;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut word_len = 1usize;
                    while char_iter
                        .peek()
                        .is_some_and(|ch| ch.is_alphanumeric() || *ch == '_')
                    {
                        char_iter.next();
                        word_len += 1;
                    }
                    let word_tokens = if word_len <= 4 {
                        1.0
                    } else {
                        1.0 + (word_len as f64 - 4.0) / 4.0
                    };
                    tokens += word_tokens * multiplier;
                }
                '0'..='9' => {
                    let mut num_len = 1usize;
                    while char_iter.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                        char_iter.next();
                        num_len += 1;
                    }
                    tokens += ((num_len as f64) / 3.0).ceil() * multiplier;
                }
                ' ' | '\t' => {}
                _ => {
                    tokens += 0.25 * multiplier;
                }
            }
        }
        tokens += 0.1;
    }

    (tokens.ceil() as usize).max(1)
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

/// Erzeugt überlappende Text-Chunks mit definierter Fenstergröße und Überlappung (autark ohne Engine).
pub fn chunk_text_with_overlap(text: &str, window_chars: usize, overlap_chars: usize) -> Vec<&str> {
    if text.is_empty() || window_chars == 0 {
        return Vec::new();
    }

    let indices: Vec<usize> = text
        .char_indices()
        .map(|(idx, _)| idx)
        .chain(std::iter::once(text.len()))
        .collect();

    let total_chars = indices.len().saturating_sub(1);
    let overlap = overlap_chars.min(window_chars.saturating_sub(1));
    let step = window_chars.saturating_sub(overlap).max(1);

    let mut chunks = Vec::new();
    let mut start_char = 0;

    while start_char < total_chars {
        let end_char = (start_char.saturating_add(window_chars)).min(total_chars);
        let Some(&start_byte) = indices.get(start_char) else {
            break;
        };
        let Some(&end_byte) = indices.get(end_char) else {
            break;
        };

        if let Some(slice) = text.get(start_byte..end_byte) {
            chunks.push(slice);
        }

        if end_char == total_chars {
            break;
        }

        start_char = start_char.saturating_add(step);
    }

    chunks
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
