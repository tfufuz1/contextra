//! Fuzzing-Target: `fuzz_detect.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `acta-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let hints = acta_ingest::detect::DetectHints {
        filename: Some("fuzz.bin"),
        mime: None,
    };
    let _ = acta_ingest::detect::detect(data, &hints);
});

// ANCHOR[acta-ingest:FUZZ-0451]
// Fuzzing-Target-Definition fuer fuzz_detect.rs.
