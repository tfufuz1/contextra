// FILE-CONTEXT
// ZWECK: Akzeptanztests und Zitierbarkeitstests für PagedChunker (WP-ING-14).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic, clippy::cast_possible_truncation)]
// INVARIANTEN: Zero-Panic, deterministisch, präzise Zitierbarkeit, Span-Konsistenz.
// STAND: TS:2026-10-31T00:00:00Z

use contextra_ingest::chunking::config::ChunkingConfig;
use contextra_ingest::chunking::paged::PagedChunker;
use contextra_ingest::chunking::IrChunker;
use contextra_ingest::format::FormatId;
use contextra_ingest::ir::{
    BlockKind, Boundary, DocumentIr, Locator, NoteKind, Table,
};

fn page_loc(page: u32) -> Locator {
    let mut loc = Locator::default();
    loc.page = Some(page);
    loc
}

fn slide_loc(slide: u32) -> Locator {
    let mut loc = Locator::default();
    loc.slide = Some(slide);
    loc
}

#[test]
fn test_header_footer_note_isolation() {
    let mut ir = DocumentIr::new(FormatId::Pdf);

    ir.push(
        BlockKind::Boundary(Boundary::Page { number: 1 }),
        page_loc(1),
    );
    ir.push(
        BlockKind::Note {
            kind: NoteKind::Header,
            text: "Kopfzeile Dokument".into(),
        },
        page_loc(1),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "Erster Absatz auf Seite 1.".into(),
        },
        page_loc(1),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "Zweiter Absatz auf Seite 1.".into(),
        },
        page_loc(1),
    );
    ir.push(
        BlockKind::Note {
            kind: NoteKind::Footer,
            text: "Fußzeile Dokument".into(),
        },
        page_loc(1),
    );

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();

    let drafts = PagedChunker.draft(&ir, &canon, &cfg).unwrap();
    assert_eq!(drafts.len(), 1);

    let draft = &drafts[0];
    assert!(!draft.text.contains("Kopfzeile Dokument"));
    assert!(!draft.text.contains("Fußzeile Dokument"));
    assert_eq!(
        draft.extra.get("header"),
        Some(&serde_json::Value::String("Kopfzeile Dokument".into()))
    );
    assert_eq!(
        draft.extra.get("footer"),
        Some(&serde_json::Value::String("Fußzeile Dokument".into()))
    );
}

#[test]
fn test_pdf_40_small_pages_merged() {
    let mut ir = DocumentIr::new(FormatId::Pdf);

    for page_num in 1..=40 {
        ir.push(
            BlockKind::Boundary(Boundary::Page { number: page_num }),
            page_loc(page_num),
        );
        ir.push(
            BlockKind::Paragraph {
                text: format!("Dies ist ein kurzer Satz auf Seite {}.", page_num),
            },
            page_loc(page_num),
        );
    }

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.paged.merge_small_pages = true;
    cfg.paged.join_cross_page_paragraphs = false;
    cfg.paged.notes_with_slide = true;

    let drafts = PagedChunker.draft(&ir, &canon, &cfg).unwrap();

    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].locator.page, Some(1));
    assert_eq!(drafts[0].locator.page_end, Some(40));
    assert_eq!(&canon.text[drafts[0].span.clone()], drafts[0].text);
}

#[test]
fn test_pdf_6000_tokens_large_page_parts() {
    let mut ir = DocumentIr::new(FormatId::Pdf);
    ir.push(
        BlockKind::Boundary(Boundary::Page { number: 1 }),
        page_loc(1),
    );

    let mut sentences = Vec::new();
    for i in 1..=800 {
        sentences.push(format!("Satz Nummer {} beschreibt ein langes Dokument.", i));
    }
    let large_text = sentences.join(" ");

    ir.push(
        BlockKind::Paragraph { text: large_text },
        page_loc(1),
    );

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.max_tokens = 512;
    cfg.hard_limit_pct = 120;

    let drafts = PagedChunker.draft(&ir, &canon, &cfg).unwrap();

    assert!(drafts.len() > 1);

    for draft in &drafts {
        assert_eq!(draft.locator.page, Some(1));
        assert_eq!(draft.locator.page_end, Some(1));
        assert!(draft.extra.contains_key("part"));
        let part_val = draft.extra["part"].as_str().unwrap();
        assert!(part_val.contains('/'));
        assert_eq!(&canon.text[draft.span.clone()], draft.text);
    }
}

