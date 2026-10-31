//! Integrationstest-Suite: `pptx_golden_test.rs`
//!
//! # Zweck & Testabdeckung
//! Verifiziert das Verhalten, Zero-Panic-Invariante und deterministische Ausgabe
//! von `contextra-ingest` fuer dieses Modul/Format gemaess Spezifikation.
//!
// FILE-CONTEXT
// ZWECK: Golden-Tests für PPTX-Extraktion (PresentationML, Folienreihenfolge, Formen, Notizen, Limits) (WP-ING-05).
// INVARIANTEN: Zero-Panic, Test-First, dynamische Zip-Generierung im Code.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use contextra_ingest::error::{IngestError, LimitKind};
use contextra_ingest::extractor::{DocumentExtractor, ExtractContext, ResourceBudget};
use contextra_ingest::format::FormatId;
use contextra_ingest::formats::pptx::PptxExtractor;
use contextra_ingest::ir::{BlockKind, Boundary, WarningCode};
use contextra_ingest::limits::{ExtractPolicy, IngestLimits};
use contextra_ingest::registry::ExtractorRegistry;
use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

fn build_golden_pptx() -> Vec<u8> {
    let buf = Vec::new();
    let cursor = Cursor::new(buf);
    let mut zip = ZipWriter::new(cursor);
    let options = FileOptions::default();

    // 1. [Content_Types].xml
    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
</Types>"#,
    )
    .unwrap();

    // 2. docProps/core.xml
    zip.start_file("docProps/core.xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Presentation Golden Title</dc:title>
    <dc:creator>Test Author</dc:creator>
</cp:coreProperties>"#,
    )
    .unwrap();

    // 3. ppt/presentation.xml (SldIdLst mit Reihenfolge slide3 -> slide1 -> slide2)
    zip.start_file("ppt/presentation.xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
    <p:sldIdLst>
        <p:sldId id="256" r:id="rId3"/>
        <p:sldId id="257" r:id="rId1"/>
        <p:sldId id="258" r:id="rId2"/>
    </p:sldIdLst>
</p:presentation>"#,
    )
    .unwrap();

    // 4. ppt/_rels/presentation.xml.rels
    zip.start_file("ppt/_rels/presentation.xml.rels", options)
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide3.xml"/>
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
    <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide2.xml"/>
</Relationships>"#,
    )
    .unwrap();

    // 5. ppt/slides/slide3.xml (Folien-Index 1 in Präsentation: Titel, Liste, Gruppe, sldNum Platzhalter)
    zip.start_file("ppt/slides/slide3.xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:nvGrpSpPr/>
            <p:grpSpPr/>
            <!-- Title shape -->
            <p:sp>
                <p:nvSpPr>
                    <p:cNvPr id="1" name="Title"/>
                    <p:nvPr><p:ph type="ctrTitle"/></p:nvPr>
                </p:nvSpPr>
                <p:txBody>
                    <a:bodyPr/>
                    <a:p><a:r><a:t>Slide Three First</a:t></a:r></a:p>
                </p:txBody>
            </p:sp>
            <!-- Bullet list shape -->
            <p:sp>
                <p:nvSpPr><p:cNvPr id="2" name="Body"/><p:nvPr/></p:nvSpPr>
                <p:txBody>
                    <a:bodyPr/>
                    <a:p>
                        <a:pPr lvl="0"><a:buChar char="*"/></a:pPr>
                        <a:r><a:t>Level 0 Item</a:t></a:r>
                    </a:p>
                    <a:p>
                        <a:pPr lvl="1"><a:buChar char="*"/></a:pPr>
                        <a:r><a:t>Level 1 Nested Item</a:t></a:r>
                    </a:p>
                </p:txBody>
            </p:sp>
            <!-- Slide Number placeholder (MUST BE SKIPPED) -->
            <p:sp>
                <p:nvSpPr><p:cNvPr id="3" name="SlideNum"/><p:nvPr><p:ph type="sldNum"/></p:nvPr></p:nvSpPr>
                <p:txBody>
                    <a:bodyPr/>
                    <a:p><a:r><a:t>42</a:t></a:r></a:p>
                </p:txBody>
            </p:sp>
            <!-- Group Shape -->
            <p:grpSp>
                <p:nvGrpSpPr/>
                <p:grpSpPr/>
                <p:sp>
                    <p:nvSpPr><p:cNvPr id="4" name="Inside Group"/><p:nvPr/></p:nvSpPr>
                    <p:txBody>
                        <a:bodyPr/>
                        <a:p><a:r><a:t>Text Inside Group</a:t></a:r></a:p>
                    </p:txBody>
                </p:sp>
            </p:grpSp>
        </p:spTree>
    </p:cSld>
</p:sld>"#,
    )
    .unwrap();

    // 6. ppt/slides/_rels/slide3.xml.rels (Sprechernotizen)
    zip.start_file("ppt/slides/_rels/slide3.xml.rels", options)
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rIdNotes" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide3.xml"/>
</Relationships>"#,
    )
    .unwrap();

    // 7. ppt/notesSlides/notesSlide3.xml
    zip.start_file("ppt/notesSlides/notesSlide3.xml", options)
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:sp>
                <p:nvSpPr><p:cNvPr id="1" name="Notes"/><p:nvPr/></p:nvSpPr>
                <p:txBody>
                    <a:bodyPr/>
                    <a:p><a:r><a:t>Speaker note for slide 3</a:t></a:r></a:p>
                </p:txBody>
            </p:sp>
        </p:spTree>
    </p:cSld>
