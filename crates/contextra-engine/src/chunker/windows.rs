//! # Modul: windows.rs
//!
//! **Zweck:** Gleitende Fenster (Sliding Windows) für überlappende Chunks.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, deterministische Textverarbeitung.

// FILE-CONTEXT
// ZWECK: Hilfsfunktionen für Zeichen-basiertes und Überlappendes Chunken.
// INVARIANTEN: Beachtet strikt UTF-8-Codepoint-Grenzen; schneidet nie in Unicode-Zeichen. Indexfrei & Panikfrei.
// NICHT-OFFENSICHTLICH: chunk_text_with_overlap erzeugt Gleitfenster für übergroße Absätze.
// STAND: TS:2026-10-29T00:00:00Z

// ANCHOR[contextra-engine:WIN-001]
/// Erzeugt überlappende Text-Chunks mit definierter Fenstergröße und Überlappung.
pub fn chunk_text_with_overlap<'a>(
    text: &'a str,
    window_chars: usize,
    overlap_chars: usize,
) -> Vec<&'a str> {
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

// ANCHOR[contextra-engine:WIN-002]
/// Segmentiert Text in Abschnitte bis zur Maximallänge ohne Überlappung.
pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<&str> {
    if text.is_empty() || chunk_size == 0 {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut char_count = 0;
    let mut start_byte = 0;

    for (byte_idx, _ch) in text.char_indices() {
        if char_count == chunk_size {
            if let Some(slice) = text.get(start_byte..byte_idx) {
                chunks.push(slice);
            }
            start_byte = byte_idx;
            char_count = 0;
        }
        char_count = char_count.saturating_add(1);
    }

    if start_byte < text.len() {
        if let Some(slice) = text.get(start_byte..) {
            chunks.push(slice);
        }
    }

    chunks
}
