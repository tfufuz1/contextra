//! Fuzzing-Target: `fuzz_email.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `acta-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let (fmt, payload) = if data[0] % 2 == 0 {
        (acta_ingest::format::FormatId::Eml, &data[1..])
    } else {
        (acta_ingest::format::FormatId::Mbox, &data[1..])
    };
    acta_ingest::fuzz_support::run(fmt, payload);
});

// ANCHOR[acta-ingest:FUZZ-0453]
// Fuzzing-Target-Definition fuer fuzz_email.rs.
