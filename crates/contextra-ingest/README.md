<!-- doc-stamp: contextra-ingest v0.1.0 (SPEC C-10.3) -->
# contextra-ingest

Adapter-Crate für Dokumentformat-Extraktion und strukturbewusstes Chunking in Contextra (Ring 4).

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-ingest` ist ein vollständig autarker, modularer Baustein zur Extraktion, Bereinigung, Kanonisierung und semantischen Zerlegung (Chunking) unterschiedlichster Dokumentformate (Markdown, Plaintext, PDF, DOCX, XLSX, HTML, E-Mail, JSON/YAML/TOML etc.).

Gemäß den Baukasten-Richtlinien (M1–M11) hängt `contextra-ingest` im Kern ausschließlich an `contextra-types` und Standardformatbibliotheken. Es besitzt **keine** Laufzeit-Abhängigkeit zu `contextra-engine`, Datenbanken oder Server-Komponenten und kann isoliert in Fremdprojekte eingebunden werden.

## 2. Abhängigkeiten & Features
- **Abhängigkeiten:** `contextra-types` (Ring 0) sowie isolierte Parser-Crates. Keine Aufwärtskanten.
- **Features (additiv, Defaults minimal):**
  - `fmt-text` (Default): Plaintext, Markdown, AsciiDoc, RTF, TeX, RST.
  - `fmt-web`: HTML, XHTML, EPUB.
  - `fmt-docx`: Microsoft Word DOCX.
  - `fmt-pptx`: Microsoft PowerPoint PPTX.
  - `fmt-odf`: OpenDocument (ODT, ODS, ODP).
  - `fmt-sheets`: Excel XLSX/XLSB/XLS, CSV, TSV.
  - `fmt-pdf`: PDF Extraktion.
  - `fmt-ole`: OLE / Compound File Binary (DOC, PPT, XLS, MSG).
  - `fmt-email`: E-Mail (EML).
  - `fmt-structured`: JSON, JSONL, YAML, TOML, XML.
  - `all-formats`: Aktiviert alle Extraktionsformate.

## 3. Garantien & Invarianten
- **Speichersicherheit:** `#![forbid(unsafe_code)]` im gesamten Crate.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()`, `expect()` und `panic!()` im Produktivcode. Parser-Fehler resultieren stets in typisierten Fehlern (`IngestError`).
- **Schutz vor Ressourcenerschöpfung (INV-LIMIT-1):** Nicht abschaltbare Schutzgrenzen gegen Zip-Bomben, tiefe XML/HTML-Verschachtelungen und exzessive Dateigrößen (`IngestLimits`, `ResourceBudget`).
- **Deterministische Provenienz:** Jeder erzeugte `ContextChunk` referenziert präzise Locators (Seitenzahlen, Zeilenspannen, Pfade), Heading-Breadcrumbs und kanonische Hashes.

## 4. Einbettungsbeispiel

```rust
use contextra_ingest::chunking::config::ChunkingConfig;
use contextra_ingest::format::FormatId;
use contextra_ingest::pipeline::{ingest, IngestConfig, IngestRequest};
use contextra_ingest::registry::ExtractorRegistry;
use contextra_types::DocId;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ExtractorRegistry::with_defaults();
    let sample = "# Titel\n\nErster Absatz mit Text.";

    let request = IngestRequest {
        bytes: sample.as_bytes(),
        filename: Some("doc.md"),
        mime: Some("text/markdown"),
        format_override: Some(FormatId::Markdown),
        doc_id: DocId::new(1),
    };

    let config = IngestConfig::default();
    let output = ingest(&request, &config, &registry)?;

    println!("Extrahierte Chunks: {}", output.chunks.len());
    for chunk in output.chunks {
        println!("- {}: {}", chunk.token_count, chunk.content);
    }
    Ok(())
}
```

Ein vollständiges, ausführbares Beispiel befindet sich unter `examples/extract_and_chunk.rs` und kann via `cargo run -p contextra-ingest --example extract_and_chunk` gestartet werden.
