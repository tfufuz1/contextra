// FILE-CONTEXT
// ZWECK: Property-basierte Tests für PagedChunker (WP-ING-14).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic, clippy::cast_possible_truncation)]
// INVARIANTEN: Zero-Panic, Abdeckung aller Seiten, page <= page_end, monotone Spans, Determinismus.
// STAND: TS:2026-10-31T00:00:00Z

use contextra_ingest::chunking::config::ChunkingConfig;
use contextra_ingest::chunking::paged::PagedChunker;
use contextra_ingest::chunking::IrChunker;
use contextra_ingest::format::FormatId;
use contextra_ingest::ir::{BlockKind, Boundary, DocumentIr, Locator};
use proptest::prelude::*;

fn page_locator(p: u32) -> Locator {
    let mut loc = Locator::default();
    loc.page = Some(p);
    loc
}

fn slide_locator(s: u32) -> Locator {
    let mut loc = Locator::default();
    loc.slide = Some(s);
    loc
}

proptest! {
    #[test]
    fn prop_pdf_page_invariants(
        num_pages in 1u32..20u32,
        paragraphs_per_page in 1usize..5usize,
        max_tokens in 50usize..500usize,
        merge_small_pages in proptest::bool::ANY,
        join_cross_page in proptest::bool::ANY,
    ) {
        let mut ir = DocumentIr::new(FormatId::Pdf);
        let mut non_empty_pages = Vec::new();

        for p in 1..=num_pages {
            ir.push(
                BlockKind::Boundary(Boundary::Page { number: p }),
                page_locator(p),
            );

            let mut page_has_text = false;
            for k in 0..paragraphs_per_page {
                let text = format!("Inhalt auf Seite {} Paragraph {}.", p, k);
                if !text.trim().is_empty() {
                    page_has_text = true;
                }
                ir.push(
                    BlockKind::Paragraph { text },
                    page_locator(p),
                );
            }

            if page_has_text {
                non_empty_pages.push(p);
            }
        }

        let canon = ir.canonical();
        let mut cfg = ChunkingConfig::default();
        cfg.max_tokens = max_tokens;
        cfg.paged.merge_small_pages = merge_small_pages;
        cfg.paged.join_cross_page_paragraphs = join_cross_page;

        let drafts1 = PagedChunker.draft(&ir, &canon, &cfg).unwrap();
        let drafts2 = PagedChunker.draft(&ir, &canon, &cfg).unwrap();

        // 1. Determinismus
        prop_assert_eq!(&drafts1, &drafts2);

        if non_empty_pages.is_empty() {
            prop_assert!(drafts1.is_empty());
        } else {
            // 2. Abdeckung: Jede nicht-leere Seite ist von mindestens einem Chunk abgedeckt
            for &p in &non_empty_pages {
                let covered = drafts1.iter().any(|d| {
                    if let (Some(start), Some(end)) = (d.locator.page, d.locator.page_end) {
                        p >= start && p <= end
                    } else {
                        false
                    }
                });
                prop_assert!(covered, "Seite {} nicht von Chunks abgedeckt", p);
            }

            // 3. page <= page_end & Monotonie & Spans
            let mut last_page_end = 0u32;
            let mut last_span_end = 0usize;

            for d in &drafts1 {
                let p_start = d.locator.page.expect("page must be set");
                let p_end = d.locator.page_end.expect("page_end must be set");

                prop_assert!(p_start <= p_end, "page > page_end");
                prop_assert!(p_start >= last_page_end, "Seitenbereiche nicht aufsteigend");
                last_page_end = p_start;

                prop_assert!(d.span.start <= d.span.end, "Span start > end");
                prop_assert!(d.span.start >= last_span_end, "Spans nicht monoton");
                last_span_end = d.span.start;

                // 4. Valid UTF-8 & Span Invariant
                let slice = &canon.text[d.span.clone()];
                prop_assert_eq!(slice, &d.text);
            }
        }
    }

    #[test]
    fn prop_presentation_slide_invariants(
        num_slides in 1u32..15u32,
        notes_with_slide in proptest::bool::ANY,
        max_tokens in 50usize..300usize,
    ) {
        let mut ir = DocumentIr::new(FormatId::Pptx);

        for s in 1..=num_slides {
            ir.push(
                BlockKind::Boundary(Boundary::Slide {
                    number: s,
                    title: Some(format!("Folie {}", s)),
                }),
                slide_locator(s),
            );
            ir.push(
                BlockKind::Paragraph {
                    text: format!("Haupttext der Folie {}.", s),
                },
                slide_locator(s),
            );
            ir.push(
                BlockKind::Note {
                    kind: contextra_ingest::ir::NoteKind::SpeakerNotes,
                    text: format!("Notizen zu Folie {}.", s),
                },
                slide_locator(s),
            );
        }

        let canon = ir.canonical();
        let mut cfg = ChunkingConfig::default();
        cfg.max_tokens = max_tokens;
        cfg.paged.notes_with_slide = notes_with_slide;

        let drafts = PagedChunker.draft(&ir, &canon, &cfg).unwrap();

        for d in &drafts {
            let s_start = d.locator.slide.expect("slide must be set");
            let s_end = d.locator.slide_end.expect("slide_end must be set");

            prop_assert!(s_start <= s_end, "slide > slide_end");
            prop_assert_eq!(&canon.text[d.span.clone()], &d.text);
        }
    }
}
