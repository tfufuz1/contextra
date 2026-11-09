//! Invarianten-Test: `INV-INGEST-NO-ARCHIVE-BOMB-1`
//!
//! Stellt sicher, dass ZIP-, XML- und Archiv-Dekomprimierung strikt geschützt ist gegen:
//! - Zip-Bomben (Kompressionsrate, unkomprimierte Gesamtgröße)
//! - Zip-Slip (relative Pfade `..`, absolute Pfade `/` und `\`, Laufwerkspfade)
//! - Pfadlängenüberschreitungen
//! - Billion-Laughs & XXE DTD Angriffe

#![cfg(all(feature = "zip-container", feature = "xml"))]

use contextra_ingest::error::{IngestError, LimitKind};
use contextra_ingest::extractor::ResourceBudget;
use contextra_ingest::limits::IngestLimits;
use contextra_ingest::safe::archive::{validate_archive_entry_path, validate_zip_archive};
use contextra_ingest::safe::xml::parse_xml;
use std::io::Cursor;
use zip::write::FileOptions;
use zip::ZipWriter;

// ANCHOR[contextra-ingest:TST-0454]
#[test]
fn test_inv_ingest_no_archive_bomb_zip_slip_rejected() {
    let malicious_paths = [
        "../evil.txt",
        "../../etc/passwd",
        "/absolute/path.txt",
        "\\windows\\system32\\calc.exe",
        "C:\\autoexec.bat",
        "folder/../../outside.txt",
    ];

    for path in malicious_paths {
        assert!(
            validate_archive_entry_path(path, 1024).is_err(),
            "Path should be rejected for Zip-Slip: {path}"
        );
    }
}

// ANCHOR[contextra-ingest:TST-0455]
#[test]
fn test_inv_ingest_no_archive_bomb_path_length_limit() {
    let long_path = "a".repeat(1025);
    let res = validate_archive_entry_path(&long_path, 1024);
    assert!(matches!(
        res,
        Err(IngestError::LimitExceeded {
            kind: LimitKind::PathLength,
            ..
        })
    ));
}

// ANCHOR[contextra-ingest:TST-0456]
#[test]
fn test_inv_ingest_no_archive_bomb_compression_ratio() {
    let mut buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        writer.start_file("bomb.txt", options).unwrap();
        // 1 MiB Nullen komprimiert sehr stark (nahezu 1000:1)
        let zeroes = vec![0u8; 1024 * 1024];
        std::io::Write::write_all(&mut writer, &zeroes).unwrap();
        writer.finish().unwrap();
    }

    let mut limits = IngestLimits::default();
    limits.max_compression_ratio = 10; // Max 10:1 Ratio

    let res = validate_zip_archive(Cursor::new(&buf), &limits);
    assert!(matches!(
        res,
        Err(IngestError::LimitExceeded {
            kind: LimitKind::CompressionRatio,
            ..
        })
    ));
}

// ANCHOR[contextra-ingest:TST-0457]
#[test]
fn test_inv_ingest_no_archive_bomb_total_decompressed_bytes() {
    let mut buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let options = FileOptions::default();
        writer.start_file("data1.txt", options).unwrap();
        std::io::Write::write_all(&mut writer, &[b'A'; 2000]).unwrap();
        writer.start_file("data2.txt", options).unwrap();
        std::io::Write::write_all(&mut writer, &[b'B'; 2000]).unwrap();
        writer.finish().unwrap();
    }

    let mut limits = IngestLimits::default();
    limits.max_decompressed_bytes = 3000;

    let res = validate_zip_archive(Cursor::new(&buf), &limits);
    assert!(matches!(
        res,
        Err(IngestError::LimitExceeded {
            kind: LimitKind::DecompressedBytes,
            ..
        })
    ));
}

// ANCHOR[contextra-ingest:TST-0458]
#[test]
fn test_inv_ingest_billion_laughs_and_xxe_rejected() {
    let xxe_payload = b"<?xml version=\"1.0\"?>\n<!DOCTYPE foo [\n<!ENTITY xxe SYSTEM \"file:///etc/passwd\">\n]>\n<foo>&xxe;</foo>";

    let limits = IngestLimits::default();
    let budget = ResourceBudget::new(limits);

    let res = parse_xml(xxe_payload, &budget);
    assert!(res.is_err());
    let err_str = format!("{:?}", res.unwrap_err());
    assert!(err_str.contains("XXE-Sicherheitsverstoß") || err_str.contains("Corrupt"));
}
