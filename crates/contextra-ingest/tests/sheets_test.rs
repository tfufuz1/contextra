// FILE-CONTEXT
// ZWECK: Golden- & DoS-Integrationstests für SheetsExtractor (Xlsx, Ods, Csv, Tsv).
// INVARIANTEN: All-features Lints einhalten, Test-First, programmatic fixture generation.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use contextra_ingest::error::{IngestError, LimitKind};
use contextra_ingest::extractor::ExtractContext;
use contextra_ingest::format::FormatId;
use contextra_ingest::limits::{ExtractPolicy, IngestLimits};
use contextra_ingest::pipeline::{ingest, IngestConfig, IngestRequest};
use contextra_ingest::registry::ExtractorRegistry;
use contextra_types::DocId;
use std::io::{Cursor, Write};
use zip::write::FileOptions;
use zip::ZipWriter;

fn create_test_xlsx() -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let opt = FileOptions::default();

        zip.start_file("[Content_Types].xml", opt).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/worksheets/sheet2.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#,
        )
        .unwrap();

        zip.start_file("_rels/.rels", opt).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#,
        )
        .unwrap();

        zip.start_file("xl/workbook.xml", opt).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="Blatt1" sheetId="1" r:id="rId1"/>
    <sheet name="Versteckt" sheetId="2" state="hidden" r:id="rId2"/>
  </sheets>
</workbook>"#,
        )
        .unwrap();

        zip.start_file("xl/_rels/workbook.xml.rels", opt).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/>
</Relationships>"#,
        )
        .unwrap();

        zip.start_file("xl/worksheets/sheet1.xml", opt).unwrap();
        zip.write_all(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dimension ref="A1:C5"/>
  <sheetData>
    <row r="1">
      <c r="A1" t="str"><v>Titel</v></c>
      <c r="B1" t="str"><v>Wert</v></c>
      <c r="C1" t="str"><v>Notiz</v></c>
    </row>
    <row r="2">
      <c r="A2"><v>100</v></c>
      <c r="B2"><v>45.67</v></c>
      <c r="C2" t="b"><v>1</v></c>
    </row>
    <row r="4">
      <c r="A4" t="e"><v>#DIV/0!</v></c>
      <c r="B4" t="str"><v>München</v></c>
      <c r="C4" t="str"><v>Groß</v></c>
    </row>
  </sheetData>
  <mergeCells count="1">
    <mergeCell ref="A1:B1"/>
  </mergeCells>
</worksheet>"#
                .as_bytes(),
        )
        .unwrap();

        zip.start_file("xl/worksheets/sheet2.xml", opt).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dimension ref="A1:A1"/>
  <sheetData>
    <row r="1"><c r="A1" t="str"><v>Geheim</v></c></row>
  </sheetData>
</worksheet>"#,
        )
        .unwrap();

        zip.finish().unwrap();
    }
    buf
}

fn create_sparse_xlsx() -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let opt = FileOptions::default();

        zip.start_file("[Content_Types].xml", opt).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#,
        )
        .unwrap();

        zip.start_file("_rels/.rels", opt).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#,
        )
        .unwrap();

        zip.start_file("xl/workbook.xml", opt).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="Sparse" sheetId="1" r:id="rId1"/>
  </sheets>
</workbook>"#,
        )
        .unwrap();

        zip.start_file("xl/_rels/workbook.xml.rels", opt).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#,
        )
        .unwrap();

        zip.start_file("xl/worksheets/sheet1.xml", opt).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dimension ref="A1:XFD1048576"/>
  <sheetData>
    <row r="1"><c r="A1"><v>1</v></c></row>
    <row r="1048576"><c r="XFD1048576"><v>2</v></c></row>
  </sheetData>
</worksheet>"#,
        )
        .unwrap();

        zip.finish().unwrap();
    }
    buf
}

#[test]
fn test_golden_xlsx_extraction() {
    let reg = ExtractorRegistry::with_defaults();
    let xlsx_bytes = create_test_xlsx();

    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &reg,
        filename: Some("test.xlsx"),
    };

    let ir = reg
        .extract(FormatId::Xlsx, &xlsx_bytes, &ctx)
        .expect("XLSX Extraction must succeed");
    let canon = ir.canonical();

    assert!(canon.text.contains("Titel"));
    assert!(canon.text.contains("München"));
}