</p:notes>"#,
    )
    .unwrap();

    // 8. ppt/slides/slide1.xml (Folien-Index 2: Titel, Tabelle)
    zip.start_file("ppt/slides/slide1.xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:sp>
                <p:nvSpPr><p:cNvPr id="1" name="Title"/><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr>
                <p:txBody>
                    <a:bodyPr/>
                    <a:p><a:r><a:t>Slide One Second</a:t></a:r></a:p>
                </p:txBody>
            </p:sp>
            <!-- Table Frame -->
            <p:graphicFrame>
                <p:nvGraphicFramePr><p:cNvPr id="2" name="Table"/><p:nvPr/></p:nvGraphicFramePr>
                <a:graphic>
                    <a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table">
                        <a:tbl>
                            <a:tblPr firstRow="1"/>
                            <a:tr>
                                <a:tc><a:txBody><a:p><a:r><a:t>Col A</a:t></a:r></a:p></a:txBody></a:tc>
                                <a:tc><a:txBody><a:p><a:r><a:t>Col B</a:t></a:r></a:p></a:txBody></a:tc>
                            </a:tr>
                            <a:tr>
                                <a:tc><a:txBody><a:p><a:r><a:t>Val A1</a:t></a:r></a:p></a:txBody></a:tc>
                                <a:tc><a:txBody><a:p><a:r><a:t>Val B1</a:t></a:r></a:p></a:txBody></a:tc>
                            </a:tr>
                        </a:tbl>
                    </a:graphicData>
                </a:graphic>
            </p:graphicFrame>
        </p:spTree>
    </p:cSld>
</p:sld>"#,
    )
    .unwrap();

    // 9. ppt/slides/slide2.xml (Folien-Index 3: show="0" hidden slide)
    zip.start_file("ppt/slides/slide2.xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" show="0">
    <p:cSld>
        <p:spTree>
            <p:sp>
                <p:nvSpPr><p:cNvPr id="1" name="Title"/><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr>
                <p:txBody>
                    <a:bodyPr/>
                    <a:p><a:r><a:t>Hidden Slide Title</a:t></a:r></a:p>
                </p:txBody>
            </p:sp>
            <p:sp>
                <p:nvSpPr><p:cNvPr id="2" name="Body"/><p:nvPr/></p:nvSpPr>
                <p:txBody>
                    <a:bodyPr/>
                    <a:p><a:r><a:t>PROMPT_INJECTION_ATTACK_TEXT_DO_NOT_EXTRACT</a:t></a:r></a:p>
                </p:txBody>
            </p:sp>
        </p:spTree>
    </p:cSld>
</p:sld>"#,
    )
    .unwrap();

    let res = zip.finish().unwrap();
    res.into_inner()
}

