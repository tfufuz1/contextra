//! Fuzzing-Target: `fuzz_pdf.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `acta-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    acta_ingest::fuzz_support::run(acta_ingest::format::FormatId::Pdf, data);
});

// ANCHOR[acta-ingest:FUZZ-0456]
// Fuzzing-Target-Definition fuer fuzz_pdf.rs.
