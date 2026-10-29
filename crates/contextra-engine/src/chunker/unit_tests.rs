// FILE-CONTEXT
// ZWECK: Unittests für den Markdown-Chunker.
// INVARIANTEN: Stellt sicher, dass Grundfunktionen des Chunkers korrekt arbeiten.
// STAND: TS:2026-10-29T00:00:00Z

use super::*;

#[test]
fn test_chunk_text_unicode_german_umlauts() {
    let text = "Äpfel, Öle, Übermut und Straße sind wunderschön!! ".repeat(2);
    let char_count = text.chars().count();
    assert!(
        char_count >= 100,
        "Test string must have at least 100 chars, got {}",
        char_count
    );

    let chunks = chunk_text(&text, 30);
    assert!(!chunks.is_empty());

    for chunk in &chunks {
        assert!(std::str::from_utf8(chunk.as_bytes()).is_ok());
        assert!(chunk.chars().count() <= 30);
    }

    let reassembled = chunks.join("");
    assert_eq!(reassembled, text);
}

#[test]
fn chunker_empty_input() {
    let chunks = MarkdownChunker::with_defaults().chunk(DocId::new(1), "");
    assert!(chunks.is_empty());
}

#[test]
fn chunker_single_chunk_no_split() {
    let text = "Short text";
    let chunks = MarkdownChunker::with_defaults().chunk(DocId::new(1), text);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].content, text);
}

#[test]
fn chunker_respects_max_chunk_size() {
    let text = (0..50)
        .map(|_| "word ".repeat(20))
        .collect::<Vec<_>>()
        .join("\n\n");
    let config = ChunkerConfig {
        max_tokens: 100,
        min_tokens: 10,
        ..Default::default()
    };
    let chunks = MarkdownChunker::new(config).chunk(DocId::new(1), &text);
    let limit = (100.0 * 1.2) as usize;
    for chunk in &chunks {
        assert!(
            chunk.token_count <= limit,
            "Chunk too large: {}",
            chunk.token_count
        );
    }
}

#[test]
fn test_chunk_by_headings() {
    let markdown = "# Title\nSome intro.\n## Section 1\nContent 1\n### Sub 1\nSub content";
    let doc_id = DocId::new(1);

    let config = ChunkerConfig {
        min_tokens: 0,
        ..Default::default()
    };
    let chunker = MarkdownChunker::new(config);
    let chunks = chunker.chunk(doc_id, markdown);

    assert_eq!(chunks.len(), 3);

    let m0 = chunks[0].metadata.as_ref().unwrap();
    assert_eq!(m0["breadcrumb"], "# Title");
    assert_eq!(m0["heading_level"], 1);

    let m1 = chunks[1].metadata.as_ref().unwrap();
    assert_eq!(m1["breadcrumb"], "# Title > ## Section 1");
    assert_eq!(m1["heading_level"], 2);

    let m2 = chunks[2].metadata.as_ref().unwrap();
    assert_eq!(m2["breadcrumb"], "# Title > ## Section 1 > ### Sub 1");
    assert_eq!(m2["heading_level"], 3);
}

#[test]
fn test_merge_small_sections() {
    let config = ChunkerConfig {
        min_tokens: 50,
        ..Default::default()
    };
    let chunker = MarkdownChunker::new(config);
    let markdown = "# S1\na\n# S2\nb\n# S3\nc\n# S4\nd\n# S5\ne";
    let doc_id = DocId::new(2);
    let chunks = chunker.chunk(doc_id, markdown);

    assert_eq!(chunks.len(), 1);

    let m = chunks[0].metadata.as_ref().unwrap();
    assert_eq!(m["breadcrumb"], "# S1");
    assert!(chunks[0].content.contains("e"));
}

#[test]
fn test_no_content_loss() {
    let chunker = MarkdownChunker::with_defaults();
    let markdown = "# Title\n\nSome text\n\n## Subtitle\nMore text.";
    let doc_id = DocId::new(3);
    let chunks = chunker.chunk(doc_id, markdown);

    let concatenated: String = chunks
        .iter()
        .map(|c| c.content.clone())
        .collect::<Vec<_>>()
        .join("\n");

    let orig_words: Vec<_> = markdown.split_whitespace().collect();
    let concat_words: Vec<_> = concatenated.split_whitespace().collect();
    assert_eq!(orig_words, concat_words);
}

