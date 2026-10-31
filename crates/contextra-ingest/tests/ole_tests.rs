//! Integrationstest-Suite: `ole_tests.rs`
//!
//! # Zweck & Testabdeckung
//! Verifiziert das Verhalten, Zero-Panic-Invariante und deterministische Ausgabe
//! von `contextra-ingest` fuer dieses Modul/Format gemaess Spezifikation.
//!
// FILE-CONTEXT
// ZWECK: Golden & Contract Tests für OLE Formate (Doc, Ppt, Msg) (WP M4).
// INVARIANTEN: Zero-Panic, deterministische Extraktion, Limit- und Fehlerprüfungen.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod ole_support;

use contextra_ingest::error::IngestError;
use contextra_ingest::extractor::{DocumentExtractor, ExtractContext, ResourceBudget};
use contextra_ingest::format::FormatId;
use contextra_ingest::formats::ole::OleExtractor;
use contextra_ingest::ir::{BlockKind, Boundary, NoteKind};
use contextra_ingest::limits::{ExtractPolicy, IngestLimits};
use contextra_ingest::registry::ExtractorRegistry;

fn create_test_context(limits: &IngestLimits) -> (ResourceBudget, ExtractorRegistry) {
    let budget = ResourceBudget::new(limits.clone());
    let registry = ExtractorRegistry::with_defaults();
    (budget, registry)
}

