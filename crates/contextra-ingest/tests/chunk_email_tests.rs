//! Integrationstest-Suite: `chunk_email_tests.rs`
//!
//! # Zweck & Testabdeckung
//! Verifiziert das Verhalten, Zero-Panic-Invariante und deterministische Ausgabe
//! von `contextra-ingest` fuer dieses Modul/Format gemaess Spezifikation.
//!
// FILE-CONTEXT
// ZWECK: Akzeptanz- und Proptests für EmailChunker (WP-ING-15).
// INVARIANTEN: Zero-Panic in Produktion, Test-First, Lints eingehalten.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use contextra_ingest::chunking::config::ChunkingConfig;
use contextra_ingest::chunking::email::EmailChunker;
use contextra_ingest::chunking::IrChunker;
use contextra_ingest::format::FormatId;
use contextra_ingest::ir::{BlockKind, Boundary, DocumentIr, Locator, MessageHeader, Table};
use proptest::prelude::*;

// ANCHOR[contextra-ingest:TST-0301]
// Testet die Funktionsfaehigkeit von `test_email_3_messages_isolation`.
#[test]
fn test_email_3_messages_isolation() {
    let mut ir = DocumentIr::new(FormatId::Mbox);

    for i in 1u32..=3u32 {
        let mut msg_header = MessageHeader::default();
        msg_header.subject = Some(format!("Message Subject {}", i));
        msg_header.from = vec![format!("user{}@example.com", i)];
        msg_header.date = Some("2026-10-31".into());

        let mut loc = Locator::default();
        loc.message = Some(i);

        ir.push(BlockKind::Message(msg_header), loc.clone());
        ir.push(
            BlockKind::Paragraph {
                text: format!("Body content for message {}", i),
            },
            loc,
        );
    }

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();
    let chunker = EmailChunker;

    let drafts = chunker.draft(&ir, &canon, &cfg).unwrap();

    assert_eq!(drafts.len(), 3);
    for (idx, draft) in drafts.iter().enumerate() {
        let expected_msg_num = u32::try_from(idx + 1).unwrap();
        assert_eq!(draft.locator.message, Some(expected_msg_num));
        assert_eq!(
            draft.breadcrumb,
            Some(format!("Message Subject {}", expected_msg_num))
        );
    }
}

// ANCHOR[contextra-ingest:TST-0302]
// Testet die Funktionsfaehigkeit von `test_email_quoted_replies_toggle`.
#[test]
fn test_email_quoted_replies_toggle() {
    let mut ir = DocumentIr::new(FormatId::Eml);

    let mut msg_header = MessageHeader::default();
    msg_header.subject = Some("Re: Project Status".into());
    msg_header.from = vec!["alice@example.com".into()];
    msg_header.date = Some("2026-10-31".into());

    ir.push(BlockKind::Message(msg_header), Locator::default());
    ir.push(
        BlockKind::Paragraph {
            text: "Here is my latest reply.".into(),
        },
        Locator::default(),
    );
    ir.push(
        BlockKind::Quote {
            text: "> On yesterday, Bob wrote:\n> Status is green.".into(),
        },
        Locator::default(),
    );

    let canon = ir.canonical();

    // 1. include_quoted = false
    let mut cfg_no_quoted = ChunkingConfig::default();
    cfg_no_quoted.email.include_quoted = false;
    cfg_no_quoted.email.header_context = true;

    let chunker = EmailChunker;
    let drafts_no_quoted = chunker.draft(&ir, &canon, &cfg_no_quoted).unwrap();

    assert_eq!(drafts_no_quoted.len(), 1);
    assert_eq!(
        drafts_no_quoted[0].extra.get("quoted_skipped"),
        Some(&serde_json::json!(1))
    );

    // 2. include_quoted = true
    let mut cfg_quoted = ChunkingConfig::default();
    cfg_quoted.email.include_quoted = true;
    cfg_quoted.email.header_context = true;

    let drafts_quoted = chunker.draft(&ir, &canon, &cfg_quoted).unwrap();

    assert_eq!(drafts_quoted.len(), 1);
    assert!(drafts_quoted[0].text.contains("Bob wrote"));
}

// ANCHOR[contextra-ingest:TST-0303]
// Testet die Funktionsfaehigkeit von `test_email_60000_char_body_window_split`.
#[test]
fn test_email_60000_char_body_window_split() {
    let mut ir = DocumentIr::new(FormatId::Eml);

    let mut msg_header = MessageHeader::default();
    msg_header.subject = Some("Long Report".into());
    msg_header.from = vec!["reporter@example.com".into()];

    ir.push(BlockKind::Message(msg_header), Locator::default());
    ir.push(
        BlockKind::Paragraph {
            text: "X".repeat(60_000),
        },
        Locator::default(),
    );

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();
    let chunker = EmailChunker;

    let drafts = chunker.draft(&ir, &canon, &cfg).unwrap();

    assert!(drafts.len() > 1, "60,000 char body must be window-split");
    for draft in &drafts {
        assert_eq!(draft.locator.message, Some(1));
        assert!(draft.text.starts_with("Long Report | reporter@example.com"));
    }
}

