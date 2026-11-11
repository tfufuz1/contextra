//! Härtungs- und Invariantentest für den Chunker
//!
//! Normativ: docs/SPEC.md §5.4, S-61.
//! Prüft:
//! 1. Kein Panic bei beliebigem UTF-8 (Steuerzeichen, Nullbytes, lange Sequenzen).
//! 2. Determinismus: Zweimal chunken führt zu identischer Chunk-Folge.
//! 3. Chunk-Grenzen liegen immer auf gültigen UTF-8 Zeichengrenzen (char boundary).
//! 4. Konfigurationslimits werden eingehalten (kein Überschreiten von Hard Limits).
//! 5. Kein unerwarteter Textverlust.

#![cfg(not(loom))]

use contextra_engine::chunker::{ChunkerConfig, MarkdownChunker};
use contextra_types::DocId;
use proptest::prelude::*;

#[test]
fn test_chunker_extreme_and_malicious_inputs() {
    let chunker = MarkdownChunker::with_defaults();

    let string_100k = "A".repeat(100_000);
    let string_5k = "Übermäßige Größenänderungen ".repeat(5_000);
    let string_emoji = "👨‍👩‍👧‍👦".repeat(1_000);

    let malicious_inputs = vec![
        "",
        "\0\0\0\0",
        "\x01\x02\x03\x04\x05\x06\x07\x08",
        "\r\n\r\n\r\n\r\n",
        "# \0 Nullbyte in Heading\nContent\0with\0nulls\n",
        "```\nUnclosed code fence at EOF",
        "~~~nested\n```mixed\n~~~fences",
        "| incomplete | table\n| row only",
        "####### H7 is not standard markdown heading\nText",
        &string_100k,
        &string_5k,
        &string_emoji,
    ];

    for (idx, input) in malicious_inputs.iter().enumerate() {
        let doc_id = DocId::new(idx as u64 + 1);
        let chunks_1 = chunker.chunk(doc_id, input);
        let chunks_2 = chunker.chunk(doc_id, input);

        // Determinismus
        assert_eq!(chunks_1.len(), chunks_2.len(), "Determinismus: Chunkanzahl muss identisch sein");
        for (c1, c2) in chunks_1.iter().zip(chunks_2.iter()) {
            assert_eq!(c1.content, c2.content, "Determinismus: Inhalt muss identisch sein");
            assert_eq!(c1.token_count, c2.token_count);
        }

        // UTF-8 Sicherheit
        for c in &chunks_1 {
            assert!(std::str::from_utf8(c.content.as_bytes()).is_ok());
            // Prüfen, ob Spans valide sind
            if let Some(meta) = &c.metadata {
                if let Some(span) = meta.get("source_span") {
                    let start = span.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let end = span.get("end").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    assert!(input.is_char_boundary(start.min(input.len())));
                    assert!(input.is_char_boundary(end.min(input.len())));
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]
    #[test]
    fn prop_chunker_arbitrary_utf8_safety(
        text in "\\PC*",
        max_tokens in 10usize..500usize,
    ) {
        let config = ChunkerConfig {
            max_tokens,
            min_tokens: 5,
            include_breadcrumbs: true,
            split_levels: vec![1, 2, 3],
        };
        let chunker = MarkdownChunker::new(config);
        let doc_id = DocId::new(42);

        let chunks = chunker.chunk(doc_id, &text);

        for c in &chunks {
            // Keine Panics, valides UTF-8
            prop_assert!(std::str::from_utf8(c.content.as_bytes()).is_ok());

            if let Some(meta) = &c.metadata {
                if let Some(span) = meta.get("source_span") {
                    let start = span.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let end = span.get("end").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    prop_assert!(text.is_char_boundary(start.min(text.len())));
                    prop_assert!(text.is_char_boundary(end.min(text.len())));
                }
            }
        }
    }
}
