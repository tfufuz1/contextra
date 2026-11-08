# SPEC.md: `contextra-ingest`

> **Ring 4** · **Klasse A** (Isolierter Baukasten) · Reifegrad `stable` · Unsafe: `forbid(unsafe_code)` · Status: SPEC v4.0 konform

---

## 1. Zweck & Abgrenzung

`contextra-ingest` ist ein zustandsloser, autarker Extraktions- und Chunking-Baustein für unstrukturierte und strukturierte Dateiformate (PDF, DOCX, PPTX, ODF, OLE/MSG, XLSX/CSV, HTML/EPUB, E-Mail, JSON/YAML/TOML, Plaintext).

### Nicht-Ziele (Abgrenzung)
* Kein I/O auf persistente Datenbanken oder Retrieval-Kerne (`contextra-store`, `contextra-engine`).
* Keine Löschlogik oder Löschbeweise (`contextra-erasure`).
* Keine dynamischen Netzwerkaufrufe oder externe Entity-Resolution (XXE-Schutz, Offline-Betrieb).
* Keine Mandanten- oder Transaktions-Bindung (`TxId`, `TenantId`).

---

## 2. Invarianten (INV-ING-*)

| ID | Invariante | Prüfung |
|---|---|---|
| **INV-ING-01** | **Vertikale Autarkie (Klasse A):** Keine internen Abhängigkeiten zu anderen `contextra-*` Crates im Produktionscode. | `cargo tree -p contextra-ingest -e normal --depth 1` |
| **INV-ING-02** | **Zero-Panic:** Alle Parser und Chunking-Routinen sind fehlergekapselt und lösen niemals Panics aus. | `tools/zero_panic_analyzer.py` / `catch_unwind` |
| **INV-ING-03** | **Ressourcen- und DoS-Schutz:** Striktes Einhalten von `IngestLimits` (Dokumentgröße, Entpackgröße, Dekompressionsverhältnis, XML-Tiefe, Rekursionstiefe). | `safe::archive::validate_zip_archive`, `SafeArchive`, `SafeXmlReader` |
| **INV-ING-04** | **Deterministisches Chunking:** Identischer Eingabetext führt bei gleicher `ChunkingConfig` zu exakt identischen Chunks und Spans. | Proptests, `tools/determinism_guard.py` |
| **INV-ING-05** | **XXE- und Injektionsschutz:** DTDs, externe Entitäten und gefährliche Skripte werden fail-closed abgewiesen oder neutralisiert. | XXE-Audit-Tests in `docx_test.rs`, `xml.rs` |

---

## 3. Fehlermodell (`IngestError`)

Alle Fehler werden typisiert und strukturiert über `crate::error::IngestError` kommuniziert:
* `UnsupportedFormat`: Format wird vom Baustein oder den aktivierten Features nicht unterstützt.
* `Encrypted`: Dokument ist kennwortgeschützt oder DRM-verschlüsselt.
* `Corrupt`: Datei ist beschädigt, ungültig oder verletzt Syntaxregeln.
* `LimitExceeded`: Überschreitung eines konfigurierten Sicherheitslimits (`LimitKind`).
* `NoExtractableText`: Gültiges Dokument ohne Textinhalt.
* `ParserPanic`: Isolierte Panik eines Drittanbieter-Parsers (gefangen via `catch_unwind`).
* `Chunking`: Fehler während der Segmentierungsphase.

---

## 4. Concurrency & Reentrancy

* Alle Extractor- und Chunker-Implementierungen sind synchron, reentrant und `Send + Sync`.
* Zustand existiert nur auf Aufrufer-Stack-Ebene (`ExtractContext`, `ResourceBudget`).
* Parallelisierung erfolgt durch den Aufrufer (z. B. Dokumenten-Parallelität).

---

## 5. Feature-Matrix (SPEC v4.0 G4)

| Feature | Abhängigkeiten | Beschreibung |
|---|---|---|
| `default` | `fmt-text` | Minimaler Standard (Plaintext, Markdown, RST, TeX) |
| `fmt-text` | `encoding_rs` | Plaintext und DACH-Encoding-Unterstützung |
| `fmt-web` | `html5ever`, `markup5ever_rcdom`, `encoding_rs`, `zip-container`, `xml` | HTML5-DOM-Extraktion & EPUB |
| `fmt-docx` | `zip-container`, `xml` | WordprocessingML (.docx) |
| `fmt-pptx` | `zip-container`, `xml` | PresentationML (.pptx) |
| `fmt-odf` | `zip-container`, `xml` | OpenDocument Format (.odt, .odp) |
| `fmt-sheets` | `calamine`, `csv`, `encoding_rs`, `zip-container`, `xml` | Excel (.xlsx, .xls) und CSV/TSV |
| `fmt-pdf` | `lopdf` | PDF Extraktion |
| `fmt-ole` | `cfb` | Compound File Binary (DOC, PPT, MSG) |
| `fmt-email` | `mail-parser`, `encoding_rs` | E-Mail (EML, MBOX) |
| `fmt-structured` | `yaml-rust2`, `toml`, `xml` | JSON, YAML, TOML, XML |
| `all-formats` | Alle oben genannten | Vollständiger Formatumfang |
