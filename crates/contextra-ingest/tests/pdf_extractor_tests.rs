// FILE-CONTEXT
// ZWECK: Integrationstests für PdfExtractor (WP ING-08).
// INVARIANTEN: #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)],
//              Programmatische Erzeugung von PDF-Fixtures mit lopdf.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_ingest::error::IngestError;
use contextra_ingest::extractor::{DocumentExtractor, ExtractContext, ResourceBudget};
use contextra_ingest::format::FormatId;
use contextra_ingest::formats::pdf::PdfExtractor;
use contextra_ingest::ir::{BlockKind, Boundary, NoteKind};
use contextra_ingest::limits::{ExtractPolicy, IngestLimits};
use contextra_ingest::registry::ExtractorRegistry;
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};

/// Hilfsfunktion zum Erzeugen einer minimalen PDF-Datei im Speicher.
fn create_test_pdf(
    pages_text: &[&str],
    title: Option<&str>,
    author: Option<&str>,
    with_outline: bool,
) -> Vec<u8> {
    let mut doc = Document::new();

    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
        "Encoding" => "WinAnsiEncoding",
    });

    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });

    let mut page_ids = Vec::new();

    for &text in pages_text {
        let lines: Vec<&str> = text.split('\n').collect();
        let mut ops = Vec::new();

        for line in lines {
            ops.push(Operation::new("BT", vec![]));
            ops.push(Operation::new("Tf", vec!["F1".into(), 12.into()]));
            ops.push(Operation::new("Td", vec![100.into(), 700.into()]));
            ops.push(Operation::new(
                "Tj",
                vec![Object::String(
                    line.as_bytes().to_vec(),
                    lopdf::StringFormat::Literal,
                )],
            ));
            ops.push(Operation::new("ET", vec![]));
        }

        let content = Content { operations: ops };

        let content_bytes = content.encode().unwrap_or_default();
        let content_id = doc.add_object(Stream::new(dictionary! {}, content_bytes));

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
        });

        page_ids.push(page_id);
    }

    let count = u32::try_from(page_ids.len()).unwrap_or(0);
    let kids: Vec<Object> = page_ids.iter().map(|&id| Object::Reference(id)).collect();

    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => count,
        }),
    );

    let mut catalog_dict = dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    };

    if with_outline && !page_ids.is_empty() {
        let outline_item_id = doc.new_object_id();
        let outlines_id = doc.add_object(dictionary! {
            "Type" => "Outlines",
            "First" => outline_item_id,
            "Last" => outline_item_id,
            "Count" => 1,
        });

        doc.objects.insert(
            outline_item_id,
            Object::Dictionary(dictionary! {
                "Title" => Object::String("Test Kapitel 1".as_bytes().to_vec(), lopdf::StringFormat::Literal),
                "Parent" => outlines_id,
                "Dest" => vec![page_ids[0].into(), "Fit".into()],
            }),
        );

        catalog_dict.set("Outlines", outlines_id);
    }

    let catalog_id = doc.add_object(catalog_dict);
    doc.trailer.set("Root", catalog_id);

    let mut info_dict = dictionary! {};
    if let Some(t) = title {
        info_dict.set(
            "Title",
            Object::String(t.as_bytes().to_vec(), lopdf::StringFormat::Literal),
        );
    }
    if let Some(a) = author {
        info_dict.set(
            "Author",
            Object::String(a.as_bytes().to_vec(), lopdf::StringFormat::Literal),
        );
    }
    if !info_dict.is_empty() {
        let info_id = doc.add_object(info_dict);
        doc.trailer.set("Info", info_id);
    }

    let mut buffer = Vec::new();
    doc.save_to(&mut buffer).unwrap();
    buffer
}

