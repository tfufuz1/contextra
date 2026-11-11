# PROOFS: contextra-ingest — Fuzz, Korpora, Limits- und Golden-Beweise

Stand: 2026-11-11 · Reifegrad-Ziel: R3
Verantwortlich: Auftrag INGEST-B (Beweis-Auftrag)

---

## 1. Matrix: Format × Nachweis-Status

| Format | Fuzz-Ziel | Seed-Korpus | Limits-Test | Golden-Extraktion | Property-Test | Status / Beweis |
|---|---|---|---|---|---|---|
| **Text** (`txt`, `md`, `rst`, `adoc`, `tex`, `rtf`) | `fuzz_text` | 6 Seeds (`tests/corpus/sample.*`) | `text_extractors_test`, `security_limits_tests` | `golden_extraction_tests`, `text_extractors_test` | `ingest_property_tests`, `text_proptest` | **Vollständig abgedeckt** |
| **Web** (`html`, `xhtml`, `epub`) | `fuzz_web` | 1 Seed (`tests/corpus/sample.html`) | `web_golden_tests`, `security_limits_tests` | `golden_extraction_tests`, `web_golden_tests` | `ingest_property_tests`, `web_golden_tests` | **Vollständig abgedeckt** |
| **Docx** (`docx`) | `fuzz_docx` | 11 Seeds (`fuzz/corpus/fuzz_docx`) | `docx_test`, `security_limits_tests` | `docx_test` | `docx_test` | **Vollständig abgedeckt** |
| **Pptx** (`pptx`) | `fuzz_pptx` | 11 Seeds (`fuzz/corpus/fuzz_pptx`) | `pptx_golden_test`, `security_limits_tests` | `pptx_golden_test` | `pptx_golden_test` | **Vollständig abgedeckt** |
| **Odf** (`odt`, `odp`, `ods`) | `fuzz_odf` | 1 Seed (`seed_minimal.odt`) | `odf_tests`, `security_limits_tests` | `odf_tests` | `odf_tests` | **Vollständig abgedeckt** |
| **Sheets** (`xlsx`, `csv`, `tsv`) | `fuzz_sheets` | 2 Seeds (`sample.csv`, `sample.tsv`) | `sheets_test`, `security_limits_tests` | `golden_extraction_tests`, `sheets_test` | `chunk_tabular_tests` | **Vollständig abgedeckt** |
| **PDF** (`pdf`) | `fuzz_pdf` | 1 Seed (`seed_minimal.pdf`) | `pdf_extractor_tests`, `security_limits_tests` | `pdf_extractor_tests` | `chunk_paged_proptest` | **Vollständig abgedeckt** |
| **OLE / CFB** (`doc`, `ppt`, `msg`) | `fuzz_ole` | 12 Seeds (`fuzz/corpus/fuzz_ole`) | `ole_tests`, `security_limits_tests` | `ole_tests` | `ole_tests` | **Vollständig abgedeckt** |
| **Email** (`eml`, `mbox`) | `fuzz_email` | 2 Seeds (`sample.eml`, `sample.mbox`) | `email_golden_tests`, `security_limits_tests` | `golden_extraction_tests`, `email_golden_tests` | `chunk_email_tests` | **Vollständig abgedeckt** |
| **Structured** (`json`, `yaml`, `toml`, `xml`) | `fuzz_structured` | 5 Seeds (`sample.json`, `sample.jsonl`, etc.) | `structured_abuse_tests`, `security_limits_tests` | `golden_extraction_tests`, `structured_golden_tests` | `chunk_structured_tests` | **Vollständig abgedeckt** |
| **Safe Archive** (ZIP) | `fuzz_safe_archive` | 1 Seed (`seed_simple.zip`) | `inv_ingest_archive_bomb_tests`, `security_limits_tests` | N/A (Container-Schutz) | `inv_ingest_archive_bomb_tests` | **Vollständig abgedeckt** |
| **Safe XML** (quick-xml) | `fuzz_safe_xml` | 1 Seed (`sample.xml`) | `inv_ingest_archive_bomb_tests`, `security_limits_tests` | N/A (Container-Schutz) | `inv_ingest_archive_bomb_tests` | **Vollständig abgedeckt** |
| **Detect / Magic** | `fuzz_detect` | 6 Seeds (`sample.*`) | `inv_ingest_bounded_tests`, `security_limits_tests` | `detect_tests` | `detect_proptest` | **Vollständig abgedeckt** |
| **Pipeline (End-to-End)** | `fuzz_pipeline` | 5 Seeds (`sample.*`) | `contract_tests`, `security_limits_tests` | `golden_extraction_tests` | `ingest_property_tests` | **Vollständig abgedeckt** |

