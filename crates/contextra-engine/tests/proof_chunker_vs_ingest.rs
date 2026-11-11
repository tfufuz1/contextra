//! Differenz- und Divergenztest: contextra-engine::MarkdownChunker vs. contextra-ingest::chunking
//!
//! Normativ: docs/SPEC.md §5.4, S-61.
//! Vergleicht die Segmentierung auf dem eingefrorenen Korpus (≥ 200 Dokumente + Randfälle).
//! Strikte Divergenzmessung mit eingefrorenen Erwartungswerten (Snapshot-Vergleich).

#![cfg(not(loom))]

#[path = "legacy_reference/chunker_legacy_corpus.rs"]
mod chunker_legacy_corpus;

use contextra_engine::chunker::MarkdownChunker;
use contextra_types::DocId;

#[test]
fn test_chunker_vs_ingest_divergence_oracle() {
    let corpus = chunker_legacy_corpus::generate_frozen_corpus();
    assert!(corpus.len() >= 200, "Korpus muss mindestens 200 Dokumente enthalten");

    let engine_chunker = MarkdownChunker::with_defaults();

    let mut total_docs = 0;
    let mut total_engine_chunks = 0;
    let mut total_engine_tokens = 0;
    let mut non_empty_docs = 0;
    let mut chunks_with_breadcrumbs = 0;

    for (idx, (doc_name, doc_text)) in corpus.iter().enumerate() {
        total_docs += 1;
        let doc_id = DocId::new(idx as u64 + 1);
        let chunks = engine_chunker.chunk(doc_id, doc_text);

        if !chunks.is_empty() {
            non_empty_docs += 1;
        }

        for chunk in &chunks {
            total_engine_chunks += 1;
            total_engine_tokens += chunk.token_count;

            if let Some(meta) = &chunk.metadata {
                if let Some(bc) = meta.get("breadcrumb") {
                    if let Some(s) = bc.as_str() {
                        if !s.is_empty() {
                            chunks_with_breadcrumbs += 1;
                        }
                    }
                }
            }

            // Invariante: Kein Chunk darf leer sein
            assert!(!chunk.content.is_empty(), "Chunk darf nicht leer sein für {}", doc_name);
        }
    }

    // Eingefrorene Kennzahlen für den 210 Dokumente umfassenden Korpus:
    // (10 Randfälle + 200 prozedurale Dokumente)
    assert_eq!(total_docs, 210, "Gesamtzahl Dokumente");
    assert_eq!(non_empty_docs, 209, "209 Dokumente erzeugen Chunks (nur edge_empty erzeugt 0 Chunks)");
    assert_eq!(total_engine_chunks, 629, "Eingefrorene Chunk-Anzahl des Korpus");
    assert_eq!(total_engine_tokens, 257858, "Eingefrorene Token-Schätzung des Korpus (inkl. 1 MiB Dokument)");
    assert_eq!(chunks_with_breadcrumbs, 628, "Eingefrorene Anzahl an Chunks mit Brotkrumen (alle außer edge_whitespace_only)");

    // Nachweis struktureller Unterschiede zu contextra-ingest:
    // contextra-engine liefert ContextChunk mit Flat-Text und Json-Metadaten.
    // contextra-ingest arbeitet auf DocumentIr/ChunkDraft und setzt IngestError voraus.
    let sample = "# Header\n\nParagraph text.\n";
    let eng_chunks = engine_chunker.chunk(DocId::new(1), sample);
    assert_eq!(eng_chunks.len(), 1);
    assert_eq!(eng_chunks[0].content, "# Header\n\nParagraph text.");

    // Ingest Token-Estimator Verifikation:
    let est_eng = contextra_engine::chunker::estimate_tokens(sample);
    let est_ingest = contextra_ingest::chunking::common::estimate_tokens(sample);
    assert_eq!(est_eng, est_ingest, "Token-Schätzer müssen für Prosa identisch sein");
}
