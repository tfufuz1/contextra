# Extraktion fuer `acta-parse`

**Aufgabe:** Parser PDF/Office/HTML/Text/Archive (nicht vertrauenswuerdige Eingaben)  
**Level:** L1 · **Budget:** 5000 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.1, 13.2  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 112 Dateien, 21329 Zeilen (inkl. Tests)  

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
| `ingest/src/detect/mod.rs` | 769 | panic=5 |
| `ingest/src/extractor.rs` | 224 |  |
| `ingest/src/format.rs` | 251 | serde=1 |
| `ingest/src/formats/docx/document.rs` | 610 |  |
| `ingest/src/formats/docx/mod.rs` | 318 |  |
| `ingest/src/formats/docx/notes.rs` | 213 | unsafe=1 |
| `ingest/src/formats/docx/numbering.rs` | 148 | unsafe=1 |
| `ingest/src/formats/docx/styles.rs` | 218 | unsafe=1 |
| `ingest/src/formats/email/eml.rs` | 392 | unsafe=1 |
| `ingest/src/formats/email/html.rs` | 227 |  |
| `ingest/src/formats/email/mbox.rs` | 205 |  |
| `ingest/src/formats/email/mod.rs` | 58 |  |
| `ingest/src/formats/mod.rs` | 46 |  |
| `ingest/src/formats/odf/content.rs` | 474 |  |
| `ingest/src/formats/odf/meta.rs` | 94 |  |
| `ingest/src/formats/odf/mod.rs` | 199 |  |
| `ingest/src/formats/odf/presentation.rs` | 232 |  |
| `ingest/src/formats/ole/cfb_reader.rs` | 152 | std_fs=1 |
| `ingest/src/formats/ole/doc.rs` | 427 |  |
| `ingest/src/formats/ole/mod.rs` | 65 |  |
| `ingest/src/formats/ole/msg.rs` | 575 |  |
| `ingest/src/formats/ole/ppt.rs` | 292 |  |
| `ingest/src/formats/pdf/mod.rs` | 331 | unsafe=1 |
| `ingest/src/formats/pdf/text_processing.rs` | 260 | unsafe=1 |
| `ingest/src/formats/pptx/mod.rs` | 128 |  |
| `ingest/src/formats/pptx/presentation.rs` | 65 |  |
| `ingest/src/formats/pptx/slide.rs` | 504 |  |
| `ingest/src/formats/sheets/csv_fmt.rs` | 333 | float=9, unsafe=1 |
| `ingest/src/formats/sheets/excel.rs` | 381 | float=1, unsafe=1 |
| `ingest/src/formats/sheets/mod.rs` | 69 | unsafe=1 |
| `ingest/src/formats/structured/json.rs` | 426 | float=1, serde=7, panic=5 |
| `ingest/src/formats/structured/mod.rs` | 93 |  |
| `ingest/src/formats/structured/path.rs` | 116 |  |
| `ingest/src/formats/structured/toml.rs` | 190 | panic=2 |
| `ingest/src/formats/structured/xml.rs` | 385 | panic=2 |
| `ingest/src/formats/structured/yaml.rs` | 261 | panic=2 |
| `ingest/src/formats/text/asciidoc.rs` | 213 |  |
| `ingest/src/formats/text/encoding.rs` | 212 |  |
| `ingest/src/formats/text/markdown.rs` | 112 | panic=1 |
| `ingest/src/formats/text/mod.rs` | 94 |  |
| `ingest/src/formats/text/rst.rs` | 298 |  |
| `ingest/src/formats/text/rtf.rs` | 399 | panic=1 |
| `ingest/src/formats/text/tex.rs` | 303 |  |
| `ingest/src/formats/text/txt.rs` | 80 |  |
| `ingest/src/formats/web/encoding.rs` | 130 |  |
| `ingest/src/formats/web/epub.rs` | 413 |  |
| `ingest/src/formats/web/html.rs` | 984 | unsafe=1, panic=1 |
| `ingest/src/formats/web/mod.rs` | 61 |  |
| `ingest/src/registry.rs` | 165 | panic=1 |
| `ingest/src/safe/archive.rs` | 382 | panic=6 |
| `ingest/src/safe/mod.rs` | 25 |  |
| `ingest/src/safe/opc.rs` | 260 |  |
| `ingest/src/safe/xml.rs` | 367 |  |
| `ingest/tests/corpus/sample.adoc` | 10 |  |
| `ingest/tests/corpus/sample.csv` | 5 |  |
| `ingest/tests/corpus/sample.eml` | 14 |  |
| `ingest/tests/corpus/sample.html` | 16 |  |
| `ingest/tests/corpus/sample.json` | 11 |  |
| `ingest/tests/corpus/sample.jsonl` | 3 |  |
| `ingest/tests/corpus/sample.mbox` | 18 |  |
| `ingest/tests/corpus/sample.md` | 20 |  |
| `ingest/tests/corpus/sample.rst` | 10 |  |
| `ingest/tests/corpus/sample.rtf` | 1 |  |
| `ingest/tests/corpus/sample.tex` | 9 |  |
| `ingest/tests/corpus/sample.toml` | 7 | serde=1 |
| `ingest/tests/corpus/sample.txt` | 3 |  |
| `ingest/tests/corpus/sample.xml` | 9 |  |
| `ingest/tests/corpus/sample.yaml` | 10 |  |
| `ingest/tests/detect_proptest.rs` | 35 |  |
| `ingest/tests/detect_tests.rs` | 369 | panic=6 |
| `ingest/tests/docx_support/mod.rs` | 123 |  |
| `ingest/tests/docx_test.rs` | 437 | panic=11, std_fs=1 |
| `ingest/tests/email_golden_tests.rs` | 505 | panic=11 |
| `ingest/tests/fixtures/chunk_email/sample_mail_with_attachment.eml` | 12 |  |
| `ingest/tests/fixtures/chunk_prose/mod.rs` | 118 |  |
| `ingest/tests/fixtures/chunk_structured/sample_kunden.json` | 21 |  |
| `ingest/tests/fixtures/structured/sample.json` | 15 |  |
| `ingest/tests/fixtures/structured/sample.jsonl` | 3 |  |
| `ingest/tests/fixtures/structured/sample.toml` | 11 |  |
| `ingest/tests/fixtures/structured/sample.xml` | 13 |  |
| `ingest/tests/fixtures/structured/sample.yaml` | 9 |  |
| `ingest/tests/fuzz_regression_tests.rs` | 188 | panic=4, std_fs=1 |
| `ingest/tests/golden/sample.adoc.golden.json` | 36 |  |
| `ingest/tests/golden/sample.csv.golden.json` | 43 |  |
| `ingest/tests/golden/sample.eml.golden.json` | 33 |  |
| `ingest/tests/golden/sample.html.golden.json` | 29 |  |
| `ingest/tests/golden/sample.json.golden.json` | 28 |  |
| `ingest/tests/golden/sample.jsonl.golden.json` | 32 |  |
| `ingest/tests/golden/sample.mbox.golden.json` | 57 |  |
| `ingest/tests/golden/sample.md.golden.json` | 56 |  |
| `ingest/tests/golden/sample.rst.golden.json` | 35 |  |
| `ingest/tests/golden/sample.rtf.golden.json` | 28 |  |
| `ingest/tests/golden/sample.tex.golden.json` | 51 |  |
| `ingest/tests/golden/sample.toml.golden.json` | 28 | serde=1 |
| `ingest/tests/golden/sample.tsv.golden.json` | 42 |  |
| `ingest/tests/golden/sample.txt.golden.json` | 28 |  |
| `ingest/tests/golden/sample.xml.golden.json` | 29 |  |
| `ingest/tests/golden/sample.yaml.golden.json` | 28 |  |
| `ingest/tests/golden_extraction_tests.rs` | 177 | serde=1, panic=17, std_fs=1 |
| `ingest/tests/inv_ingest_archive_bomb_tests.rs` | 121 | panic=8 |
| `ingest/tests/odf_tests.rs` | 584 | panic=18 |
| `ingest/tests/ole_support/mod.rs` | 294 | panic=24, std_fs=3 |
| `ingest/tests/ole_tests.rs` | 399 | panic=7, std_fs=1 |
| `ingest/tests/pdf_extractor_tests.rs` | 319 | panic=4 |
| `ingest/tests/pptx_golden_test.rs` | 495 | panic=24, std_fs=1 |
| `ingest/tests/security_limits_tests.rs` | 291 | zeit_zufall=1, panic=8 |
| `ingest/tests/sheets_test.rs` | 427 | panic=33 |
| `ingest/tests/structured_abuse_tests.rs` | 193 | panic=1 |
| `ingest/tests/structured_golden_tests.rs` | 280 | panic=5 |
| `ingest/tests/text_extractors_test.rs` | 265 | panic=7 |
| `ingest/tests/text_proptest.rs` | 84 |  |
| `ingest/tests/web_golden_tests.rs` | 583 | panic=23 |
