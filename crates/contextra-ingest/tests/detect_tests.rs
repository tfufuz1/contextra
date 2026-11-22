// FILE-CONTEXT
// ZWECK: Inhaltsbasierte Formaterkennungs-Tests für detect (WP M3 / P01).
// INVARIANTEN: Keine eingecheckten Binärdateien, Zero-Panic, deterministische Fixture-Erzeugung.
// STAND: TS:2026-10-31T00:00:00Z

use contextra_ingest::detect::{detect, Confidence, DetectHints};
use contextra_ingest::format::FormatId;

/// Testet positive Erkennung aller Formate über synthetisch generierte minimale Byte-Fixtures.
#[test]
fn test_detect_positive_fixtures() {
    let cases: Vec<(&'static str, Vec<u8>, DetectHints<'_>, FormatId)> = vec![
        (
            "PDF",
            b"%PDF-1.7\n%abc".to_vec(),
            DetectHints {
                filename: Some("doc.pdf"),
                mime: Some("application/pdf"),
            },
            FormatId::Pdf,
        ),
        (
            "RTF",
            b"{\\rtf1\\ansi\\deff0}".to_vec(),
            DetectHints {
                filename: Some("note.rtf"),
                mime: Some("application/rtf"),
            },
            FormatId::Rtf,
        ),
        (
            "OOXML DOCX",
            make_zip_fixture("word/document.xml", b"[Content_Types].xml"),
            DetectHints {
                filename: Some("letter.docx"),
                mime: None,
            },
            FormatId::Docx,
        ),
        (
            "OOXML XLSX",
            make_zip_fixture("xl/workbook.xml", b"[Content_Types].xml"),
            DetectHints {
                filename: Some("sheet.xlsx"),
                mime: None,
            },
            FormatId::Xlsx,
        ),
        (
            "OOXML PPTX",
            make_zip_fixture("ppt/presentation.xml", b"[Content_Types].xml"),
            DetectHints {
                filename: Some("slides.pptx"),
                mime: None,
            },
            FormatId::Pptx,
        ),
        (
            "ODF ODT",
            make_odf_mimetype_fixture("application/vnd.oasis.opendocument.text"),
            DetectHints {
                filename: Some("text.odt"),
                mime: None,
            },
            FormatId::Odt,
        ),
        (
            "ODF ODS",
            make_odf_mimetype_fixture("application/vnd.oasis.opendocument.spreadsheet"),
            DetectHints {
                filename: Some("data.ods"),
                mime: None,
            },
            FormatId::Ods,
        ),
        (
            "ODF ODP",
            make_odf_mimetype_fixture("application/vnd.oasis.opendocument.presentation"),
            DetectHints {
                filename: Some("pres.odp"),
                mime: None,
            },
            FormatId::Odp,
        ),
        (
            "EPUB",
            make_odf_mimetype_fixture("application/epub+zip"),
            DetectHints {
                filename: Some("book.epub"),
                mime: None,
            },
            FormatId::Epub,
        ),
        (
            "OLE DOC",
            make_ole_fixture(b"WordDocument"),
            DetectHints {
                filename: Some("legacy.doc"),
                mime: None,
            },
            FormatId::Doc,
        ),
        (
            "OLE XLS",
            make_ole_fixture(b"Workbook"),
            DetectHints {
                filename: Some("legacy.xls"),
                mime: None,
            },
            FormatId::Xls,
        ),
        (
            "OLE PPT",
            make_ole_fixture(b"PowerPoint Document"),
            DetectHints {
                filename: Some("legacy.ppt"),
                mime: None,
            },
            FormatId::Ppt,
        ),
        (
            "OLE MSG",
            make_ole_fixture(b"__substg1.0_0001001E"),
            DetectHints {
                filename: Some("mail.msg"),
                mime: None,
            },
            FormatId::Msg,
        ),
        (
            "HTML",
            b"<!DOCTYPE html><html><head></head><body></body></html>".to_vec(),
            DetectHints {
                filename: Some("index.html"),
                mime: None,
            },
            FormatId::Html,
        ),
        (
            "XHTML",
            b"<html xmlns=\"http://www.w3.org/1999/xhtml\"><head></head></html>".to_vec(),
            DetectHints {
                filename: Some("page.xhtml"),
                mime: None,
            },
            FormatId::Xhtml,
        ),
        (
            "XML",
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><root></root>".to_vec(),
            DetectHints {
                filename: Some("data.xml"),
                mime: None,
            },
            FormatId::Xml,
        ),
        (
            "JSON",
            b"{\"name\": \"test\", \"value\": 42}".to_vec(),
            DetectHints {
                filename: Some("config.json"),
                mime: None,
            },
            FormatId::Json,
        ),
        (
            "JSONL",
            b"{\"a\": 1}\n{\"b\": 2}\n".to_vec(),
            DetectHints {
                filename: Some("logs.jsonl"),
                mime: None,
            },
            FormatId::Jsonl,
        ),
        (
            "MBOX",
            b"From sender@example.com Mon Jan 1 00:00:00 2026\nSubject: Hello".to_vec(),
            DetectHints {
                filename: Some("inbox.mbox"),
                mime: None,
            },
            FormatId::Mbox,
        ),
        (
            "EML",
            b"From: sender@example.com\nTo: recipient@example.com\nSubject: Test\n\nBody".to_vec(),
            DetectHints {
                filename: Some("mail.eml"),
                mime: None,
            },
            FormatId::Eml,
        ),
        (
            "TEX",
            b"\\documentclass{article}\n\\begin{document}\nHello\\end{document}".to_vec(),
            DetectHints {
                filename: Some("paper.tex"),
                mime: None,
            },
            FormatId::Tex,
        ),
    ];

    for (name, bytes, hints, expected_fmt) in cases {
        let det = detect(&bytes, &hints).unwrap_or_else(|e| panic!("Failed on {name}: {e:?}"));
        assert_eq!(
            det.format, expected_fmt,
            "Format mismatch on test case '{name}'"
        );
        assert_eq!(
            det.confidence,
            Confidence::High,
            "High confidence expected for matching magic and hint on '{name}'"
        );
    }
}