#[test]
fn test_real_document() {
    let chunker = MarkdownChunker::with_defaults();
    let doc_id = DocId::new(4);

    let mut markdown = String::new();
    for i in 0..100 {
        markdown.push_str(&format!("## Heading {}\n", i));
        for j in 0..20 {
            markdown.push_str(&format!("This is line {} in paragraph {}.\n", j, i));
        }
    }

    let limit = (chunker.config.max_tokens as f64 * 1.2) as usize;

    let chunks = chunker.chunk(doc_id, &markdown);
    assert!(!chunks.is_empty());
    for chunk in chunks {
        assert!(
            chunk.token_count <= limit,
            "Chunk token count {} exceeds limit {}",
            chunk.token_count,
            limit
        );
    }
}

#[test]
fn test_code_block_within_target_size_preserved_atomically() {
    let code = "```rust\nfn main() {\n    let mut x = 10;\n    x += 5;\n    println!(\"Val: {}\", x);\n}\n```";
    let markdown = format!("# Code Test\n\nSome introductory paragraph.\n\n{}", code);

    let config = ChunkerConfig {
        max_tokens: 500,
        min_tokens: 0,
        ..Default::default()
    };
    let chunker = MarkdownChunker::new(config);
    let chunks = chunker.chunk(DocId::new(10), &markdown);

    let code_chunk = chunks
        .iter()
        .find(|c| c.content.contains("fn main()"))
        .expect("Should find chunk containing code block");

    assert!(
        code_chunk.content.contains("```rust") && code_chunk.content.contains("```"),
        "Code block within max_tokens must be preserved atomically without mid-block splits"
    );
}

#[test]
fn test_table_header_retention_and_row_splitting() {
    let mut table_md = String::from("| ID | Name | Role |\n|---|---|---|\n");
    for i in 0..50 {
        table_md.push_str(&format!("| {} | User_{} | Role_{} |\n", i, i, i));
    }

    let config = ChunkerConfig {
        max_tokens: 30,
        min_tokens: 0,
        ..Default::default()
    };
    let chunker = MarkdownChunker::new(config);
    let chunks = chunker.chunk(DocId::new(20), &table_md);

    assert!(
        chunks.len() > 1,
        "Oversized table should be split into multiple chunks"
    );

    for chunk in &chunks {
        assert!(
            chunk.content.contains("| ID | Name | Role |"),
            "Every sub-chunk of an oversized table must retain the table header row"
        );
        assert!(
            chunk.content.contains("|---|---|---|"),
            "Every sub-chunk of an oversized table must retain the table delimiter row"
        );
    }
}

#[test]
fn test_oversized_paragraph_sliding_window_fallback() {
    let long_para = "Satz mit vielen Informationen und Details. ".repeat(100);
    let config = ChunkerConfig {
        max_tokens: 20,
        min_tokens: 0,
        ..Default::default()
    };
    let chunker = MarkdownChunker::new(config);
    let chunks = chunker.chunk(DocId::new(30), &long_para);

    assert!(
        chunks.len() > 1,
        "Oversized paragraph must be split into multiple chunks"
    );

    let limit = (20.0 * 1.2) as usize;
    for chunk in &chunks {
        assert!(
            chunk.token_count <= limit || chunk.content.len() <= limit * CHARS_PER_TOKEN,
            "Sliding window sub-chunk must respect hard limit"
        );
        assert!(
            std::str::from_utf8(chunk.content.as_bytes()).is_ok(),
            "Sliding window sub-chunk must remain valid UTF-8"
        );
    }
}

#[test]
fn test_chunker_determinism() {
    let markdown = "# Title\n\nParagraph 1 text.\n\n```python\ndef hello():\n    print('world')\n```\n\n| A | B |\n|---|---|\n| 1 | 2 |\n";
    let chunker = MarkdownChunker::with_defaults();

    let run1 = chunker.chunk(DocId::new(40), markdown);
    let run2 = chunker.chunk(DocId::new(40), markdown);

    assert_eq!(run1.len(), run2.len());
    for (c1, c2) in run1.iter().zip(run2.iter()) {
        assert_eq!(c1.content, c2.content);
        assert_eq!(c1.token_count, c2.token_count);
        assert_eq!(c1.metadata, c2.metadata);
        assert_eq!(c1.doc_id, c2.doc_id);
    }
}

