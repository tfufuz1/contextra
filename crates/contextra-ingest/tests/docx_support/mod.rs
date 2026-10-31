//! Integrationstest-Suite: `mod.rs`
//!
//! # Zweck & Testabdeckung
//! Verifiziert das Verhalten, Zero-Panic-Invariante und deterministische Ausgabe
//! von `contextra-ingest` fuer dieses Modul/Format gemaess Spezifikation.
//!
// FILE-CONTEXT
// ZWECK: Hilfsfunktionen zum Erzeugen von DOCX-ZIP-Dateien im Speicher für Integrationstests (WP-ING-04).
// INVARIANTEN: Zero-Panic in Testhelfern.
// STAND: TS:2026-10-31T00:00:00Z

use std::io::Write;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Erzeugt eine minimale DOCX-Datei als Byte-Vector.
pub fn build_test_docx(
    document_xml: &str,
    styles_xml: Option<&str>,
    numbering_xml: Option<&str>,
    footnotes_xml: Option<&str>,
    comments_xml: Option<&str>,
    headers_footers: &[(&str, &str)],
    core_xml: Option<&str>,
) -> Vec<u8> {
    let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = FileOptions::default();

    // 1. _rels/.rels
    let default_rels = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#;
    let _ = writer.start_file("_rels/.rels", options);
    let _ = writer.write_all(default_rels.as_bytes());

    // 2. word/_rels/document.xml.rels
    let mut doc_rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );

    if styles_xml.is_some() {
        doc_rels.push_str(r#"<Relationship Id="rIdStyle" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#);
    }
    if numbering_xml.is_some() {
        doc_rels.push_str(r#"<Relationship Id="rIdNum" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>"#);
    }
    if footnotes_xml.is_some() {
        doc_rels.push_str(r#"<Relationship Id="rIdFn" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="footnotes.xml"/>"#);
    }
    if comments_xml.is_some() {
        doc_rels.push_str(r#"<Relationship Id="rIdComm" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="comments.xml"/>"#);
    }
    for (i, (fname, _)) in headers_footers.iter().enumerate() {
        let rel_type = if fname.starts_with("header") {
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header"
        } else {
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer"
        };
        doc_rels.push_str(&format!(
            r#"<Relationship Id="rIdHF{i}" Type="{rel_type}" Target="{fname}"/>"#
        ));
    }
    doc_rels.push_str("</Relationships>");

    let _ = writer.start_file("word/_rels/document.xml.rels", options);
    let _ = writer.write_all(doc_rels.as_bytes());

    // 3. word/document.xml
    let _ = writer.start_file("word/document.xml", options);
    let _ = writer.write_all(document_xml.as_bytes());

    // 4. styles.xml
    if let Some(styles) = styles_xml {
        let _ = writer.start_file("word/styles.xml", options);
        let _ = writer.write_all(styles.as_bytes());
    }

    // 5. numbering.xml
    if let Some(num) = numbering_xml {
        let _ = writer.start_file("word/numbering.xml", options);
        let _ = writer.write_all(num.as_bytes());
    }

    // 6. footnotes.xml
    if let Some(fn_xml) = footnotes_xml {
        let _ = writer.start_file("word/footnotes.xml", options);
        let _ = writer.write_all(fn_xml.as_bytes());
    }

    // 7. comments.xml
    if let Some(comm_xml) = comments_xml {
        let _ = writer.start_file("word/comments.xml", options);
        let _ = writer.write_all(comm_xml.as_bytes());
    }

    // 8. headers / footers
    for (fname, content) in headers_footers {
        let path = format!("word/{fname}");
        let _ = writer.start_file(&path, options);
        let _ = writer.write_all(content.as_bytes());
    }

    // 9. docProps/core.xml
    let core = core_xml.unwrap_or(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>Test Dokument</dc:title>
  <dc:language>de-DE</dc:language>
  <dc:creator>Test Autor</dc:creator>
</cp:coreProperties>"#,
    );
    let _ = writer.start_file("docProps/core.xml", options);
    let _ = writer.write_all(core.as_bytes());

    match writer.finish() {
        Ok(cursor) => cursor.into_inner(),
        Err(_) => Vec::new(),
    }
}