#[test]
fn test_golden_hidden_sheet_policy() {
    let reg = ExtractorRegistry::with_defaults();
    let xlsx_bytes = create_test_xlsx();

    let limits = IngestLimits::default();

    // Default policy (include_hidden = false)
    let mut policy1 = ExtractPolicy::default();
    policy1.include_hidden = false;
    let budget1 = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx1 = ExtractContext {
        limits: &limits,
        policy: &policy1,
        budget: &budget1,
        registry: &reg,
        filename: Some("test.xlsx"),
    };

    let ir1 = reg
        .extract(FormatId::Xlsx, &xlsx_bytes, &ctx1)
        .expect("Default policy should exclude hidden sheet");
    let canon1 = ir1.canonical();
    assert!(!canon1.text.contains("Geheim"));

    // Policy with include_hidden = true
    let mut policy2 = ExtractPolicy::default();
    policy2.include_hidden = true;
    let budget2 = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx2 = ExtractContext {
        limits: &limits,
        policy: &policy2,
        budget: &budget2,
        registry: &reg,
        filename: Some("test.xlsx"),
    };

    let ir2 = reg
        .extract(FormatId::Xlsx, &xlsx_bytes, &ctx2)
        .expect("Policy with include_hidden should extract hidden sheet");
    let canon2 = ir2.canonical();
    assert!(canon2.text.contains("Geheim"));
}

#[test]
fn test_golden_csv_semicolon_and_quotes() {
    let csv_data = "Name;Ort;Notiz\n\"Müller, Hans\";\"München\";\"Zeile 1\nZeile 2\"\n";

    let reg = ExtractorRegistry::with_defaults();
    let req = IngestRequest {
        bytes: csv_data.as_bytes(),
        filename: Some("data.csv"),
        mime: Some("text/csv"),
        format_override: Some(FormatId::Csv),
        doc_id: DocId::new(3),
    };

    let cfg = IngestConfig::default();
    let out = ingest(&req, &cfg, &reg).expect("CSV extraction must succeed");

    let combined_chunks: String = out
        .chunks
        .iter()
        .map(|c| c.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(combined_chunks.contains("Müller, Hans"));
    assert!(combined_chunks.contains("München"));
}

#[test]
fn test_golden_tsv_and_windows_1252() {
    let win1252_bytes: Vec<u8> = vec![
        0x53, 0x74, 0x61, 0x64, 0x74, 0x09, 0x54, 0x65, 0x78, 0x74, 0x0A, 0x4D, 0xFC, 0x6E, 0x63,
        0x68, 0x65, 0x6E, 0x09, 0x47, 0x72, 0x6F, 0xDF, 0x0A,
    ];

    let reg = ExtractorRegistry::with_defaults();
    let req = IngestRequest {
        bytes: &win1252_bytes,
        filename: Some("win1252.tsv"),
        mime: None,
        format_override: Some(FormatId::Tsv),
        doc_id: DocId::new(4),
    };

    let cfg = IngestConfig::default();
    let out = ingest(&req, &cfg, &reg).expect("Windows-1252 TSV extraction must succeed");

    let combined_chunks: String = out
        .chunks
        .iter()
        .map(|c| c.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(combined_chunks.contains("München"));
    assert!(combined_chunks.contains("Groß"));
}

#[test]
fn test_dos_sparse_sheet_triggers_limit_exceeded() {
    let reg = ExtractorRegistry::with_defaults();
    let sparse_bytes = create_sparse_xlsx();

    let mut limits = IngestLimits::default();
    limits.max_table_cells = 100_000;

    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &reg,
        filename: Some("sparse.xlsx"),
    };

    let err = reg
        .extract(FormatId::Xlsx, &sparse_bytes, &ctx)
        .expect_err("Sparse sheet must be rejected fail-closed");
    assert!(matches!(
        err,
        IngestError::LimitExceeded {
            kind: LimitKind::TableCells,
            ..
        }
    ));
}

#[test]
fn test_dos_zip_bomb_triggers_limit_exceeded() {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let opt = FileOptions::default();
        for i in 0..105 {
            zip.start_file(format!("file_{i}.txt"), opt).unwrap();
            zip.write_all(b"dummy data").unwrap();
        }
        zip.finish().unwrap();
    }

    let reg = ExtractorRegistry::with_defaults();
    let mut limits = IngestLimits::default();
    limits.max_archive_entries = 100;

    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &reg,
        filename: Some("bomb.xlsx"),
    };

    let err = reg
        .extract(FormatId::Xlsx, &buf, &ctx)
        .expect_err("Zip bomb entries must be rejected");
    assert!(matches!(
        err,
        IngestError::LimitExceeded {
            kind: LimitKind::ArchiveEntries,
            ..
        }
    ));
}

#[test]
fn test_csv_injection_prefixes_preserved() {
    let csv_data = "Formula,Prefix\n=1+1,+2\n-3,@mention\n";

    let reg = ExtractorRegistry::with_defaults();
    let req = IngestRequest {
        bytes: csv_data.as_bytes(),
        filename: Some("injection.csv"),
        mime: None,
        format_override: Some(FormatId::Csv),
        doc_id: DocId::new(7),
    };

    let cfg = IngestConfig::default();
    let out = ingest(&req, &cfg, &reg).expect("CSV with injection prefixes must extract");

    let combined_chunks: String = out
        .chunks
        .iter()
        .map(|c| c.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(combined_chunks.contains("=1+1"));
    assert!(combined_chunks.contains("+2"));
    assert!(combined_chunks.contains("-3"));
    assert!(combined_chunks.contains("@mention"));
}
