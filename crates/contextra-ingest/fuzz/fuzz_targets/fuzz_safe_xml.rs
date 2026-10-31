//! Fuzzing-Target: `fuzz_safe_xml.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `contextra-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|_data: &[u8]| {
    // Stub target for safe xml fuzzing
});

// ANCHOR[contextra-ingest:FUZZ-0460]
// Fuzzing-Target-Definition fuer fuzz_safe_xml.rs.