#[test]
fn test_cross_page_paragraph_join() {
    let mut ir = DocumentIr::new(FormatId::Pdf);

    // Seite 1 endet OHNE Punkt/Satzendzeichen
    ir.push(
        BlockKind::Boundary(Boundary::Page { number: 1 }),
        page_loc(1),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "Dies ist der erste Teil eines unvollständigen Satzes".into(),
        },
        page_loc(1),
    );

    // Seite 2 beginnt mit Kleinbuchstabe
    ir.push(
        BlockKind::Boundary(Boundary::Page { number: 2 }),
        page_loc(2),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "und wird hier auf der zweiten Seite fortgesetzt.".into(),
        },
        page_loc(2),
    );

    let canon = ir.canonical();

    // Fall 1: join_cross_page_paragraphs = true
    let mut cfg_join = ChunkingConfig::default();
    cfg_join.paged.merge_small_pages = false;
    cfg_join.paged.join_cross_page_paragraphs = true;
    cfg_join.paged.notes_with_slide = true;

    let drafts_join = PagedChunker.draft(&ir, &canon, &cfg_join).unwrap();
    assert_eq!(drafts_join.len(), 1);
    assert_eq!(drafts_join[0].locator.page, Some(1));
    assert_eq!(drafts_join[0].locator.page_end, Some(2));
    assert_eq!(
        drafts_join[0].extra.get("cross_page_join"),
        Some(&serde_json::Value::Bool(true))
    );

    // Fall 2: join_cross_page_paragraphs = false
    let mut cfg_no_join = ChunkingConfig::default();
    cfg_no_join.paged.merge_small_pages = false;
    cfg_no_join.paged.join_cross_page_paragraphs = false;
    cfg_no_join.paged.notes_with_slide = true;

    let drafts_no_join = PagedChunker.draft(&ir, &canon, &cfg_no_join).unwrap();
    assert_eq!(drafts_no_join.len(), 2);
    assert_eq!(drafts_no_join[0].locator.page, Some(1));
    assert_eq!(drafts_no_join[1].locator.page, Some(2));
}

#[test]
fn test_bookmark_hierarchy_across_pages() {
    let mut ir = DocumentIr::new(FormatId::Pdf);

    ir.push(
        BlockKind::Boundary(Boundary::Page { number: 1 }),
        page_loc(1),
    );
    ir.push(
        BlockKind::Heading {
            level: 1,
            text: "Kapitel 1".into(),
        },
        page_loc(1),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "Einleitungstext zu Kapitel 1.".into(),
        },
        page_loc(1),
    );

    ir.push(
        BlockKind::Boundary(Boundary::Page { number: 2 }),
        page_loc(2),
    );
    ir.push(
        BlockKind::Heading {
            level: 2,
            text: "Abschnitt 1.1".into(),
        },
        page_loc(2),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "Detailtext zu Abschnitt 1.1.".into(),
        },
        page_loc(2),
    );

    ir.push(
        BlockKind::Boundary(Boundary::Page { number: 3 }),
        page_loc(3),
    );
    ir.push(
        BlockKind::Heading {
            level: 1,
            text: "Kapitel 2".into(),
        },
        page_loc(3),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "Inhalt von Kapitel 2.".into(),
        },
        page_loc(3),
    );

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.paged.merge_small_pages = false;

    let drafts = PagedChunker.draft(&ir, &canon, &cfg).unwrap();
    assert_eq!(drafts.len(), 3);

    assert_eq!(drafts[0].breadcrumb.as_deref(), Some("# Kapitel 1"));
    assert_eq!(
        drafts[1].breadcrumb.as_deref(),
        Some("# Kapitel 1 > ## Abschnitt 1.1")
    );
    assert_eq!(drafts[2].breadcrumb.as_deref(), Some("# Kapitel 2"));
}