// ANCHOR[contextra-ingest:TST-0304]
// Testet die Funktionsfaehigkeit von `test_email_attachments_table_and_prose`.
#[test]
fn test_email_attachments_table_and_prose() {
    let mut ir = DocumentIr::new(FormatId::Eml);

    let mut msg_header = MessageHeader::default();
    msg_header.subject = Some("Quarterly Figures".into());
    msg_header.from = vec!["finance@example.com".into()];

    ir.push(BlockKind::Message(msg_header), Locator::default());
    ir.push(
        BlockKind::Paragraph {
            text: "Please find attached the figures and notes.".into(),
        },
        Locator::default(),
    );

    // Anhang 1: Table (XLSX/CSV)
    ir.push(
        BlockKind::Boundary(Boundary::Attachment {
            name: "sheet.csv".into(),
            mime: Some("text/csv".into()),
        }),
        Locator::default(),
    );

    let mut table = Table::default();
    table.caption = Some("Q3 Table".into());
    table.rows = vec![
        vec!["Month".into(), "Revenue".into()],
        vec!["July".into(), "10000".into()],
    ];
    let mut loc1 = Locator::default();
    loc1.attachment = Some("sheet.csv".into());
    ir.push(BlockKind::Table(table), loc1);

    // Anhang 2: Text/Prose
    ir.push(
        BlockKind::Boundary(Boundary::Attachment {
            name: "readme.txt".into(),
            mime: Some("text/plain".into()),
        }),
        Locator::default(),
    );
    let mut loc2 = Locator::default();
    loc2.attachment = Some("readme.txt".into());
    ir.push(
        BlockKind::Paragraph {
            text: "Some prose text in attachment.".into(),
        },
        loc2,
    );

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();
    let chunker = EmailChunker;

    let drafts = chunker.draft(&ir, &canon, &cfg).unwrap();

    assert!(!drafts.is_empty());

    let csv_draft = drafts
        .iter()
        .find(|d| d.locator.attachment.as_deref() == Some("sheet.csv"));
    assert!(csv_draft.is_some());
    assert_eq!(
        csv_draft.unwrap().breadcrumb,
        Some("Quarterly Figures > Anhang: sheet.csv".into())
    );

    let txt_draft = drafts
        .iter()
        .find(|d| d.locator.attachment.as_deref() == Some("readme.txt"));
    assert!(txt_draft.is_some());
    assert_eq!(
        txt_draft.unwrap().breadcrumb,
        Some("Quarterly Figures > Anhang: readme.txt".into())
    );
}

// ANCHOR[contextra-ingest:TST-0305]
// Testet die Funktionsfaehigkeit von `test_email_empty_message_header_only`.
#[test]
fn test_email_empty_message_header_only() {
    let mut ir = DocumentIr::new(FormatId::Eml);

    let mut msg_header = MessageHeader::default();
    msg_header.subject = Some("Empty Subject".into());
    msg_header.from = vec!["empty@example.com".into()];
    msg_header.date = Some("2026-10-31".into());

    ir.push(BlockKind::Message(msg_header), Locator::default());

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();
    let chunker = EmailChunker;

    let drafts = chunker.draft(&ir, &canon, &cfg).unwrap();

    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].locator.message, Some(1));
    assert!(drafts[0].text.contains("From: empty@example.com"));
}

proptest! {
// ANCHOR[contextra-ingest:TST-0306]
// Testet die Funktionsfaehigkeit von `prop_email_chunker_determinism_and_spans`.
    #[test]
    fn prop_email_chunker_determinism_and_spans(
        subj in ".*",
        body in ".*"
    ) {
        let mut ir = DocumentIr::new(FormatId::Eml);
        let mut msg_header = MessageHeader::default();
        msg_header.subject = Some(subj.clone());
        msg_header.from = vec!["test@example.com".into()];

        ir.push(BlockKind::Message(msg_header), Locator::default());
        ir.push(BlockKind::Paragraph { text: body.clone() }, Locator::default());

        let canon = ir.canonical();
        let cfg = ChunkingConfig::default();
        let chunker = EmailChunker;

        let res1 = chunker.draft(&ir, &canon, &cfg).unwrap();
        let res2 = chunker.draft(&ir, &canon, &cfg).unwrap();

        prop_assert_eq!(res1.len(), res2.len());
        for (d1, d2) in res1.iter().zip(res2.iter()) {
            prop_assert_eq!(&d1.text, &d2.text);
            prop_assert_eq!(&d1.span, &d2.span);
        }

        let mut prev_end = 0usize;
        for d in &res1 {
            prop_assert!(d.span.start >= prev_end);
            prop_assert!(d.span.end <= canon.text.len());
            prev_end = d.span.end;
        }
    }
}
