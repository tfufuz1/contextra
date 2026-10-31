//! Fuzzing-Target: `fuzz_pipeline.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `contextra-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let registry = contextra_ingest::registry::ExtractorRegistry::with_defaults();
    let req = contextra_ingest::pipeline::IngestRequest {
        bytes: data,
        filename: Some("fuzz_pipeline_input.bin"),
        mime: None,
        format_override: None,
        doc_id: contextra_types::DocId::new(1),
    };
    let cfg = contextra_ingest::pipeline::IngestConfig::default();
    let _ = contextra_ingest::pipeline::ingest(&req, &cfg, &registry);
});

// ANCHOR[contextra-ingest:FUZZ-0457]
// Fuzzing-Target-Definition fuer fuzz_pipeline.rs.