#[test]
fn test_pdf_extractor_basic_multi_page() {
    let pdf_bytes = create_test_pdf(
        &["Hallo Welt auf Seite 1", "Zweite Seite Inhalt"],
        Some("Dokumenten Titel"),
        Some("Max Mustermann"),
        true,
    );

    let extractor = PdfExtractor::new();
    let limits = IngestLimits::default();
    let mut policy = ExtractPolicy::default();
    policy.keep_personal_metadata = true;

    let budget = ResourceBudget::new(limits.clone());
    let registry = ExtractorRegistry::new_empty();
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &registry,
        filename: Some("test.pdf"),
    };

    let ir = extractor.extract(FormatId::Pdf, &pdf_bytes, &ctx).unwrap();

    assert_eq!(ir.meta.title.as_deref(), Some("Dokumenten Titel"));
    assert_eq!(ir.meta.authors, vec!["Max Mustermann".to_string()]);
    assert_eq!(ir.meta.page_count, Some(2));

    let pages: Vec<u32> = ir
        .blocks
        .iter()
        .filter_map(|b| match b.kind {
            BlockKind::Boundary(Boundary::Page { number }) => Some(number),
            _ => None,
        })
        .collect();

    assert_eq!(pages, vec![1, 2]);
}

#[test]
fn test_pdf_extractor_headers_footers_policy() {
    let pages_text = vec![
        "Kopfzeile Firma\nInhalt A",
        "Kopfzeile Firma\nInhalt B",
        "Kopfzeile Firma\nInhalt C",
        "Kopfzeile Firma\nInhalt D",
        "Kopfzeile Firma\nInhalt E",
    ];
    let pdf_bytes = create_test_pdf(&pages_text, None, None, false);

    let extractor = PdfExtractor::new();
    let limits = IngestLimits::default();
    let registry = ExtractorRegistry::new_empty();

    // 1. Policy include_headers_footers = false (Standard)
    let mut policy_off = ExtractPolicy::default();
    policy_off.include_headers_footers = false;

    let budget1 = ResourceBudget::new(limits.clone());
    let ctx1 = ExtractContext {
        limits: &limits,
        policy: &policy_off,
        budget: &budget1,
        registry: &registry,
        filename: None,
    };

    let ir1 = extractor.extract(FormatId::Pdf, &pdf_bytes, &ctx1).unwrap();
    let has_notes = ir1
        .blocks
        .iter()
        .any(|b| matches!(b.kind, BlockKind::Note { .. }));
    assert!(!has_notes);

    // 2. Policy include_headers_footers = true
    let mut policy_on = ExtractPolicy::default();
    policy_on.include_headers_footers = true;
    let budget2 = ResourceBudget::new(limits.clone());
    let ctx2 = ExtractContext {
        limits: &limits,
        policy: &policy_on,
        budget: &budget2,
        registry: &registry,
        filename: None,
    };

    let ir2 = extractor.extract(FormatId::Pdf, &pdf_bytes, &ctx2).unwrap();
    let header_notes: Vec<&str> = ir2
        .blocks
        .iter()
        .filter_map(|b| match &b.kind {
            BlockKind::Note {
                kind: NoteKind::Header,
                text,
            } => Some(text.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(header_notes, vec!["Kopfzeile Firma"]);
}

#[test]
fn test_pdf_extractor_no_extractable_text() {
    let pdf_bytes = create_test_pdf(&["", "   "], None, None, false);

    let extractor = PdfExtractor::new();
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let registry = ExtractorRegistry::new_empty();

    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &registry,
        filename: None,
    };

    let err = extractor
        .extract(FormatId::Pdf, &pdf_bytes, &ctx)
        .unwrap_err();
    assert_eq!(
        err,
        IngestError::NoExtractableText {
            format: FormatId::Pdf
        }
    );
}

#[test]
fn test_pdf_extractor_corrupt_bytes() {
    let corrupt_bytes = b"%PDF-1.4 corrupt content bad xref table %EOF";

    let extractor = PdfExtractor::new();
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let registry = ExtractorRegistry::new_empty();

    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &registry,
        filename: None,
    };

    let err = extractor
        .extract(FormatId::Pdf, corrupt_bytes, &ctx)
        .unwrap_err();
    assert!(matches!(err, IngestError::Corrupt { .. }));
}
