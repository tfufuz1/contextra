//! Fuzzing-Target: `fuzz_detect.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `contextra-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let hints = contextra_ingest::detect::DetectHints {
        filename: Some("fuzz.bin"),
        mime: None,
    };
    let _ = contextra_ingest::detect::detect(data, &hints);
});

// ANCHOR[contextra-ingest:FUZZ-0451]
// Fuzzing-Target-Definition fuer fuzz_detect.rs.