// ANCHOR[contextra-ingest:TST-0401]
// Testet die Funktionsfaehigkeit von `test_pptx_golden_extraction_order_shapes_table_notes_and_hidden`.
#[test]
fn test_pptx_golden_extraction_order_shapes_table_notes_and_hidden() {
    let pptx_bytes = build_golden_pptx();
    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let mut policy = ExtractPolicy::default();
    policy.include_hidden = false;
    policy.keep_personal_metadata = true;

    let budget = ResourceBudget::new(limits.clone());
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &reg,
        filename: Some("presentation.pptx"),
    };

    let extractor = PptxExtractor::new();
    let ir = extractor
        .extract(FormatId::Pptx, &pptx_bytes, &ctx)
        .expect("Extraction should succeed");

    // Metadaten
    assert_eq!(ir.meta.title.as_deref(), Some("Presentation Golden Title"));
    assert_eq!(ir.meta.authors, vec!["Test Author"]);
    assert_eq!(ir.meta.page_count, Some(3));

    // Blöcke
    let mut texts = Vec::new();
    for block in &ir.blocks {
        match &block.kind {
            BlockKind::Boundary(Boundary::Slide { number, title }) => {
                texts.push(format!(
                    "SLIDE {}: {}",
                    number,
                    title.as_deref().unwrap_or("")
                ));
            }
            BlockKind::Paragraph { text } => {
                texts.push(format!("P: {}", text));
            }
            BlockKind::ListItem {
                depth,
                ordered,
                text,
            } => {
                texts.push(format!("LI(d={}, ord={}): {}", depth, ordered, text));
            }
            BlockKind::Table(tbl) => {
                texts.push(format!("TBL(hdr={}): {:?}", tbl.header_rows, tbl.rows));
            }
            BlockKind::Note { kind, text } => {
                texts.push(format!("NOTE({:?}): {}", kind, text));
            }
            _ => {}
        }
    }

    // 1. Folie muss slide3 (Titel "Slide Three First") sein
    assert_eq!(texts[0], "SLIDE 1: Slide Three First");
    assert_eq!(texts[1], "LI(d=0, ord=false): Level 0 Item");
    assert_eq!(texts[2], "LI(d=1, ord=false): Level 1 Nested Item");
    assert_eq!(texts[3], "P: Text Inside Group");
    assert_eq!(texts[4], "NOTE(SpeakerNotes): Speaker note for slide 3");

    // Foliennummer-Platzhalter "42" darf NICHT im extrahierten Text sein!
    for text in &texts {
        assert!(
            !text.contains("42"),
            "Foliennummer '42' darf nicht extrahiert werden"
        );
    }

    // 2. Folie muss slide1 (Titel "Slide One Second") sein
    assert_eq!(texts[5], "SLIDE 2: Slide One Second");
    assert_eq!(
        texts[6],
        r#"TBL(hdr=1): [["Col A", "Col B"], ["Val A1", "Val B1"]]"#
    );

    // 3. Ausgeblendete Folie (slide2) darf NICHT extrahiert werden wenn include_hidden = false
    for text in &texts {
        assert!(
            !text.contains("PROMPT_INJECTION_ATTACK_TEXT_DO_NOT_EXTRACT"),
            "Inhalt ausgeblendeter Folie darf bei include_hidden=false nicht extrahiert werden"
        );
    }

    // Warnung für übersprungene versteckte Folie vorhanden
    assert!(ir
        .warnings
        .iter()
        .any(|w| w.code == WarningCode::HiddenContentSkipped));
}

