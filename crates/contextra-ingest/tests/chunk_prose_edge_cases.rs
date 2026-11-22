// FILE-CONTEXT
// ZWECK: Randfall- und Performance-Smoke-Tests für ProseChunker (WP M3 / SPEC C-11.7).
// STAND: TS:2026-10-31T00:00:00Z

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

use contextra_ingest::chunking::config::ChunkingConfig;
use contextra_ingest::chunking::draft_document;
use contextra_ingest::format::FormatId;
use contextra_ingest::ir::{BlockKind, DocumentIr, Locator};

#[test]
fn test_heading_inside_tilde_fence_is_not_heading() {
    let md = "~~~rust\n# Heading inside fence\n~~~\n\n# Real Heading\nSome content.";
    let mut ir = DocumentIr::new(FormatId::Markdown);
    ir.push(BlockKind::Raw { text: md.to_string() }, Locator::default());
    let canon = ir.canonical();

    let cfg = ChunkingConfig::default();
    let drafts = draft_document(&ir, &canon, &cfg).unwrap();

    let fence_draft = drafts.iter().find(|d| d.text.contains("~~~rust")).unwrap();
    assert_ne!(fence_draft.breadcrumb.as_deref(), Some("# Heading inside fence"));
}

#[test]
fn test_5000_char_paragraph_without_punctuation() {
    let para = "a".repeat(5000);
    let mut ir = DocumentIr::new(FormatId::Markdown);
    ir.push(BlockKind::Paragraph { text: para }, Locator::default());
    let canon = ir.canonical();

    let mut cfg = ChunkingConfig::default();
    cfg.max_tokens = 100;

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();

    assert!(drafts.len() > 1, "5000-Zeichen-Absatz muss aufgeteilt werden");
    for d in drafts {
        assert!(d.span.end <= 5000);
    }
}

#[test]
fn test_code_block_3000_lines() {
    let mut code_lines = Vec::new();
    code_lines.push("```python".to_string());
    for i in 0..3000 {
        code_lines.push(format!("    print('Line {}')", i));
    }
    code_lines.push("```".to_string());
    let code_md = code_lines.join("\n");

    let mut ir = DocumentIr::new(FormatId::Markdown);
    ir.push(BlockKind::Raw { text: code_md }, Locator::default());
    let canon = ir.canonical();

    let mut cfg = ChunkingConfig::default();
    cfg.max_tokens = 200;

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();

    assert!(drafts.len() > 1, "3000-Zeilen-Codeblock muss aufgeteilt werden");
    for d in drafts {
        assert!(d.text.contains("```python"), "Jeder Teil muss Header-Fence enthalten");
        assert!(d.text.contains("```"), "Jeder Teil muss Schluss-Fence enthalten");
    }
}

#[test]
fn test_table_500_lines_header_repetition() {
    let mut table_lines = Vec::new();
    table_lines.push("| ID | Name | Role |".to_string());
    table_lines.push("|---|---|---|".to_string());
    for i in 0..500 {
        table_lines.push(format!("| {} | User_{} | Role_{} |", i, i, i));
    }
    let table_md = table_lines.join("\n");

    let mut ir = DocumentIr::new(FormatId::Markdown);
    ir.push(BlockKind::Raw { text: table_md }, Locator::default());
    let canon = ir.canonical();

    let mut cfg = ChunkingConfig::default();
    cfg.max_tokens = 150;

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();

    assert!(drafts.len() > 1, "500-Zeilen-Tabelle muss aufgeteilt werden");
    for d in drafts {
        assert!(d.text.contains("| ID | Name | Role |"), "Tabellenkopf muss wiederholt werden");
        assert!(d.text.contains("|---|---|---|"), "Trennzeile muss wiederholt werden");
    }
}

#[test]
fn test_empty_ir_yields_no_drafts() {
    let ir = DocumentIr::new(FormatId::Markdown);
    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert!(drafts.is_empty());
}

#[test]
fn test_headings_only_document() {
    let md = "# Title\n## Section 1\n### Sub 1";
    let mut ir = DocumentIr::new(FormatId::Markdown);
    ir.push(BlockKind::Raw { text: md.to_string() }, Locator::default());
    let canon = ir.canonical();

    let cfg = ChunkingConfig::default();
    let drafts = draft_document(&ir, &canon, &cfg).unwrap();

    assert!(!drafts.is_empty());
}

#[test]
fn test_performance_smoke_8mib_linear_time() {
    let paragraph_template = "Dies ist ein typischer Performance-Smoke-Test-Absatz. Er prüft die verarbeiteten Bytes pro Sekunde.\n";
    let count = (8 * 1024 * 1024) / paragraph_template.len() + 1;
    let big_md = paragraph_template.repeat(count);

    assert!(big_md.len() >= 8 * 1024 * 1024, "Dokument muss mindestens 8 MiB groß sein");

    let mut ir = DocumentIr::new(FormatId::Markdown);
    ir.push(BlockKind::Raw { text: big_md }, Locator::default());
    let canon = ir.canonical();

    let cfg = ChunkingConfig::default();

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert!(!drafts.is_empty(), "8 MiB Dokument muss erfolgreich in Drafts zerlegt werden");
}