/// Testet Widerspruchsfälle: Content Magic gewinnt vor falscher Dateiendung/MIME.
#[test]
fn test_detect_contradiction_magic_overrides_hint() {
    let zip_docx = make_zip_fixture("word/document.xml", b"[Content_Types].xml");
    let hints_txt = DetectHints {
        filename: Some("renamed_document.txt"),
        mime: Some("text/plain"),
    };

    let det = detect(&zip_docx, &hints_txt).expect("Detection should succeed");
    assert_eq!(
        det.format,
        FormatId::Docx,
        "Magic content must override false .txt extension"
    );
    assert_eq!(
        det.confidence,
        Confidence::Medium,
        "Contradiction results in Medium confidence"
    );
    assert!(
        det.reason.contains("override"),
        "Reason must document the override mismatch"
    );
}

/// Testet Erkennung ohne Hints (nur aus Content-Bytes).
#[test]
fn test_detect_content_only_no_hints() {
    let pdf_bytes = b"%PDF-2.0\n1 0 obj...";
    let det = detect(pdf_bytes, &DetectHints::default()).expect("Detection should succeed");
    assert_eq!(det.format, FormatId::Pdf);
    assert_eq!(det.confidence, Confidence::Medium);
}

/// Testet Edge-Cases: Leere Eingabe, 1-Byte Eingabe, abgeschnittener Header.
#[test]
fn test_detect_edge_cases() {
    // 1. Leere Eingabe ohne Hints -> Fehler
    let err_empty = detect(&[], &DetectHints::default());
    assert!(err_empty.is_err(), "Empty input without hints must fail");

    // 2. Leere Eingabe mit Hint -> Extension match (Low per Rule 2: Nur Endung/MIME -> Low)
    let det_empty_hint = detect(
        &[],
        &DetectHints {
            filename: Some("file.pdf"),
            mime: None,
        },
    )
    .unwrap();
    assert_eq!(det_empty_hint.format, FormatId::Pdf);
    assert_eq!(det_empty_hint.confidence, Confidence::Low);

    // 3. 1-Byte Eingabe
    let det_1byte = detect(
        b"X",
        &DetectHints {
            filename: Some("data.csv"),
            mime: None,
        },
    )
    .unwrap();
    assert_eq!(det_1byte.format, FormatId::Csv);

    // 4. Abgeschnittener ZIP Header (2 Bytes Magic: PK) mit docx Hint
    let det_trunc = detect(
        b"PK",
        &DetectHints {
            filename: Some("doc.docx"),
            mime: None,
        },
    )
    .unwrap();
    assert_eq!(det_trunc.format, FormatId::Docx);
    assert_eq!(det_trunc.confidence, Confidence::Low);
}

/// Generiert synthetisches ZIP-Lokalheader Fixture für OOXML.
fn make_zip_fixture(entry_name_1: &str, entry_name_2: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();

    // Eintrag 1 (z. B. "word/document.xml")
    buf.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04]); // Magic
    buf.extend_from_slice(&[0u8; 22]); // Header Felddaten
    buf.extend_from_slice(&(entry_name_1.len() as u16).to_le_bytes()); // fn_len
    buf.extend_from_slice(&0u16.to_le_bytes()); // extra_len
    buf.extend_from_slice(entry_name_1.as_bytes());

    // Eintrag 2 (z. B. "[Content_Types].xml")
    buf.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04]); // Magic
    buf.extend_from_slice(&[0u8; 22]); // Header Felddaten
    buf.extend_from_slice(&(entry_name_2.len() as u16).to_le_bytes()); // fn_len
    buf.extend_from_slice(&0u16.to_le_bytes()); // extra_len
    buf.extend_from_slice(entry_name_2);

    buf
}

/// Generiert synthetisches ODF/EPUB `mimetype` ZIP Fixture.
fn make_odf_mimetype_fixture(mimetype: &str) -> Vec<u8> {
    let mut buf = Vec::new();

    buf.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04]); // Magic
    buf.extend_from_slice(&[0u8; 4]); // Version + flags
    buf.extend_from_slice(&0u16.to_le_bytes()); // comp_method = 0 (stored)
    buf.extend_from_slice(&[0u8; 8]); // time + crc
    buf.extend_from_slice(&(mimetype.len() as u32).to_le_bytes()); // comp_size
    buf.extend_from_slice(&(mimetype.len() as u32).to_le_bytes()); // uncomp_size
    buf.extend_from_slice(&8u16.to_le_bytes()); // fn_len ("mimetype" = 8)
    buf.extend_from_slice(&0u16.to_le_bytes()); // extra_len = 0
    buf.extend_from_slice(b"mimetype");
    buf.extend_from_slice(mimetype.as_bytes());

    buf
}

/// Generiert synthetisches OLE2 Header Fixture mit UTF-16LE Stream-Namen.
fn make_ole_fixture(utf8_stream_name: &[u8]) -> Vec<u8> {
    let mut buf = vec![0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]; // OLE2 Magic
    buf.extend_from_slice(&[0u8; 500]); // Dummy OLE header bytes

    // Schreibe UTF-16LE Repräsentation des Stream-Namens
    for &b in utf8_stream_name {
        buf.push(b);
        buf.push(0);
    }

    buf
}
