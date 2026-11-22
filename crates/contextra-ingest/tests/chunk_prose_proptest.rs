// FILE-CONTEXT
// ZWECK: Property-Tests (proptest) für den ProseChunker (WP M3 / SPEC C-11.7).
// INVARIANTEN: kein Inhaltsverlust, UTF-8-Grenzen, token_count <= hard_limit, exakter Span oder Synthese, Monotonie, Determinismus.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

use contextra_ingest::chunking::common::token_count;
use contextra_ingest::chunking::config::ChunkingConfig;
use contextra_ingest::chunking::draft_document;
use contextra_ingest::format::FormatId;
use contextra_ingest::ir::{BlockKind, DocumentIr, Locator};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_prose_chunker_properties(
        headings in prop::collection::vec("[a-zA-Z0-9öäüß]{3,20}", 1..4),
        paragraphs in prop::collection::vec("[a-zA-Z0-9öäüß]{10,100}", 1..4),
        max_tokens in 50usize..200usize,
    ) {
        let mut doc = String::new();
        let num_items = headings.len().max(paragraphs.len());

        for i in 0..num_items {
            if let Some(h) = headings.get(i) {
                let level = (i % 3) + 1;
                let hashes = "#".repeat(level);
                doc.push_str(&format!("{} {}\n\n", hashes, h));
            }
            if let Some(p) = paragraphs.get(i) {
                doc.push_str(&format!("{}\n\n", p));
            }
        }

        let mut ir = DocumentIr::new(FormatId::Markdown);
        ir.push(
            BlockKind::Raw {
                text: doc.clone(),
            },
            Locator::default(),
        );

        let canon = ir.canonical();

        let mut cfg = ChunkingConfig::default();
        cfg.max_tokens = max_tokens;
        cfg.min_tokens = 10;
        cfg.hard_limit_pct = 120;
        cfg.overlap_pct = 20;
        cfg.include_breadcrumbs = true;
        cfg.split_levels = vec![1, 2, 3];

        let hard_limit = (max_tokens * 120) / 100;

        let run1 = draft_document(&ir, &canon, &cfg).unwrap();
        let run2 = draft_document(&ir, &canon, &cfg).unwrap();

        // (f) Determinismus
        prop_assert_eq!(&run1, &run2, "Determinismus-Prüfung fehlgeschlagen");

        if run1.is_empty() {
            return Ok(());
        }

        let mut prev_start = 0usize;

        for draft in &run1 {
            // (b) UTF-8 Grenzen
            prop_assert!(canon.text.is_char_boundary(draft.span.start));
            prop_assert!(canon.text.is_char_boundary(draft.span.end));

            // (e) Monotonie der Spans
            prop_assert!(
                draft.span.start >= prev_start || draft.extra.contains_key("synthesized"),
                "Span-Monotonie verletzt: draft.start={}, prev_start={}",
                draft.span.start,
                prev_start
            );
            prev_start = draft.span.start;

            // (c) Token-Limit
            let tok = token_count(&draft.text);
            if tok > hard_limit {
                prop_assert!(
                    draft.extra.get("oversize").and_then(|v| v.as_bool()).unwrap_or(false),
                    "Chunk überschreitet Hard-Limit {}, ohne oversize=true geflaggt zu sein (tok={})",
                    hard_limit,
                    tok
                );
            }

            // (d) Exakter Slice oder synthesized
            if draft.extra.get("synthesized").and_then(|v| v.as_bool()).unwrap_or(false) {
                prop_assert!(!draft.text.is_empty());
            } else if let Some(slice) = canon.text.get(draft.span.clone()) {
                let draft_words: Vec<&str> = draft.text.split_whitespace().collect();
                let slice_words: Vec<&str> = slice.split_whitespace().collect();
                prop_assert_eq!(draft_words, slice_words);
            }
        }

        // (a) Kein Inhaltsverlust
        let concat_prose = run1.iter().map(|d| d.text.as_str()).collect::<Vec<_>>().join(" ");
        let orig_words: Vec<&str> = doc.split_whitespace().collect();
        let prose_words: Vec<&str> = concat_prose.split_whitespace().collect();

        prop_assert_eq!(orig_words, prose_words, "Inhaltsverlust in Wortfolge festgestellt!");
    }
}