#[test]
fn test_chunk_text_zero_chunk_size() {
    assert!(chunk_text("hello world", 0).is_empty());
}

#[test]
fn test_chunk_text_empty_string_returns_empty() {
    assert!(chunk_text("", 10).is_empty());
}

#[test]
fn test_chunk_text_emoji_multibyte_boundary() {
    let text = "🦀🚀🔥⭐🎉";
    let chunks = chunk_text(text, 2);
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], "🦀🚀");
    assert_eq!(chunks[1], "🔥⭐");
    assert_eq!(chunks[2], "🎉");
}

#[test]
fn test_chunk_text_exact_chunk_size_multiple() {
    let text = "abcdefghij";
    let chunks = chunk_text(text, 5);
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0], "abcde");
    assert_eq!(chunks[1], "fghij");
}

#[test]
fn test_chunker_zero_max_tokens_handled_safely() {
    let config = ChunkerConfig {
        max_tokens: 0,
        ..Default::default()
    };
    let chunker = MarkdownChunker::new(config);
    let chunks = chunker.chunk(DocId::new(10), "Line 1\nLine 2");
    assert!(!chunks.is_empty());
}

#[test]
fn test_chunker_single_line_no_headings() {
    let chunker = MarkdownChunker::with_defaults();
    let chunks = chunker.chunk(DocId::new(5), "Plain single line document.");
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].content, "Plain single line document.");
    let meta = chunks[0].metadata.as_ref().unwrap();
    assert_eq!(meta["breadcrumb"], "");
    assert_eq!(meta["heading_level"], 0);
}

#[test]
fn test_chunker_unsplit_heading_level_ignored() {
    let config = ChunkerConfig {
        split_levels: vec![1, 2],
        min_tokens: 0,
        ..Default::default()
    };
    let chunker = MarkdownChunker::new(config);
    let markdown = "# Level 1\nText 1\n### Level 3\nText 3";
    let chunks = chunker.chunk(DocId::new(6), markdown);
    assert_eq!(chunks.len(), 1);
    let meta = chunks[0].metadata.as_ref().unwrap();
    assert_eq!(meta["breadcrumb"], "# Level 1");
}

#[test]
fn chunker_handles_headingless_single_paragraph_document() {
    let line =
        "This is a continuous sentence representing extracted PDF text with single line breaks.\n";
    let text = line.repeat(250);
    assert!(!text.contains("#"));
    assert!(!text.contains("\n\n"));

    let chunker = MarkdownChunker::with_defaults();
    let hard_limit = (chunker.config.max_tokens as f64 * 1.2) as usize;
    let chunks = chunker.chunk(DocId::new(100), &text);

    assert!(
        chunks.len() > 1,
        "Document should be split into multiple chunks, got {}",
        chunks.len()
    );

    for chunk in &chunks {
        assert!(
            chunk.token_count <= hard_limit,
            "Chunk token count {} exceeds hard limit {}",
            chunk.token_count,
            hard_limit
        );
    }
}

#[test]
fn chunk_text_with_overlap_produces_overlapping_windows() {
    let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let window_chars = 10;
    let overlap_chars = 3;

    let windows = chunk_text_with_overlap(text, window_chars, overlap_chars);
    assert!(!windows.is_empty());

    for i in 0..windows.len() - 1 {
        let w1 = windows[i];
        let w2 = windows[i + 1];
        let suffix_w1 = &w1[w1.len() - overlap_chars..];
        let prefix_w2 = &w2[..overlap_chars];
        assert_eq!(
            suffix_w1,
            prefix_w2,
            "Windows {} and {} do not overlap correctly",
            i,
            i + 1
        );
    }

    assert_eq!(
        windows.first().unwrap().chars().take(3).collect::<String>(),
        "ABC"
    );
    assert_eq!(
        windows
            .last()
            .unwrap()
            .chars()
            .rev()
            .take(3)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>(),
        "789"
    );
}

#[test]
fn chunk_text_with_overlap_respects_utf8_boundaries() {
    let text = "Äpfel, Öle, Übermut und Straße sind wunderschön!! ".repeat(5);
    let windows = chunk_text_with_overlap(&text, 25, 5);
    assert!(!windows.is_empty());

    for window in &windows {
        assert!(std::str::from_utf8(window.as_bytes()).is_ok());
        assert!(window.chars().count() <= 25);
    }
}
