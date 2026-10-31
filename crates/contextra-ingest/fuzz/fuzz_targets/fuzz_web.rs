//! Fuzzing-Target: `fuzz_web.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `contextra-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    contextra_ingest::fuzz_support::run(contextra_ingest::format::FormatId::Html, data);
});

// ANCHOR[contextra-ingest:FUZZ-0464]
// Fuzzing-Target-Definition fuer fuzz_web.rs.
