//! Beispiel: Autarke Dokumenten-Extraktion und strukturbewusstes Chunking
//!
//! Demonstriert die Verwendung von `contextra-ingest` als eigenständiger, isolierter
//! Baukasten ohne Datenbank, Engine oder externe Laufzeitabhängigkeiten.

use contextra_ingest::chunking::config::ChunkingConfig;
use contextra_ingest::chunking::types::DocId;
use contextra_ingest::format::FormatId;
use contextra_ingest::pipeline::{ingest, IngestConfig, IngestRequest};
use contextra_ingest::registry::ExtractorRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Contextra-Ingest Standalone Building Block Demo ===");

    // 1. Initialisiere die Extractor-Registry mit den standardmäßig aktivierten Formaten
    let registry = ExtractorRegistry::with_defaults();

    // 2. Erstelle ein Beispieldokument (Markdown mit Abschnitten und Headings)
    let sample_markdown = r#"# Einführung in Contextra
Contextra ist eine hochgradig modulare RAG- und Wissensspeicher-Bibliothek.

## Baukasten-Architektur
Jede Komponente ist ein autarker Baustein, der strengen Entkopplungsmustern folgt.
Abhängigkeiten zeigen stets nur nach unten.

### Chunking
Das Chunking erfolgt strukturbewusst unter Berücksichtigung von Dokumenthierarchien.
"#;

    let request = IngestRequest {
        bytes: sample_markdown.as_bytes(),
        filename: Some("dokument.md"),
        mime: Some("text/markdown"),
        format_override: Some(FormatId::Markdown),
        doc_id: DocId::new(42),
    };

    // 3. Konfiguriere Limits und Chunking-Strategie
    let mut config = IngestConfig::default();
    let mut chunking = ChunkingConfig::default();
    chunking.max_tokens = 128;
    chunking.min_tokens = 10;
    chunking.overlap_pct = 10;
    chunking.hard_limit_pct = 120;
    chunking.include_breadcrumbs = true;
    chunking.split_levels = vec![1, 2, 3];
    chunking.strategy = None;
    config.chunking = chunking;

    // 4. Pipeline ausführen (Erkennung -> Extraktion -> Kanonisierung -> Chunking)
    let output = ingest(&request, &config, &registry)?;

    println!("Dokument erfolgreich verarbeitet!");
    println!("- Format: {:?}", output.format);
    println!("- Kanonischer Hash: {}", output.canonical_hash);
    println!("- Erzeugte Chunks: {}", output.chunks.len());
    println!();

    for (i, chunk) in output.chunks.iter().enumerate() {
        println!("--- Chunk {} (Tokens: {}) ---", i + 1, chunk.token_count);
        if let Some(ref meta) = chunk.metadata {
            if let Some(bc) = meta.get("breadcrumb") {
                println!("  Breadcrumb: {}", bc);
            }
        }
        println!("  Inhalt: {:?}", chunk.content);
        println!();
    }

    Ok(())
}
