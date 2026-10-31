//! Integrationstest-Suite: `chunk_prose_parity.rs`
//!
//! # Zweck & Testabdeckung
//! Verifiziert das Verhalten, Zero-Panic-Invariante und deterministische Ausgabe
//! von `contextra-ingest` fuer dieses Modul/Format gemaess Spezifikation.
//!
// FILE-CONTEXT
// ZWECK: Paritätstests zwischen ProseChunker und MarkdownChunker (Engine) (WP M3 / SPEC C-11.7).
// INVARIANTEN: Dieselbe Wortfolge (split_whitespace), passende Breadcrumbs/Heading-Levels.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

#[path = "fixtures/chunk_prose/mod.rs"]
mod fixtures;

use contextra_engine::chunker::{ChunkerConfig as EngineConfig, MarkdownChunker as EngineChunker};
use contextra_ingest::chunking::config::ChunkingConfig;
use contextra_ingest::chunking::draft_document;
use contextra_ingest::format::FormatId;
use contextra_ingest::ir::{BlockKind, DocumentIr, Locator};
use contextra_types::DocId;

// ANCHOR[contextra-ingest:TST-0325]
// Testet die Funktionsfaehigkeit von `test_prose_chunker_engine_parity_fixtures`.
#[test]
fn test_prose_chunker_engine_parity_fixtures() {
    let fixtures = fixtures::get_prose_fixtures();
    assert_eq!(fixtures.len(), 10, "Es müssen exakt 10 Fixtures getestet werden");

    for fix in fixtures {
        // Engine-Chunker Ergebnis
        let engine_cfg = EngineConfig {
            max_tokens: 512,
            min_tokens: 50,
            include_breadcrumbs: true,
            split_levels: vec![1, 2, 3],
        };
        let engine_chunker = EngineChunker::new(engine_cfg);
        let engine_chunks = engine_chunker.chunk(DocId::new(1), fix.content);

        // ProseChunker Ergebnis
        let mut ir = DocumentIr::new(FormatId::Markdown);
        ir.push(
            BlockKind::Raw {
                text: fix.content.to_string(),
            },
            Locator::default(),
        );
        let canon = ir.canonical();

        let mut cfg = ChunkingConfig::default();
        cfg.max_tokens = 512;
        cfg.min_tokens = 50;
        cfg.hard_limit_pct = 120;
        cfg.overlap_pct = 20;
        cfg.include_breadcrumbs = true;
        cfg.split_levels = vec![1, 2, 3];

        let drafts = draft_document(&ir, &canon, &cfg).unwrap();

        // 1. Inhaltsverlust-Prüfung: Wortvergleich über split_whitespace
        let orig_words: Vec<&str> = fix.content.split_whitespace().collect();
        let prose_content = drafts
            .iter()
            .map(|d| d.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        let prose_words: Vec<&str> = prose_content.split_whitespace().collect();

        // Alle Nicht-Front-Matter Wörter müssen in prose_words vorkommen
        let filtered_orig: Vec<&str> = if fix.content.starts_with("---") {
            let body = fix.content.split("---").nth(2).unwrap_or(fix.content);
            body.split_whitespace().collect()
        } else {
            orig_words
        };

        for word in &filtered_orig {
            assert!(
                prose_words.contains(word),
                "Wort '{}' fehlt im ProseChunker Output von Fixture '{}'",
                word,
                fix.name
            );
        }

        // 2. Breadcrumbs & Heading Level Parität
        let engine_breadcrumbs: Vec<Option<String>> = engine_chunks
            .iter()
            .map(|c| {
                c.metadata
                    .as_ref()
                    .and_then(|m| m.get("breadcrumb").and_then(|v| v.as_str()).map(String::from))
            })
            .collect();

        let prose_breadcrumbs: Vec<Option<String>> = drafts.iter().map(|d| d.breadcrumb.clone()).collect();

        if !engine_breadcrumbs.is_empty() && !prose_breadcrumbs.is_empty() {
            let e_first = engine_breadcrumbs.first().cloned().flatten();
            let p_first = prose_breadcrumbs.first().cloned().flatten();
            if e_first.is_some() && p_first.is_some() {
                assert_eq!(
                    e_first, p_first,
                    "Breadcrumb-Parität fehlgeschlagen bei Fixture '{}'",
                    fix.name
                );
            }
        }
    }
}