---

## 2. Nachweise & Beweisführung

1. **Golden-Extraktion (`tests/golden_extraction_tests.rs`)**:
   - Synthetischer Eingabekorpus unter `tests/corpus/` deckt alle wesentlichen Formate ab.
   - `MANIFEST.sha256` sichert die Bitidentität des Korpus und der zugehörigen Golden-JSON-Dateien.
   - Der ständige Prüftest `test_golden_outputs_bitidentical` läuft in der CI und verifiziert Bitidentität über Hash- und semantische Strukturvergleiche.
2. **Limits- und Sicherheitstests (`tests/security_limits_tests.rs`)**:
   - `test_security_zip_slip_rejection`: Erkennt und weist Traversierungsversuche (`..`, absolute Pfade, Drive-Letters) zurück.
   - `test_security_zip_bomb_ratio_limit`: Schützt vor extremen Kompressionsraten (> 15:1 im Test).
   - `test_security_zip_bomb_total_decompressed_bytes`: Schützt vor Überschreitung der unkomprimierten Maximalgröße.
   - `test_security_xml_xxe_and_entity_expansion_rejected`: Schützt vor Billion-Laughs, externen DTDs und SYSTEM-Entities.
   - `test_security_xml_deep_nesting_limit`: Schützt vor Rekursions-Stack-Overflow bei extrem tiefer XML-Verschachtelung.
   - `test_security_pipeline_max_document_bytes`: Setzt das Dateigrößen-Limit deterministisch vor der Extraktion durch.
   - `test_security_truncated_and_corrupt_headers_no_panic`: Verifiziert Zero-Panic und saubere typisierte Fehler bei unvollständigen/abgeschnittenen Datei-Headern für 17 Formate unter Zeitschranke (< 5s).
3. **Fuzz-Regression & Dictionaries (`tests/fuzz_regression_tests.rs`, `fuzz/dict/`)**:
   - Dictionaries für HTML, PDF und XML bereitgestellt.
   - Alle 14 Fuzz-Ziele verfügen über Seed-Korpora und werden im Integrationstest `fuzz_regression_tests` deterministisch ausgeführt.
4. **Property-Tests (`tests/ingest_property_tests.rs`)**:
   - Beweist mathematische Idempotenz von `sanitize_text`: `sanitize(sanitize(x)) == sanitize(x)`.
   - Beweist strikten Pipeline-Determinismus bei identischen Eingaben.
   - Beweist vollständige Textabdeckung der Chunks ohne unzulässigen Datenverlust.
   - Beweist monotone und syntaktisch valide Breadcrumb-Hierarchien.

---

## 3. Übergaben an Z (Governance & Root-Dateien)

1. **justfile (`fuzz-all`)**:
   - Datei: `justfile`
   - Befund: Die Liste `fuzz-all` enthält aktuell keine Fuzz-Ziele aus `contextra-ingest`.
   - Empfohlene Ergänzung:
     ```just
     fuzz-ingest target="fuzz_pipeline":
         cargo +nightly fuzz run {{target}} crates/contextra-ingest/fuzz -- -max_total_time=60
     ```
     Aufnahme der 14 Ingest-Ziele (`fuzz_detect`, `fuzz_text`, `fuzz_web`, `fuzz_docx`, `fuzz_pptx`, `fuzz_sheets`, `fuzz_odf`, `fuzz_pdf`, `fuzz_ole`, `fuzz_email`, `fuzz_structured`, `fuzz_safe_archive`, `fuzz_safe_xml`, `fuzz_pipeline`) in die globale Testmatrix.
2. **capabilities.toml**:
   - Keine Änderung notwendig; `contextra-ingest` ist dort bereits korrekt in Tier `experimental`, Group `data`, Ring `4` deklariert.
