//! # Integrationstest: chunker hardening tests
//!
//! **Zweck:** Prüft spezifische Garantien, Transaktionskonsistenz und Fehlertoleranz der Engine.
//!
//! Gehört zur Testsuite von `contextra-engine`.

use contextra_engine::chunker::{ChunkerConfig, MarkdownChunker};
use contextra_types::DocId;
use proptest::prelude::*;

// ANCHOR[contextra-engine:TST06-001]
/// Testblock zur Überprüfung von `test_tilde_fence_with_ignored_heading`.
#[test]
fn test_tilde_fence_with_ignored_heading() {
    let markdown = "~~~rust\n# kein Heading\nfn test() {}\n~~~";
    let chunker = MarkdownChunker::with_defaults();
    let chunks = chunker.chunk(DocId::new(1), markdown);

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].content, markdown);
    let meta = chunks[0].metadata.as_ref().unwrap();
    assert_eq!(meta["breadcrumb"], "");
    assert_eq!(meta["heading_level"], 0);
    assert_eq!(meta["source_line"], 1);
    assert_eq!(meta["source_line_end"], 4);
    assert_eq!(meta["source_span"]["start"], 0);
    assert_eq!(meta["source_span"]["end"], markdown.len());
}

// ANCHOR[contextra-engine:TST06-002]
/// Testblock zur Überprüfung von `test_fence_length_4_nested_backticks`.
#[test]
fn test_fence_length_4_nested_backticks() {
    let markdown = "````markdown\n```rust\nfn inner() {}\n```\n````";
    let chunker = MarkdownChunker::with_defaults();
    let chunks = chunker.chunk(DocId::new(1), markdown);

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].content, markdown);
    let meta = chunks[0].metadata.as_ref().unwrap();
    assert_eq!(meta["source_line_end"], 5);
}

// ANCHOR[contextra-engine:TST06-003]
/// Testblock zur Überprüfung von `test_setext_headings_h1_and_h2`.
#[test]
fn test_setext_headings_h1_and_h2() {
    let markdown = "Setext Heading 1\n================\nParagraph text.\n\nSetext Heading 2\n----------------\nMore text.";
    let config = ChunkerConfig {
        min_tokens: 0,
        ..Default::default()
    };
    let chunker = MarkdownChunker::new(config);
    let chunks = chunker.chunk(DocId::new(1), markdown);

    assert_eq!(chunks.len(), 2);

    assert_eq!(
        chunks[0].content,
        "Setext Heading 1\n================\nParagraph text.\n"
    );
    let m0 = chunks[0].metadata.as_ref().unwrap();
    assert_eq!(m0["breadcrumb"], "# Setext Heading 1");
    assert_eq!(m0["heading_level"], 1);
    assert_eq!(m0["source_line"], 1);
    assert_eq!(m0["source_line_end"], 4);

    assert_eq!(
        chunks[1].content,
        "Setext Heading 2\n----------------\nMore text."
    );
    let m1 = chunks[1].metadata.as_ref().unwrap();
    assert_eq!(m1["breadcrumb"], "# Setext Heading 1 > ## Setext Heading 2");
    assert_eq!(m1["heading_level"], 2);
    assert_eq!(m1["source_line"], 5);
    assert_eq!(m1["source_line_end"], 7);
}

// ANCHOR[contextra-engine:TST06-004]
/// Testblock zur Überprüfung von `test_crlf_document_span_accuracy`.
#[test]
fn test_crlf_document_span_accuracy() {
    let markdown = "Header 1\r\n========\r\nLine 1\r\nLine 2\r\n\r\nHeader 2\r\n--------\r\nLine 3";
    let config = ChunkerConfig {
        min_tokens: 0,
        ..Default::default()
    };
    let chunker = MarkdownChunker::new(config);
    let chunks = chunker.chunk(DocId::new(1), markdown);

    assert_eq!(chunks.len(), 2);

    let m0 = chunks[0].metadata.as_ref().unwrap();
    let start0 = m0["source_span"]["start"].as_u64().unwrap() as usize;
    let end0 = m0["source_span"]["end"].as_u64().unwrap() as usize;
    assert_eq!(start0, 0);
    // Line content ends at "Line 2"
    let slice0 = &markdown[start0..end0];
    assert!(slice0.contains("Header 1"));
    assert!(slice0.contains("Line 2"));

    let m1 = chunks[1].metadata.as_ref().unwrap();
    let start1 = m1["source_span"]["start"].as_u64().unwrap() as usize;
    let end1 = m1["source_span"]["end"].as_u64().unwrap() as usize;
    let slice1 = &markdown[start1..end1];
    assert!(slice1.contains("Header 2"));
    assert!(slice1.contains("Line 3"));
}

// ANCHOR[contextra-engine:TST06-005]
/// Testblock zur Überprüfung von `test_extreme_configs_max_tokens_zero_and_usize_max`.
#[test]
fn test_extreme_configs_max_tokens_zero_and_usize_max() {
    let zero_config = ChunkerConfig {
        max_tokens: 0,
        min_tokens: 0,
        ..Default::default()
    };
    let zero_chunker = MarkdownChunker::new(zero_config);
    let chunks_zero = zero_chunker.chunk(DocId::new(1), "# Title\nLine 1\nLine 2");
    assert!(!chunks_zero.is_empty());

    let max_config = ChunkerConfig {
        max_tokens: usize::MAX,
        min_tokens: 0,
        ..Default::default()
    };
    let max_chunker = MarkdownChunker::new(max_config);
    let chunks_max = max_chunker.chunk(DocId::new(1), "# Title\nLine 1\nLine 2");
    assert_eq!(chunks_max.len(), 1);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn prop_source_span_matches_unsynthesized_content(markdown in "\\PC*") {
        let chunker = MarkdownChunker::with_defaults();
        let chunks = chunker.chunk(DocId::new(1), &markdown);

        for chunk in chunks {
            if let Some(meta) = &chunk.metadata {
                if let (Some(start), Some(end)) = (
                    meta["source_span"]["start"].as_u64(),
                    meta["source_span"]["end"].as_u64()
                ) {
                    let start = start as usize;
                    let end = end as usize;
                    if start <= end && end <= markdown.len() {
                        let original_slice = &markdown[start..end];
                        // For LF-only documents without synthesized table headers or split code fences,
                        // chunk.content joined with \n matches original_slice line content.
                        if !markdown.contains('\r') && !chunk.content.contains("```") && !chunk.content.contains('|') {
                            let orig_words: Vec<&str> = original_slice.split_whitespace().collect();
                            let chunk_words: Vec<&str> = chunk.content.split_whitespace().collect();
                            prop_assert_eq!(orig_words, chunk_words);
                        }
                    }
                }
            }
        }
    }
}
