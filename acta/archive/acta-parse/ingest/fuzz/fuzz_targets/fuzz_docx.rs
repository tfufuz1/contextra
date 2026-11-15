//! Fuzzing-Target: `fuzz_docx.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `acta-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    acta_ingest::fuzz_support::run(acta_ingest::format::FormatId::Docx, data);
});

// ANCHOR[acta-ingest:FUZZ-0452]
// Fuzzing-Target-Definition fuer fuzz_docx.rs.
