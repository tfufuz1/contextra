//! Fuzzing-Target: `fuzz_pipeline.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `acta-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let registry = acta_ingest::registry::ExtractorRegistry::with_defaults();
    let req = acta_ingest::pipeline::IngestRequest {
        bytes: data,
        filename: Some("fuzz_pipeline_input.bin"),
        mime: None,
        format_override: None,
        doc_id: acta_ingest::chunking::types::DocId::new(1),
    };
    let cfg = acta_ingest::pipeline::IngestConfig::default();
    let _ = acta_ingest::pipeline::ingest(&req, &cfg, &registry);
});

// ANCHOR[acta-ingest:FUZZ-0457]
// Fuzzing-Target-Definition fuer fuzz_pipeline.rs.
