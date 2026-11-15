# Extraktion fuer `acta-parse`

**Aufgabe:** Parser PDF/Office/HTML/Text/Archive (nicht vertrauenswuerdige Eingaben)  
**Level:** L1 · **Budget:** 5000 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.1, 13.2  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 126 Dateien, 20772 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `ingest` | Parser (nur in acta-parse, 3.8); Ausfuehrung im Worker (13.2) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 11 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 11 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| unsafe | 10 | unsafe (G-UNSAFE: nur acta-sys) |
| zeit_zufall | 1 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 237 | unwrap/expect/panic (G-LINT) |
| std_fs | 9 | std::fs direkt (G-FS: nur acta-fs) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `blake3`, `calamine`, `cfb`, `csv`, `encoding_rs`, `html5ever`, `lopdf`, `mail-parser`, `quick-xml`, `toml`, `yaml-rust2`, `zip` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-parse/`) | LOC | Befunde |
|---|---:|---|
| `ingest/fuzz/fuzz_targets/fuzz_detect.rs` | 19 |  |
| `ingest/fuzz/fuzz_targets/fuzz_docx.rs` | 15 |  |
| `ingest/fuzz/fuzz_targets/fuzz_email.rs` | 23 |  |
| `ingest/fuzz/fuzz_targets/fuzz_odf.rs` | 16 |  |
| `ingest/fuzz/fuzz_targets/fuzz_ole.rs` | 25 |  |
| `ingest/fuzz/fuzz_targets/fuzz_pdf.rs` | 15 |  |
| `ingest/fuzz/fuzz_targets/fuzz_pipeline.rs` | 24 |  |
| `ingest/fuzz/fuzz_targets/fuzz_pptx.rs` | 15 |  |
| `ingest/fuzz/fuzz_targets/fuzz_safe_archive.rs` | 23 |  |
| `ingest/fuzz/fuzz_targets/fuzz_safe_xml.rs` | 24 |  |
| `ingest/fuzz/fuzz_targets/fuzz_sheets.rs` | 26 |  |
| `ingest/fuzz/fuzz_targets/fuzz_structured.rs` | 27 |  |
| `ingest/fuzz/fuzz_targets/fuzz_text.rs` | 28 |  |
| `ingest/fuzz/fuzz_targets/fuzz_web.rs` | 15 |  |
| `ingest/src/detect/mod.rs` | 768 | panic=5 |
| `ingest/src/extractor.rs` | 223 |  |
| `ingest/src/format.rs` | 250 | serde=1 |
| `ingest/src/formats/docx/document.rs` | 609 |  |
| `ingest/src/formats/docx/mod.rs` | 317 |  |
| `ingest/src/formats/docx/notes.rs` | 212 | unsafe=1 |
| `ingest/src/formats/docx/numbering.rs` | 147 | unsafe=1 |
| `ingest/src/formats/docx/styles.rs` | 217 | unsafe=1 |
| `ingest/src/formats/email/eml.rs` | 391 | unsafe=1 |
| `ingest/src/formats/email/html.rs` | 226 |  |
| `ingest/src/formats/email/mbox.rs` | 204 |  |
| `ingest/src/formats/email/mod.rs` | 57 |  |
| `ingest/src/formats/mod.rs` | 45 |  |
| `ingest/src/formats/odf/content.rs` | 473 |  |
| `ingest/src/formats/odf/meta.rs` | 93 |  |
| `ingest/src/formats/odf/mod.rs` | 198 |  |
| `ingest/src/formats/odf/presentation.rs` | 231 |  |
| `ingest/src/formats/ole/cfb_reader.rs` | 151 |  |
| `ingest/src/formats/ole/doc.rs` | 426 |  |
| `ingest/src/formats/ole/mod.rs` | 64 |  |
| `ingest/src/formats/ole/msg.rs` | 574 |  |
| `ingest/src/formats/ole/ppt.rs` | 291 |  |
| `ingest/src/formats/pdf/mod.rs` | 330 | unsafe=1 |
| `ingest/src/formats/pdf/text_processing.rs` | 259 | unsafe=1 |
| `ingest/src/formats/pptx/mod.rs` | 127 |  |
| `ingest/src/formats/pptx/presentation.rs` | 64 |  |
| `ingest/src/formats/pptx/slide.rs` | 503 |  |
| `ingest/src/formats/sheets/csv_fmt.rs` | 332 | float=9, unsafe=1 |
| `ingest/src/formats/sheets/excel.rs` | 380 | float=1, unsafe=1 |
| `ingest/src/formats/sheets/mod.rs` | 68 | unsafe=1 |
| `ingest/src/formats/structured/json.rs` | 425 | panic=5, float=1, serde=7 |
| `ingest/src/formats/structured/mod.rs` | 92 |  |
| `ingest/src/formats/structured/path.rs` | 115 |  |
| `ingest/src/formats/structured/toml.rs` | 189 | panic=2 |
| `ingest/src/formats/structured/xml.rs` | 384 | panic=2 |
| `ingest/src/formats/structured/yaml.rs` | 260 | panic=2 |
| `ingest/src/formats/text/asciidoc.rs` | 212 |  |
| `ingest/src/formats/text/encoding.rs` | 211 |  |
| `ingest/src/formats/text/markdown.rs` | 111 | panic=1 |
| `ingest/src/formats/text/mod.rs` | 93 |  |
| `ingest/src/formats/text/rst.rs` | 297 |  |
| `ingest/src/formats/text/rtf.rs` | 398 | panic=1 |
| `ingest/src/formats/text/tex.rs` | 302 |  |
| `ingest/src/formats/text/txt.rs` | 79 |  |
| `ingest/src/formats/web/encoding.rs` | 129 |  |
| `ingest/src/formats/web/epub.rs` | 412 |  |
| `ingest/src/formats/web/html.rs` | 983 | panic=1, unsafe=1 |
| `ingest/src/formats/web/mod.rs` | 60 |  |
| `ingest/src/registry.rs` | 164 | panic=1 |
| `ingest/src/safe/archive.rs` | 381 | panic=6 |
| `ingest/src/safe/mod.rs` | 24 |  |
| `ingest/src/safe/opc.rs` | 259 |  |
| `ingest/src/safe/xml.rs` | 366 |  |
| `ingest/tests/corpus/sample.adoc` | 0 |  |
| `ingest/tests/corpus/sample.csv` | 0 |  |
| `ingest/tests/corpus/sample.eml` | 0 |  |
| `ingest/tests/corpus/sample.html` | 0 |  |
| `ingest/tests/corpus/sample.json` | 0 |  |
| `ingest/tests/corpus/sample.jsonl` | 0 |  |
| `ingest/tests/corpus/sample.mbox` | 0 |  |
| `ingest/tests/corpus/sample.md` | 19 |  |
| `ingest/tests/corpus/sample.rst` | 0 |  |
| `ingest/tests/corpus/sample.rtf` | 0 |  |
| `ingest/tests/corpus/sample.tex` | 0 |  |
| `ingest/tests/corpus/sample.toml` | 6 |  |
| `ingest/tests/corpus/sample.txt` | 0 |  |
| `ingest/tests/corpus/sample.xml` | 0 |  |
| `ingest/tests/corpus/sample.yaml` | 0 |  |
| `ingest/tests/detect_proptest.rs` | 34 |  |
| `ingest/tests/detect_tests.rs` | 368 | panic=6 |
| `ingest/tests/docx_support/mod.rs` | 122 |  |
| `ingest/tests/docx_test.rs` | 436 | panic=11, std_fs=1 |
| `ingest/tests/email_golden_tests.rs` | 504 | panic=11 |
| `ingest/tests/fixtures/chunk_email/sample_mail_with_attachment.eml` | 0 |  |
| `ingest/tests/fixtures/chunk_prose/mod.rs` | 117 |  |
| `ingest/tests/fixtures/chunk_structured/sample_kunden.json` | 0 |  |
| `ingest/tests/fixtures/structured/sample.json` | 0 |  |
| `ingest/tests/fixtures/structured/sample.jsonl` | 0 |  |
| `ingest/tests/fixtures/structured/sample.toml` | 10 |  |
| `ingest/tests/fixtures/structured/sample.xml` | 0 |  |
| `ingest/tests/fixtures/structured/sample.yaml` | 0 |  |
| `ingest/tests/fuzz_regression_tests.rs` | 187 | panic=4, std_fs=1 |
| `ingest/tests/golden/sample.adoc.golden.json` | 0 |  |
| `ingest/tests/golden/sample.csv.golden.json` | 0 |  |
| `ingest/tests/golden/sample.eml.golden.json` | 0 |  |
| `ingest/tests/golden/sample.html.golden.json` | 0 |  |
| `ingest/tests/golden/sample.json.golden.json` | 0 |  |
| `ingest/tests/golden/sample.jsonl.golden.json` | 0 |  |
| `ingest/tests/golden/sample.mbox.golden.json` | 0 |  |
| `ingest/tests/golden/sample.md.golden.json` | 0 |  |
| `ingest/tests/golden/sample.rst.golden.json` | 0 |  |
| `ingest/tests/golden/sample.rtf.golden.json` | 0 |  |
| `ingest/tests/golden/sample.tex.golden.json` | 0 |  |
| `ingest/tests/golden/sample.toml.golden.json` | 0 |  |
| `ingest/tests/golden/sample.tsv.golden.json` | 0 |  |
| `ingest/tests/golden/sample.txt.golden.json` | 0 |  |
| `ingest/tests/golden/sample.xml.golden.json` | 0 |  |
| `ingest/tests/golden/sample.yaml.golden.json` | 0 |  |
| `ingest/tests/golden_extraction_tests.rs` | 176 | panic=17, serde=1, std_fs=1 |
| `ingest/tests/inv_ingest_archive_bomb_tests.rs` | 120 | panic=8 |
| `ingest/tests/odf_tests.rs` | 583 | panic=18 |
| `ingest/tests/ole_support/mod.rs` | 293 | panic=24 |
| `ingest/tests/ole_tests.rs` | 398 | panic=7, std_fs=1 |
| `ingest/tests/pdf_extractor_tests.rs` | 318 | panic=4 |
| `ingest/tests/pptx_golden_test.rs` | 494 | panic=24, std_fs=1 |
| `ingest/tests/security_limits_tests.rs` | 290 | panic=8 |
| `ingest/tests/sheets_test.rs` | 426 | panic=33 |
| `ingest/tests/structured_abuse_tests.rs` | 192 | panic=1 |
| `ingest/tests/structured_golden_tests.rs` | 279 | panic=5 |
| `ingest/tests/text_extractors_test.rs` | 264 | panic=7 |
| `ingest/tests/text_proptest.rs` | 83 |  |
| `ingest/tests/web_golden_tests.rs` | 582 | panic=23 |