// ANCHOR[contextra-ingest:TST-0402]
// Testet die Funktionsfaehigkeit von `test_pptx_hidden_slide_included_when_policy_enabled`.
#[test]
fn test_pptx_hidden_slide_included_when_policy_enabled() {
    let pptx_bytes = build_golden_pptx();
    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let mut policy = ExtractPolicy::default();
    policy.include_hidden = true;

    let budget = ResourceBudget::new(limits.clone());
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &reg,
        filename: Some("presentation.pptx"),
    };

    let extractor = PptxExtractor::new();
    let ir = extractor
        .extract(FormatId::Pptx, &pptx_bytes, &ctx)
        .expect("Extraction should succeed");

    let has_injection_text = ir.blocks.iter().any(|b| match &b.kind {
        BlockKind::Paragraph { text } => {
            text.contains("PROMPT_INJECTION_ATTACK_TEXT_DO_NOT_EXTRACT")
        }
        _ => false,
    });

    assert!(
        has_injection_text,
        "Ausgeblendete Folie muss extrahiert werden wenn include_hidden=true"
    );
}

// ANCHOR[contextra-ingest:TST-0403]
// Testet die Funktionsfaehigkeit von `test_pptx_corrupt_xml_and_zip_bomb_limits`.
#[test]
fn test_pptx_corrupt_xml_and_zip_bomb_limits() {
    let reg = ExtractorRegistry::with_defaults();
    let extractor = PptxExtractor::new();

    // Kaputtes Archiv / kaputtes XML
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &reg,
        filename: None,
    };

    let corrupt_err = extractor
        .extract(FormatId::Pptx, b"not a zip", &ctx)
        .unwrap_err();
    assert!(matches!(corrupt_err, IngestError::Corrupt { .. }));

    // Seitenlimit überschritten (max_pages = 1)
    let mut strict_limits = IngestLimits::default();
    strict_limits.max_pages = 1;

    let strict_budget = ResourceBudget::new(strict_limits.clone());
    let strict_ctx = ExtractContext {
        limits: &strict_limits,
        policy: &policy,
        budget: &strict_budget,
        registry: &reg,
        filename: None,
    };

    let golden_bytes = build_golden_pptx();
    let limit_err = extractor
        .extract(FormatId::Pptx, &golden_bytes, &strict_ctx)
        .unwrap_err();
    assert!(matches!(
        limit_err,
        IngestError::LimitExceeded {
            kind: LimitKind::Pages,
            ..
        }
    ));
}

// ANCHOR[contextra-ingest:TST-0404]
// Testet die Funktionsfaehigkeit von `test_fuzz_seeds_generation`.
#[test]
fn test_fuzz_seeds_generation() {
    let corpus_dir = Path::new("fuzz/corpus/fuzz_pptx");
    fs::create_dir_all(corpus_dir).unwrap();

    let golden_bytes = build_golden_pptx();

    let seeds: Vec<(&str, Vec<u8>)> = vec![
        ("seed_01_empty.bin", vec![]),
        (
            "seed_02_junk.bin",
            b"PK\x03\x04random junk bytes 12345".to_vec(),
        ),
        ("seed_03_golden.bin", golden_bytes.clone()),
        (
            "seed_04_corrupt_xml.bin",
            b"PK\x03\x04corrupt pptx content".to_vec(),
        ),
        (
            "seed_05_no_pres.bin",
            b"PK\x03\x04no presentation xml".to_vec(),
        ),
        (
            "seed_06_slide_only.bin",
            golden_bytes[..golden_bytes.len() / 2].to_vec(),
        ),
        ("seed_07_small_zip.bin", golden_bytes[..100].to_vec()),
        (
            "seed_08_truncated.bin",
            golden_bytes[..golden_bytes.len().saturating_sub(10)].to_vec(),
        ),
        ("seed_09_modified_header.bin", {
            let mut v = golden_bytes.clone();
            if !v.is_empty() {
                v[0] = 0xFF;
            }
            v
        }),
        ("seed_10_repeated_golden.bin", {
            let mut v = golden_bytes.clone();
            v.extend_from_slice(&golden_bytes);
            v
        }),
        ("seed_11_zeroes.bin", vec![0u8; 512]),
    ];

    for (name, data) in seeds {
        let path = corpus_dir.join(name);
        fs::write(&path, data).unwrap();
    }

    let entries = fs::read_dir(corpus_dir).unwrap().collect::<Vec<_>>();
    assert!(
        entries.len() >= 10,
        "Minderstens 10 Fuzz-Seeds müssen im Corpus existieren"
    );
}
