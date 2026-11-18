//! # Paritäts- und Korpusanalyse: engine::chunker
//!
//! SPEC-v2: §6.3, S-61, §5.4.
//! Validiert das Chunking von `contextra-engine::chunker::MarkdownChunker` auf dem Korpus.

#![cfg(not(loom))]

use contextra_engine::chunker::{ChunkerConfig, MarkdownChunker};
use contextra_types::DocId;

#[test]
fn test_chunker_parity_korpus_analysis() {
    let engine_chunker = MarkdownChunker::with_defaults();

    let test_corpus = vec![
        "# Header 1\n\nParagraph text.\n\n## Subheader\n\n- item 1\n- item 2\n\n```rust\nfn main() {}\n```\n",
        "# Unicode & Umlauts\n\nGrößenmaßstäbe und Überprüfung von Äpfeln & Ölen.\n\n| Col A | Col B |\n|---|---|\n| 1 | 2 |\n",
        "# Deep Nesting\n\n### H3 Level\n\n#### H4 Level\n\nContent under deep hierarchy.\n",
        "Single line paragraph without headings or markdown decorations.\n",
        "```python\ndef hello():\n    print('world')\n```\n\nTrailing paragraph after fence.\n",
    ];

    for (idx, doc) in test_corpus.iter().enumerate() {
        let chunks = engine_chunker.chunk(DocId::new(idx as u64 + 1), doc);
        assert!(
            !chunks.is_empty(),
            "Chunker must produce chunks for corpus item {}",
            idx
        );

        // contextra-engine liefert unmittelbar Flat-Text Chunks mit Spans in Metadaten.
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
        }
    }
}
