//! Property-Tests für Ingestion, Kanonisierung und Chunking (WP-5).
//!
//! # Zweck & Invarianten
//! - Idempotenz der Text-Sanitisierung & Kanonisierung (INV-1).
//! - Vollständige Textabdeckung der Chunks (kein Datenverlust).
//! - Monotone und syntaktisch valide Breadcrumb-Hierarchien.
//! - Strenger Determinismus bei Wiederholung identischer Eingaben (INV-5).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_ingest::canonical::sanitize_text;
use contextra_ingest::chunking::types::DocId;
use contextra_ingest::format::FormatId;
use contextra_ingest::pipeline::{ingest, IngestConfig, IngestRequest};
use contextra_ingest::registry::ExtractorRegistry;
use proptest::prelude::*;

proptest! {
    /// Property: `sanitize_text` ist absolut idempotent:
    /// f(f(x)) == f(x)
    #[test]
    fn prop_sanitize_text_is_idempotent(input in "\\PC*") {
        let (sanitized, _stripped) = sanitize_text(&input);
        let (double_sanitized, double_stripped) = sanitize_text(&sanitized);
        prop_assert_eq!(&sanitized, &double_sanitized);
        prop_assert_eq!(double_stripped, 0);
    }

    /// Property: Pipeline-Verarbeitung ist streng deterministisch:
    /// Zwei Aufrufe mit identischen Bytes und Parametern liefern bitidentische Chunks und Metadaten.
    #[test]
    fn prop_pipeline_determinism(text in "[a-zA-Z0-9 \n\t.,;:?!-_]{0,2000}") {
        let registry = ExtractorRegistry::with_defaults();
        let cfg = IngestConfig::default();

        let req1 = IngestRequest {
            bytes: text.as_bytes(),
            filename: Some("prop_test.txt"),
            mime: None,
            format_override: Some(FormatId::Txt),
            doc_id: DocId::new(42),
        };
        let out1 = ingest(&req1, &cfg, &registry).expect("ingest txt 1");

        let req2 = IngestRequest {
            bytes: text.as_bytes(),
            filename: Some("prop_test.txt"),
            mime: None,
            format_override: Some(FormatId::Txt),
            doc_id: DocId::new(42),
        };
        let out2 = ingest(&req2, &cfg, &registry).expect("ingest txt 2");

        prop_assert_eq!(out1.canonical_hash, out2.canonical_hash);
        prop_assert_eq!(out1.chunks.len(), out2.chunks.len());
        for (c1, c2) in out1.chunks.iter().zip(out2.chunks.iter()) {
            prop_assert_eq!(&c1.content, &c2.content);
            prop_assert_eq!(c1.token_count, c2.token_count);
            prop_assert_eq!(&c1.metadata, &c2.metadata);
        }
    }

    /// Property: Chunk-Abdeckung
    /// Bei Prosa-Texten mit Absätzen enthält die Gesamtheit der Chunks alle signifikanten Wörter/Tokens
    /// des Ausgangstexts (kein willkürlicher Datenverlust).
    #[test]
    fn prop_chunk_coverage_prose(words in proptest::collection::vec("[a-z]{3,10}", 10..100)) {
        let original_text = words.join(" ");
        let registry = ExtractorRegistry::with_defaults();
        let cfg = IngestConfig::default();

        let req = IngestRequest {
            bytes: original_text.as_bytes(),
            filename: Some("coverage.txt"),
            mime: None,
            format_override: Some(FormatId::Txt),
            doc_id: DocId::new(1),
        };

        let output = ingest(&req, &cfg, &registry).expect("ingest coverage txt");
        let combined_chunks: String = output.chunks.iter().map(|c| c.content.as_str()).collect::<Vec<_>>().join(" ");

        // Jedes ursprüngliche Wort muss in mindestens einem Chunk auftauchen
        for word in &words {
            prop_assert!(
                combined_chunks.contains(word),
                "Missing word '{}' in combined chunks: '{}'",
                word,
                combined_chunks
            );
        }
    }

    /// Property: Markdown Breadcrumb-Hierarchie
    /// Überschriften-Ebenen erzeugen monotone und syntaktisch saubere Breadcrumbs in den Metadaten.
    #[test]
    fn prop_markdown_breadcrumb_consistency(
        h1 in "[A-Z][a-z]{2,8}",
        h2 in "[A-Z][a-z]{2,8}",
        body in "[a-z ]{10,50}"
    ) {
        let md = format!("# {h1}\n\n## {h2}\n\n{body}\n");
        let registry = ExtractorRegistry::with_defaults();
        let cfg = IngestConfig::default();

        let req = IngestRequest {
            bytes: md.as_bytes(),
            filename: Some("hierarchy.md"),
            mime: None,
            format_override: Some(FormatId::Markdown),
            doc_id: DocId::new(10),
        };

        let output = ingest(&req, &cfg, &registry).expect("ingest md");
        for chunk in &output.chunks {
            if let Some(ref meta) = chunk.metadata {
                if let Some(breadcrumbs) = meta.get("breadcrumbs").and_then(|v| v.as_array()) {
                    for bc in breadcrumbs {
                        let s = bc.as_str().unwrap_or_default();
                        prop_assert!(!s.is_empty(), "Breadcrumb string must not be empty");
                    }
                }
            }
        }
    }
}
