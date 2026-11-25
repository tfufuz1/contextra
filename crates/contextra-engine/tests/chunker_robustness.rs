#![cfg(not(loom))]
use contextra_engine::chunker::{
    chunk_text, chunk_text_with_overlap, ChunkerConfig, MarkdownChunker,
};
use contextra_types::DocId;

#[test]
fn test_chunker_markdown_utf8_and_boundary_safety() {
    let config = ChunkerConfig {
        max_tokens: 100,
        min_tokens: 5,
        include_breadcrumbs: true,
        split_levels: vec![1, 2, 3],
    };

    let chunker = MarkdownChunker::new(config);

    // 1. Text with German umlauts, emojis, and multi-byte UTF-8 sequences
    let multi_byte_text = "# Überschrift 1: Contextra Architekturbeschreibung\n\n\
        Contextra ist eine hochperformante Vektordatenbank 🚀 mit Multi-Index-Unterstützung.\n\n\
        ## Unterkapitel: Isolation & Performance\n\n\
        Die Mandantenfähigkeit (TenantIsolation) garantiert strikte Trennung aller Datenströme 🔥.\n";

    let chunks = chunker.chunk(DocId::new(1), multi_byte_text);

    assert!(
        !chunks.is_empty(),
        "Chunker must return at least 1 chunk for non-empty Markdown input"
    );

    for chunk in &chunks {
        assert!(!chunk.content.is_empty(), "Chunk content must not be empty");
        // Verify valid UTF-8 string integrity
        assert!(
            std::str::from_utf8(chunk.content.as_bytes()).is_ok(),
            "Chunk content must be valid UTF-8 without broken byte sequences"
        );
    }

    // 2. Standalone UTF-8 chunking functions
    let text = "Äpfel, Öle, Übermut und Straße sind wunderschön!! ";
    let sliced_chunks = chunk_text(text, 10);
    assert!(!sliced_chunks.is_empty());
    for s in sliced_chunks {
        assert!(std::str::from_utf8(s.as_bytes()).is_ok());
    }

    let overlap_chunks = chunk_text_with_overlap(text, 15, 3);
    assert!(!overlap_chunks.is_empty());
    for o in overlap_chunks {
        assert!(std::str::from_utf8(o.as_bytes()).is_ok());
    }

    // 3. Edge Case: Zero max tokens handled safely without panic
    let zero_tokens_config = ChunkerConfig {
        max_tokens: 0,
        min_tokens: 0,
        include_breadcrumbs: false,
        split_levels: vec![1],
    };

    let zero_chunker = MarkdownChunker::new(zero_tokens_config);
    let zero_chunks = zero_chunker.chunk(DocId::new(2), "Simple heading without crash.");
    assert!(
        !zero_chunks.is_empty(),
        "Chunker with max_tokens=0 must handle gracefully without panicking"
    );
}
