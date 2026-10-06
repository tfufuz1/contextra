#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// FILE-CONTEXT
// STAND: 2026-10-05T00:00:00Z (TASK: J27-contextra-wire)
// ZWECK: Integrationstests für SearchResponse FlatBuffers Puffer-Finish und Größenvorab-Optionen-Verifizierung.
// INVARIANTEN: Korrekter Roundtrip für Standard- und Size-Prefixed FlatBuffers; sichere Verifizierung.

use contextra_wire::{
    root_as_search_response, ScoredDocument, ScoredDocumentArgs, SearchResponse,
    SearchResponseArgs,
};
use flatbuffers::{FlatBufferBuilder, VerifierOptions};

#[test]
fn test_search_response_standard_finish_buffer_roundtrip() {
    let mut builder = FlatBufferBuilder::new();

    let doc_id = builder.create_string("doc-std-1");
    let doc = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(doc_id),
            score: 0.92,
            metadata: None,
            embedding: None,
        },
    );

    let results = builder.create_vector(&[doc]);
    let resp_offset = SearchResponse::create(
        &mut builder,
        &SearchResponseArgs {
            results: Some(results),
            total_hits: 42,
            processing_time_ms: 5.5,
        },
    );

    // Call public method that wires finish_search_response_buffer
    SearchResponse::finish_buffer(&mut builder, resp_offset);

    let buf = builder.finished_data();
    let resp = root_as_search_response(buf).expect("Failed to parse standard SearchResponse");

    assert_eq!(resp.total_hits(), 42);
    assert!((resp.processing_time_ms() - 5.5).abs() < 1e-5);
    let docs = resp.results().expect("Expected results vector");
    assert_eq!(docs.len(), 1);
    assert_eq!(docs.get(0).id(), Some("doc-std-1"));
}

#[test]
fn test_search_response_size_prefixed_finish_buffer_roundtrip() {
    let mut builder = FlatBufferBuilder::new();

    let doc_id = builder.create_string("doc-prefix-1");
    let doc = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(doc_id),
            score: 0.85,
            metadata: None,
            embedding: None,
        },
    );

    let results = builder.create_vector(&[doc]);
    let resp_offset = SearchResponse::create(
        &mut builder,
        &SearchResponseArgs {
            results: Some(results),
            total_hits: 100,
            processing_time_ms: 10.0,
        },
    );

    // Call public method that wires finish_size_prefixed_search_response_buffer
    SearchResponse::finish_size_prefixed_buffer(&mut builder, resp_offset);

    let buf = builder.finished_data();

    // Verify first 4 bytes contain size prefix in little-endian
    assert!(buf.len() > 4);

    let opts = VerifierOptions {
        max_depth: 64,
        max_tables: 1000,
        max_apparent_size: 1_000_000,
        ..Default::default()
    };

    // Call public method that wires size_prefixed_root_as_search_response_with_opts
    let resp = SearchResponse::size_prefixed_root_with_opts(&opts, buf)
        .expect("Failed to parse size-prefixed SearchResponse with options");

    assert_eq!(resp.total_hits(), 100);
    assert!((resp.processing_time_ms() - 10.0).abs() < 1e-5);
    let docs = resp.results().expect("Expected results vector");
    assert_eq!(docs.len(), 1);
    assert_eq!(docs.get(0).id(), Some("doc-prefix-1"));
}

#[test]
fn test_size_prefixed_root_with_opts_corruption_handling() {
    let opts = VerifierOptions::default();

    // Corrupted payload
    let garbage = vec![0x04, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF];
    assert!(SearchResponse::size_prefixed_root_with_opts(&opts, &garbage).is_err());

    // Truncated size-prefixed payload
    let truncated = vec![0x10, 0x00, 0x00, 0x00, 0x00];
    assert!(SearchResponse::size_prefixed_root_with_opts(&opts, &truncated).is_err());
}