// ANCHOR[contextra-ingest:TST-0390]
// Testet die Funktionsfaehigkeit von `test_doc_golden_ansi_utf16_field_code`.
#[test]
fn test_doc_golden_ansi_utf16_field_code() {
    let doc_bytes = ole_support::create_minimal_doc(
        "Haupttext ANSI mit ÄÖÜ ",
        "UTF-16 Teil mit Umlauten äöüß",
        "HYPERLINK \"http://example.com\"",
        false,
    );

    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let (budget, registry) = create_test_context(&limits);
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &registry,
        filename: Some("test.doc"),
    };

    let extractor = OleExtractor::new();
    let ir = extractor
        .extract(FormatId::Doc, &doc_bytes, &ctx)
        .expect("DOC extraction failed");

    assert_eq!(ir.format, FormatId::Doc);
    assert!(!ir.blocks.is_empty());

    let full_text: String = ir
        .blocks
        .iter()
        .filter_map(|b| match &b.kind {
            BlockKind::Paragraph { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");

    assert!(full_text.contains("Haupttext ANSI"));
    assert!(full_text.contains("UTF-16 Teil mit Umlauten"));

    assert!(!full_text.contains("HYPERLINK"));
    assert!(full_text.contains("DisplayResult"));
}

// ANCHOR[contextra-ingest:TST-0391]
// Testet die Funktionsfaehigkeit von `test_ppt_golden_slides_and_notes`.
#[test]
fn test_ppt_golden_slides_and_notes() {
    let slides = [
        ("Erste Folie: Titel", "Inhalt der ersten Folie"),
        ("Zweite Folie: Übersicht", "Inhalt der zweiten Folie"),
    ];
    let notes = ["Sprechernotiz zu Folie 1"];

    let ppt_bytes = ole_support::create_minimal_ppt(&slides, &notes, false);

    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let (budget, registry) = create_test_context(&limits);
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &registry,
        filename: Some("presentation.ppt"),
    };

    let extractor = OleExtractor::new();
    let ir = extractor
        .extract(FormatId::Ppt, &ppt_bytes, &ctx)
        .expect("PPT extraction failed");

    assert_eq!(ir.format, FormatId::Ppt);

    let slide_boundaries: Vec<_> = ir
        .blocks
        .iter()
        .filter_map(|b| match &b.kind {
            BlockKind::Boundary(Boundary::Slide { number, title }) => {
                Some((*number, title.clone()))
            }
            _ => None,
        })
        .collect();

    assert_eq!(slide_boundaries.len(), 2);
    assert_eq!(slide_boundaries[0].0, 1);
    assert_eq!(slide_boundaries[0].1.as_deref(), Some("Erste Folie: Titel"));

    let speaker_notes: Vec<_> = ir
        .blocks
        .iter()
        .filter_map(|b| match &b.kind {
            BlockKind::Note {
                kind: NoteKind::SpeakerNotes,
                text,
            } => Some(text.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(speaker_notes.len(), 1);
    assert!(speaker_notes[0].contains("Sprechernotiz"));
}

// ANCHOR[contextra-ingest:TST-0392]
// Testet die Funktionsfaehigkeit von `test_msg_golden_subject_sender_body_attachment`.
#[test]
fn test_msg_golden_subject_sender_body_attachment() {
    let attach_doc_bytes =
        ole_support::create_minimal_doc("Inhalt des angehängten Word-Dokuments", "", "", false);
    let msg_bytes = ole_support::create_minimal_msg(
        "Projekt-Update Q4",
        "Alice Smith",
        "Hallo Team,\nAnbei der Bericht.",
        Some("bericht.doc"),
        Some(&attach_doc_bytes),
    );

    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let (budget, registry) = create_test_context(&limits);
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &registry,
        filename: Some("email.msg"),
    };

    let extractor = OleExtractor::new();
    let ir = extractor
        .extract(FormatId::Msg, &msg_bytes, &ctx)
        .expect("MSG extraction failed");

    assert_eq!(ir.format, FormatId::Msg);

    let header_block = ir.blocks.first().expect("Header block missing");
    if let BlockKind::Message(hdr) = &header_block.kind {
        assert_eq!(hdr.subject.as_deref(), Some("Projekt-Update Q4"));
        assert_eq!(hdr.from.first().map(String::as_str), Some("Alice Smith"));
    } else {
        panic!("First block must be Message header");
    }

    let body_text: String = ir
        .blocks
        .iter()
        .filter_map(|b| match &b.kind {
            BlockKind::Paragraph { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(body_text.contains("Hallo Team"));

    let attachment_boundaries: Vec<_> = ir
        .blocks
        .iter()
        .filter_map(|b| match &b.kind {
            BlockKind::Boundary(Boundary::Attachment { name, .. }) => Some(name.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(attachment_boundaries.len(), 1);
    assert_eq!(attachment_boundaries[0], "bericht.doc");
}

// ANCHOR[contextra-ingest:TST-0393]
// Testet die Funktionsfaehigkeit von `test_msg_nested_msg_attachment`.
#[test]
fn test_msg_nested_msg_attachment() {
    let child_msg = ole_support::create_minimal_msg(
        "Verschachtelte E-Mail",
        "Bob Jones",
        "Details zur Unteraufgabe.",
        None,
        None,
    );
    let parent_msg = ole_support::create_minimal_msg(
        "Hauptnachricht mit Fwd",
        "Alice Smith",
        "Siehe weitergeleitete E-Mail im Anhang.",
        Some("fwd.msg"),
        Some(&child_msg),
    );

    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let (budget, registry) = create_test_context(&limits);
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &registry,
        filename: Some("parent.msg"),
    };

    let extractor = OleExtractor::new();
    let ir = extractor
        .extract(FormatId::Msg, &parent_msg, &ctx)
        .expect("Parent MSG extraction failed");

    assert_eq!(ir.format, FormatId::Msg);

    // Verify attachment boundary for nested MSG
    let attachment_boundaries: Vec<_> = ir
        .blocks
        .iter()
        .filter_map(|b| match &b.kind {
            BlockKind::Boundary(Boundary::Attachment { name, .. }) => Some(name.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(attachment_boundaries.len(), 1);
    assert_eq!(attachment_boundaries[0], "fwd.msg");

    // Verify nested message body text is extracted in ir
    let full_text: String = ir
        .blocks
        .iter()
        .filter_map(|b| match &b.kind {
            BlockKind::Paragraph { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");

    assert!(full_text.contains("Details zur Unteraufgabe"));
}

// ANCHOR[contextra-ingest:TST-0394]
// Testet die Funktionsfaehigkeit von `test_encrypted_doc_and_ppt_returns_encrypted_error`.
#[test]
fn test_encrypted_doc_and_ppt_returns_encrypted_error() {
    let encrypted_doc = ole_support::create_minimal_doc("Geheim", "", "", true);
    let encrypted_ppt = ole_support::create_minimal_ppt(&[("Geheim", "")], &[], true);

    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let (budget, registry) = create_test_context(&limits);
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &registry,
        filename: None,
    };

    let extractor = OleExtractor::new();

    let doc_res = extractor.extract(FormatId::Doc, &encrypted_doc, &ctx);
    assert!(matches!(
        doc_res,
        Err(IngestError::Encrypted {
            format: Some(FormatId::Doc)
        })
    ));

    let ppt_res = extractor.extract(FormatId::Ppt, &encrypted_ppt, &ctx);
    assert!(matches!(
        ppt_res,
        Err(IngestError::Encrypted {
            format: Some(FormatId::Ppt)
        })
    ));
}

// ANCHOR[contextra-ingest:TST-0395]
// Testet die Funktionsfaehigkeit von `generate_fuzz_corpus_seeds`.
#[test]
fn generate_fuzz_corpus_seeds() {
    use std::fs;
    let corpus_dir = std::path::Path::new("fuzz/corpus/fuzz_ole");
    let _ = fs::create_dir_all(corpus_dir);

    let doc1 = ole_support::create_minimal_doc("Simple doc", "", "", false);
    let doc2 = ole_support::create_minimal_doc("ANSI text", "UTF16 text ÄÖÜ", "CODE", false);
    let doc3 = ole_support::create_minimal_doc("Encrypted", "", "", true);

    let ppt1 = ole_support::create_minimal_ppt(&[("Title 1", "Body 1")], &[], false);
    let ppt2 = ole_support::create_minimal_ppt(&[("T1", "B1"), ("T2", "B2")], &["Note 1"], false);
    let ppt3 = ole_support::create_minimal_ppt(&[("Encrypted", "")], &[], true);

    let msg1 = ole_support::create_minimal_msg("Subject 1", "Sender 1", "Body 1", None, None);
    let msg2 = ole_support::create_minimal_msg(
        "Subject 2",
        "Sender 2",
        "Body 2",
        Some("file.doc"),
        Some(&doc1),
    );

    let mut seeds: Vec<Vec<u8>> = Vec::new();

    // Format 0 (DOC)
    for doc in [&doc1, &doc2, &doc3] {
        let mut seed = vec![0u8];
        seed.extend_from_slice(doc);
        seeds.push(seed);
    }

    // Format 1 (PPT)
    for ppt in [&ppt1, &ppt2, &ppt3] {
        let mut seed = vec![1u8];
        seed.extend_from_slice(ppt);
        seeds.push(seed);
    }

    // Format 2 (MSG)
    for msg in [&msg1, &msg2] {
        let mut seed = vec![2u8];
        seed.extend_from_slice(msg);
        seeds.push(seed);
    }

    // Random noise / corrupt
    seeds.push(vec![0u8, 0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A]);
    seeds.push(vec![1u8, 0x00, 0x01, 0x02, 0x03, 0x04]);
    seeds.push(vec![2u8, 0xFF, 0xFE, 0xFD, 0xFC, 0xFB]);
    seeds.push(vec![0u8]);

    for (idx, seed) in seeds.iter().enumerate() {
        let file_path = corpus_dir.join(format!("seed_{:02}.bin", idx));
        fs::write(file_path, seed).unwrap();
    }
}

// ANCHOR[contextra-ingest:TST-0396]
// Testet die Funktionsfaehigkeit von `test_corrupt_cfb_returns_corrupt_error_without_panic`.
#[test]
fn test_corrupt_cfb_returns_corrupt_error_without_panic() {
    let corrupt_bytes = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];

    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let (budget, registry) = create_test_context(&limits);
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &registry,
        filename: None,
    };

    let extractor = OleExtractor::new();

    let doc_res = extractor.extract(FormatId::Doc, &corrupt_bytes, &ctx);
    assert!(matches!(doc_res, Err(IngestError::Corrupt { .. })));

    let ppt_res = extractor.extract(FormatId::Ppt, &corrupt_bytes, &ctx);
    assert!(matches!(ppt_res, Err(IngestError::Corrupt { .. })));

    let msg_res = extractor.extract(FormatId::Msg, &corrupt_bytes, &ctx);
    assert!(matches!(msg_res, Err(IngestError::Corrupt { .. })));
}