#[test]
fn test_presentation_slides_and_notes() {
    let mut ir = DocumentIr::new(FormatId::Pptx);

    ir.push(
        BlockKind::Boundary(Boundary::Slide {
            number: 1,
            title: Some("Titelfolie".into()),
        }),
        slide_loc(1),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "Willkommen zur Präsentation".into(),
        },
        slide_loc(1),
    );

    for s in 2..=11 {
        ir.push(
            BlockKind::Boundary(Boundary::Slide {
                number: s,
                title: Some(format!("Folie {}", s)),
            }),
            slide_loc(s),
        );
        ir.push(
            BlockKind::Paragraph {
                text: format!("Hauptinhalt von Folie {}", s),
            },
            slide_loc(s),
        );
        ir.push(
            BlockKind::Note {
                kind: NoteKind::SpeakerNotes,
                text: format!("Notiz für Vortragenden auf Folie {}", s),
            },
            slide_loc(s),
        );
    }

    let canon = ir.canonical();

    // Fall A: notes_with_slide = true
    let mut cfg_with_notes = ChunkingConfig::default();
    cfg_with_notes.paged.merge_small_pages = false;
    cfg_with_notes.paged.join_cross_page_paragraphs = false;
    cfg_with_notes.paged.notes_with_slide = true;

    let drafts_with_notes = PagedChunker.draft(&ir, &canon, &cfg_with_notes).unwrap();
    assert_eq!(drafts_with_notes.len(), 11);
    assert_eq!(
        drafts_with_notes[0].breadcrumb.as_deref(),
        Some("Folie 1: Titelfolie")
    );
    assert!(drafts_with_notes[1].text.contains("Notiz für Vortragenden"));

    // Fall B: notes_with_slide = false
    let mut cfg_no_notes = ChunkingConfig::default();
    cfg_no_notes.paged.merge_small_pages = false;
    cfg_no_notes.paged.join_cross_page_paragraphs = false;
    cfg_no_notes.paged.notes_with_slide = false;

    let drafts_no_notes = PagedChunker.draft(&ir, &canon, &cfg_no_notes).unwrap();
    assert_eq!(drafts_no_notes.len(), 21);
}

#[test]
fn test_empty_pages_produce_no_chunks() {
    let mut ir = DocumentIr::new(FormatId::Pdf);

    ir.push(
        BlockKind::Boundary(Boundary::Page { number: 1 }),
        page_loc(1),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "   \n\t  ".into(),
        },
        page_loc(1),
    );

    ir.push(
        BlockKind::Boundary(Boundary::Page { number: 2 }),
        page_loc(2),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "Tatsächlicher Inhalt auf Seite 2.".into(),
        },
        page_loc(2),
    );

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();

    let drafts = PagedChunker.draft(&ir, &canon, &cfg).unwrap();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].locator.page, Some(2));
}

#[test]
fn test_large_table_splitting_and_headers() {
    let mut ir = DocumentIr::new(FormatId::Pdf);
    ir.push(
        BlockKind::Boundary(Boundary::Page { number: 1 }),
        page_loc(1),
    );

    let mut rows = Vec::new();
    rows.push(vec!["Spalte A".to_string(), "Spalte B".to_string()]);
    for r in 1..=400 {
        rows.push(vec![format!("Zeile {}A", r), format!("Zeile {}B", r)]);
    }

    let mut table = Table::default();
    table.caption = Some("Große Tabelle".into());
    table.header_rows = 1;
    table.rows = rows;

    ir.push(BlockKind::Table(table), page_loc(1));

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.max_tokens = 256;
    cfg.hard_limit_pct = 120;

    let drafts = PagedChunker.draft(&ir, &canon, &cfg).unwrap();
    assert!(drafts.len() > 1);

    for draft in &drafts {
        assert_eq!(draft.locator.page, Some(1));
        assert!(draft.text.contains("| Spalte A | Spalte B |"));
        assert!(draft.text.contains("|---|---|"));
    }
}

struct LcgRng {
    state: u64,
}

impl LcgRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    fn gen_range(&mut self, low: usize, high: usize) -> usize {
        if low >= high {
            return low;
        }
        let range = high - low;
        low + (self.next_u64() as usize % range)
    }
}

#[test]
fn test_citation_accuracy_200_snippets() {
    let mut ir = DocumentIr::new(FormatId::Pdf);
    let mut rng = LcgRng::new(42);

    let mut page_sentence_map = Vec::new();

    for page_num in 1..=20 {
        ir.push(
            BlockKind::Boundary(Boundary::Page { number: page_num }),
            page_loc(page_num),
        );

        let sentence = format!(
            "Spezifischer eindeutiger Referenzsatz fuer Zitierung auf Seite {} mit Token {}.",
            page_num,
            rng.next_u64()
        );

        ir.push(
            BlockKind::Paragraph {
                text: sentence.clone(),
            },
            page_loc(page_num),
        );

        page_sentence_map.push((page_num, sentence));
    }

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.paged.merge_small_pages = false;

    let drafts = PagedChunker.draft(&ir, &canon, &cfg).unwrap();
    assert_eq!(drafts.len(), 20);

    for _ in 0..200 {
        let idx = rng.gen_range(0, 20);
        let (expected_page, ref sentence) = page_sentence_map[idx];

        let matching_draft = drafts.iter().find(|d| d.text.contains(sentence));
        assert!(matching_draft.is_some(), "Satz nicht in Drafts gefunden");

        let draft = matching_draft.unwrap();
        assert_eq!(
            draft.locator.page,
            Some(expected_page),
            "Falsche Seitennummer für Zitat"
        );
    }
}
