# `contextra-ingest`

[![Ring](https://img.shields.io/badge/ring-4-blue.svg)](https://github.com/tfufuz1/contextra)
[![Klasse](https://img.shields.io/badge/klasse-A-brightgreen.svg)](https://github.com/tfufuz1/contextra)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](Cargo.toml)

> **Zustandsloser, autarker Baukasten für Dokument-Extraktion und strukturbewusstes Chunking (SPEC v4.0 Klasse A).**

`contextra-ingest` ist ein eigenständiger Baustein ohne interne Abhängigkeiten (`contextra-*`) im Produktionscode. Er nimmt Bytestreams beliebiger Dokumentformate entgegen, analysiert und extrahiert Text- und Tabellenstrukturen und zerlegt diese in semantische, deterministische Chunks.

---

## Baustein-Eigenschaften (Klasse A)

- **Vollkommen autark (B1, B2):** Keine internen Workspace-Abhängigkeiten in `[dependencies]` und `[dev-dependencies]`.
- **Typen-Autonomie (B3):** Vollständig standardtypisiert (`DocId`, `ContextChunk`, `&[u8]`, Standardfehler).
- **Ein einheitlicher Fehlertyp (B4):** `IngestError` (`thiserror`, `#[non_exhaustive]`).
- **Feature-Isolation (B6):** Jedes Format ist hinter einem feingranularen Feature-Flag (`fmt-*`) isoliert; `default = ["fmt-text"]`.
- **Zero-Panic & DoS-Schutz (B9, B10):** `#![forbid(unsafe_code)]`, Schutz gegen Zip-Bomben, XXE, Rekursionsexplosionen und Panik-Isolation (`catch_unwind`).
- **Criterion-Benchmarks (B8):** Integrierte Benchmark-Suite `benches/bm_ingest.rs`.

---

## Standalone Export & Verwendung in Fremdprojekten

Da `contextra-ingest` ein privater Baukasten ist, kann er direkt in beliebige andere Projekte exportiert werden:

```bash
python3 tools/export_crate.py contextra-ingest --out /tmp/contextra-ingest-export
```

In `Cargo.toml` des Zielprojekts:

```toml
[dependencies]
contextra-ingest = { version = "0.1", default-features = false, features = ["fmt-text", "fmt-docx", "fmt-pdf"] }
```

---

## Minimalbeispiel

```rust
use contextra_ingest::chunking::config::ChunkingConfig;
use contextra_ingest::chunking::types::DocId;
use contextra_ingest::format::FormatId;
use contextra_ingest::pipeline::{ingest, IngestConfig, IngestRequest};
use contextra_ingest::registry::ExtractorRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ExtractorRegistry::with_defaults();

    let text = "# Bericht\nErster Absatz mit wichtigem Inhalt.";
    let req = IngestRequest {
        bytes: text.as_bytes(),
        filename: Some("bericht.md"),
        mime: Some("text/markdown"),
        format_override: Some(FormatId::Markdown),
        doc_id: DocId::new(1),
    };

    let config = IngestConfig::default();
    let output = ingest(&req, &config, &registry)?;

    println!("Extrahierte Chunks: {}", output.chunks.len());
    for chunk in output.chunks {
        println!("Chunk [{}]: {}", chunk.doc_id, chunk.content);
    }

    Ok(())
}
```

---

## Nicht-Ziele

- Keine Datenbankanbindung oder Persistenzschicht (Storage).
- Keine Vektor-Embeddings oder Inferenz.
- Keine Löschbeweis- oder Kryptoschredder-Logik (gehört in `contextra-erasure`).
- Kein Netzwerkladen von externen Referenzen oder DTDs.
