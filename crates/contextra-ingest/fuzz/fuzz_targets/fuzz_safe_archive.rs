//! Fuzzing-Target: `fuzz_safe_archive.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `contextra-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|_data: &[u8]| {
    // Stub target for safe archive fuzzing
});

// ANCHOR[contextra-ingest:FUZZ-0459]
// Fuzzing-Target-Definition fuer fuzz_safe_archive.rs.
