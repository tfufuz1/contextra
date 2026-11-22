use contextra_engine::chunker::{ChunkerConfig, MarkdownChunker};
use contextra_types::DocId;
use serde_json::{json, Value};

fn assert_golden(case_name: &str, chunker: &MarkdownChunker, doc: &str, expected_json: Value) {
    let chunks = chunker.chunk(DocId::new(1), doc);
    let serialized: Vec<Value> = chunks
        .into_iter()
        .map(|c| {
            json!({
                "content": c.content,
                "metadata": c.metadata,
            })
        })
        .collect();

    let actual_json = Value::Array(serialized);
    assert_eq!(
        actual_json,
        expected_json,
        "Golden snapshot mismatch for case '{}'\nActual: {}\nExpected: {}",
        case_name,
        serde_json::to_string_pretty(&actual_json).unwrap_or_default(),
        serde_json::to_string_pretty(&expected_json).unwrap_or_default()
    );
}

#[test]
fn test_golden_cases_01_to_27() {
    let default_chunker = MarkdownChunker::with_defaults();

    // 1. Empty input
    assert_golden("01_empty", &default_chunker, "", json!([]));

    // 2. Single short paragraph without headings
    let doc2 = "Hello world from simple paragraph.";
    assert_golden(
        "02_single_paragraph",
        &default_chunker,
        doc2,
        json!([
            {
                "content": "Hello world from simple paragraph.",
                "metadata": {
                    "breadcrumb": "",
                    "heading_level": 0,
                    "source_line": 1,
                    "source_line_end": 1,
                    "source_span": {
                        "start": 0,
                        "end": doc2.len()
                    }
                }
            }
        ]),
    );

    // 3. Simple H1 document
    let doc3 = "# Title\nThis is a paragraph under title.";
    assert_golden(
        "03_h1_paragraph",
        &default_chunker,
        doc3,
        json!([
            {
                "content": "# Title\nThis is a paragraph under title.",
                "metadata": {
                    "breadcrumb": "# Title",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 2,
                    "source_span": {
                        "start": 0,
                        "end": doc3.len()
                    }
                }
            }
        ]),
    );

    // 4. Multi-level headings with default min_tokens=50 (merged)
    let doc4 = "# H1\nText 1\n## H2\nText 2\n### H3\nText 3";
    assert_golden(
        "04_multilevel_headings_merged",
        &default_chunker,
        doc4,
        json!([
            {
                "content": "# H1\nText 1\n\n## H2\nText 2\n\n### H3\nText 3",
                "metadata": {
                    "breadcrumb": "# H1",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 6,
                    "source_span": {
                        "start": 0,
                        "end": doc4.len()
                    }
                }
            }
        ]),
    );

    // 4b. Multi-level headings with min_tokens=0 (unmerged)
    let min0_chunker = MarkdownChunker::new(ChunkerConfig {
        min_tokens: 0,
        ..Default::default()
    });
    assert_golden(
        "04b_multilevel_headings_split",
        &min0_chunker,
        doc4,
        json!([
            {
                "content": "# H1\nText 1",
                "metadata": {
                    "breadcrumb": "# H1",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 2,
                    "source_span": {
                        "start": 0,
                        "end": 11
                    }
                }
            },
            {
                "content": "## H2\nText 2",
                "metadata": {
                    "breadcrumb": "# H1 > ## H2",
                    "heading_level": 2,
                    "source_line": 3,
                    "source_line_end": 4,
                    "source_span": {
                        "start": 12,
                        "end": 24
                    }
                }
            },
            {
                "content": "### H3\nText 3",
                "metadata": {
                    "breadcrumb": "# H1 > ## H2 > ### H3",
                    "heading_level": 3,
                    "source_line": 5,
                    "source_line_end": 6,
                    "source_span": {
                        "start": 25,
                        "end": doc4.len()
                    }
                }
            }
        ]),
    );

    // 5. Ignored heading level (H4 with default split_levels [1, 2, 3])
    let doc5 = "# H1\nText\n#### H4 Ignored\nSubtext under H4";
    assert_golden(
        "05_ignored_h4",
        &default_chunker,
        doc5,
        json!([
            {
                "content": "# H1\nText\n#### H4 Ignored\nSubtext under H4",
                "metadata": {
                    "breadcrumb": "# H1",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 4,
                    "source_span": {
                        "start": 0,
                        "end": doc5.len()
                    }
                }
            }
        ]),
    );

    // 6. Merging small sections below min_tokens
    let low_min_config = ChunkerConfig {
        min_tokens: 50,
        split_levels: vec![1],
        ..Default::default()
    };
    let min_tokens_chunker = MarkdownChunker::new(low_min_config);
    let doc6 = "# Section A\nShort.\n# Section B\nAlso short.";
    assert_golden(
        "06_merge_small_sections",
        &min_tokens_chunker,
        doc6,
        json!([
            {
                "content": "# Section A\nShort.\n\n# Section B\nAlso short.",
                "metadata": {
                    "breadcrumb": "# Section A",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 4,
                    "source_span": {
                        "start": 0,
                        "end": doc6.len()
                    }
                }
            }
        ]),
    );

    // 7. Code block within target size preserved
    let doc7 = "```rust\nfn main() {\n    println!(\"hello\");\n}\n```";
    assert_golden(
        "07_code_block_small",
        &default_chunker,
        doc7,
        json!([
            {
                "content": "```rust\nfn main() {\n    println!(\"hello\");\n}\n```",
                "metadata": {
                    "breadcrumb": "",
                    "heading_level": 0,
                    "source_line": 1,
                    "source_line_end": 5,
                    "source_span": {
                        "start": 0,
                        "end": doc7.len()
                    }
                }
            }
        ]),
    );

    // 8. Code block with language tag under heading
    let doc8 = "# Code Section\n\n```python\n# Python code\ndef test():\n    return 42\n```";
    assert_golden(
        "08_code_block_lang_tag",
        &default_chunker,
        doc8,
        json!([
            {
                "content": "# Code Section\n\n```python\n# Python code\ndef test():\n    return 42\n```",
                "metadata": {
                    "breadcrumb": "# Code Section",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 7,
                    "source_span": {
                        "start": 0,
                        "end": doc8.len()
                    }
                }
            }
        ]),
    );

    // 9. Simple Markdown Table
    let doc9 = "| A | B |\n|---|---|\n| 1 | 2 |";
    assert_golden(
        "09_table_small",
        &default_chunker,
        doc9,
        json!([
            {
                "content": "| A | B |\n|---|---|\n| 1 | 2 |",
                "metadata": {
                    "breadcrumb": "",
                    "heading_level": 0,
                    "source_line": 1,
                    "source_line_end": 3,
                    "source_span": {
                        "start": 0,
                        "end": doc9.len()
                    }
                }
            }
        ]),
    );

    // 10. Table with section heading
    let doc10 = "# Metrics\n| Metric | Value |\n|---|---|\n| Latency | 5ms |";
    assert_golden(
        "10_table_under_heading",
        &default_chunker,
        doc10,
        json!([
            {
                "content": "# Metrics\n| Metric | Value |\n|---|---|\n| Latency | 5ms |",
                "metadata": {
                    "breadcrumb": "# Metrics",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 4,
                    "source_span": {
                        "start": 0,
                        "end": doc10.len()
                    }
                }
            }
        ]),
    );

    // 12. German Umlauts & Emojis
    let doc12 = "# Kapitel 1 🚀\nÜberprüfen von Straße, Ölen und Äpfeln 🔥.";
    assert_golden(
        "12_utf8_umlauts_emojis",
        &default_chunker,
        doc12,
        json!([
            {
                "content": "# Kapitel 1 🚀\nÜberprüfen von Straße, Ölen und Äpfeln 🔥.",
                "metadata": {
                    "breadcrumb": "# Kapitel 1 🚀",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 2,
                    "source_span": {
                        "start": 0,
                        "end": doc12.len()
                    }
                }
            }
        ]),
    );

    // 13. CJK text token estimation and chunking
    let doc13 = "# 标题\n这是中文测试段落。日本语のテキストも含まれています。";
    assert_golden(
        "13_cjk_text",
        &default_chunker,
        doc13,
        json!([
            {
                "content": "# 标题\n这是中文测试段落。日本语のテキストも含まれています。",
                "metadata": {
                    "breadcrumb": "# 标题",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 2,
                    "source_span": {
                        "start": 0,
                        "end": doc13.len()
                    }
                }
            }
        ]),
    );

    // 14. Alphanumeric words and numbers token counting
    let doc14 = "Testing numbers 1234567890 and longwords like supercalifragilisticexpialidocious.";
    assert_golden(
        "14_alphanumeric_numbers",
        &default_chunker,
        doc14,
        json!([
            {
                "content": "Testing numbers 1234567890 and longwords like supercalifragilisticexpialidocious.",
                "metadata": {
                    "breadcrumb": "",
                    "heading_level": 0,
                    "source_line": 1,
                    "source_line_end": 1,
                    "source_span": {
                        "start": 0,
                        "end": doc14.len()
                    }
                }
            }
        ]),
    );

    // 15. Paragraphs with multiple blank lines
    let doc15 = "First paragraph.\n\n\n\nSecond paragraph.";
    assert_golden(
        "15_multiple_blank_lines",
        &default_chunker,
        doc15,
        json!([
            {
                "content": "First paragraph.\n\n\n\nSecond paragraph.",
                "metadata": {
                    "breadcrumb": "",
                    "heading_level": 0,
                    "source_line": 1,
                    "source_line_end": 5,
                    "source_span": {
                        "start": 0,
                        "end": doc15.len()
                    }
                }
            }
        ]),
    );

    // 16. Document starting with H2 without preceding H1
    let doc16 = "## SubSection direct\nContent under sub-section.";
    assert_golden(
        "16_start_with_h2",
        &default_chunker,
        doc16,
        json!([
            {
                "content": "## SubSection direct\nContent under sub-section.",
                "metadata": {
                    "breadcrumb": "## SubSection direct",
                    "heading_level": 2,
                    "source_line": 1,
                    "source_line_end": 2,
                    "source_span": {
                        "start": 0,
                        "end": doc16.len()
                    }
                }
            }
        ]),
    );

    // 17. Trailing whitespace and empty lines
    let doc17 = "# Clean Heading   \nParagraph text with trailing space.   \n\n";
    assert_golden(
        "17_trailing_whitespace",
        &default_chunker,
        doc17,
        json!([
            {
                "content": "# Clean Heading   \nParagraph text with trailing space.   \n",
                "metadata": {
                    "breadcrumb": "# Clean Heading",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 3,
                    "source_span": {
                        "start": 0,
                        "end": 58
                    }
                }
            }
        ]),
    );

    // 18. Line starting with `#tag` without space (not a heading)
    let doc18 = "#tag1 #tag2\nThis is not a heading line.";
    assert_golden(
        "18_hashtag_not_heading",
        &default_chunker,
        doc18,
        json!([
            {
                "content": "#tag1 #tag2\nThis is not a heading line.",
                "metadata": {
                    "breadcrumb": "",
                    "heading_level": 0,
                    "source_line": 1,
                    "source_line_end": 2,
                    "source_span": {
                        "start": 0,
                        "end": doc18.len()
                    }
                }
            }
        ]),
    );

    // 19. Backticks inside paragraph
    let doc19 = "Use `let x = 10;` for inline variable declaration.";
    assert_golden(
        "19_inline_backticks",
        &default_chunker,
        doc19,
        json!([
            {
                "content": "Use `let x = 10;` for inline variable declaration.",
                "metadata": {
                    "breadcrumb": "",
                    "heading_level": 0,
                    "source_line": 1,
                    "source_line_end": 1,
                    "source_span": {
                        "start": 0,
                        "end": doc19.len()
                    }
                }
            }
        ]),
    );

    // 20. Consecutive code blocks
    let doc20 = "```sh\necho hello\n```\n```sh\necho world\n```";
    assert_golden(
        "20_consecutive_code_blocks",
        &default_chunker,
        doc20,
        json!([
            {
                "content": "```sh\necho hello\n```\n```sh\necho world\n```",
                "metadata": {
                    "breadcrumb": "",
                    "heading_level": 0,
                    "source_line": 1,
                    "source_line_end": 6,
                    "source_span": {
                        "start": 0,
                        "end": doc20.len()
                    }
                }
            }
        ]),
    );

    // 21. Consecutive tables
    let doc21 = "| H1 | H2 |\n|---|---|\n| A | B |\n\n| H3 | H4 |\n|---|---|\n| C | D |";
    assert_golden(
        "21_consecutive_tables",
        &default_chunker,
        doc21,
        json!([
            {
                "content": "| H1 | H2 |\n|---|---|\n| A | B |\n\n| H3 | H4 |\n|---|---|\n| C | D |",
                "metadata": {
                    "breadcrumb": "",
                    "heading_level": 0,
                    "source_line": 1,
                    "source_line_end": 7,
                    "source_span": {
                        "start": 0,
                        "end": doc21.len()
                    }
                }
            }
        ]),
    );

    // 22. Mixed document with H1, paragraph, code block, H2, table, H3 (merged under default min_tokens=50)
    let doc22 = "# Main Title\n\nIntro text.\n\n```rust\nlet a = 1;\n```\n\n## Sub Title\n\n| Col1 | Col2 |\n|---|---|\n| Val1 | Val2 |\n\n### SubSub Title\nFinal line.";
    assert_golden(
        "22_mixed_doc",
        &default_chunker,
        doc22,
        json!([
            {
                "content": "# Main Title\n\nIntro text.\n\n```rust\nlet a = 1;\n```\n\n\n## Sub Title\n\n| Col1 | Col2 |\n|---|---|\n| Val1 | Val2 |\n\n\n### SubSub Title\nFinal line.",
                "metadata": {
                    "breadcrumb": "# Main Title",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 16,
                    "source_span": {
                        "start": 0,
                        "end": doc22.len()
                    }
                }
            }
        ]),
    );

    // 23. include_breadcrumbs = false config
    let no_bc_config = ChunkerConfig {
        include_breadcrumbs: false,
        ..Default::default()
    };
    let no_bc_chunker = MarkdownChunker::new(no_bc_config);
    let doc23 = "# Heading\nSome text.";
    assert_golden(
        "23_no_breadcrumbs_config",
        &no_bc_chunker,
        doc23,
        json!([
            {
                "content": "# Heading\nSome text.",
                "metadata": null
            }
        ]),
    );

    // 24. H1 with space and special symbols in title
    let doc24 = "# Title: Contextra [v1.2] & (Test)";
    assert_golden(
        "24_heading_special_chars",
        &default_chunker,
        doc24,
        json!([
            {
                "content": "# Title: Contextra [v1.2] & (Test)",
                "metadata": {
                    "breadcrumb": "# Title: Contextra [v1.2] & (Test)",
                    "heading_level": 1,
                    "source_line": 1,
                    "source_line_end": 1,
                    "source_span": {
                        "start": 0,
                        "end": doc24.len()
                    }
                }
            }
        ]),
    );

    // 26. Backslash escaped characters in text
    let doc26 = "Escaped \\# heading and \\* bold \\* text.";
    assert_golden(
        "26_escaped_chars",
        &default_chunker,
        doc26,
        json!([
            {
                "content": "Escaped \\# heading and \\* bold \\* text.",
                "metadata": {
                    "breadcrumb": "",
                    "heading_level": 0,
                    "source_line": 1,
                    "source_line_end": 1,
                    "source_span": {
                        "start": 0,
                        "end": doc26.len()
                    }
                }
            }
        ]),
    );

    // 27. Blank line separation between code blocks
    let doc27 = "```\ncode1\n```\n\n```\ncode2\n```";
    assert_golden(
        "27_blank_lines_between_code_blocks",
        &default_chunker,
        doc27,
        json!([
            {
                "content": "```\ncode1\n```\n\n```\ncode2\n```",
                "metadata": {
                    "breadcrumb": "",
                    "heading_level": 0,
                    "source_line": 1,
                    "source_line_end": 7,
                    "source_span": {
                        "start": 0,
                        "end": doc27.len()
                    }
                }
            }
        ]),
    );
}
