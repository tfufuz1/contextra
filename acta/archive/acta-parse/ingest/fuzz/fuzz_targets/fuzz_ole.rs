//! Fuzzing-Target: `fuzz_ole.rs`
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

    let format = match data[0] % 3 {
        0 => acta_ingest::format::FormatId::Doc,
        1 => acta_ingest::format::FormatId::Ppt,
        _ => acta_ingest::format::FormatId::Msg,
    };

    acta_ingest::fuzz_support::run(format, &data[1..]);
});

// ANCHOR[acta-ingest:FUZZ-0455]
// Fuzzing-Target-Definition fuer fuzz_ole.rs.
