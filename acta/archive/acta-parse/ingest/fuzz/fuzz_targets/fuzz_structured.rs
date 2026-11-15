//! Fuzzing-Target: `fuzz_structured.rs`
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

    let format = match data[0] % 5 {
        0 => acta_ingest::format::FormatId::Json,
        1 => acta_ingest::format::FormatId::Jsonl,
        2 => acta_ingest::format::FormatId::Xml,
        3 => acta_ingest::format::FormatId::Yaml,
        _ => acta_ingest::format::FormatId::Toml,
    };

    acta_ingest::fuzz_support::run(format, &data[1..]);
});

// ANCHOR[acta-ingest:FUZZ-0462]
// Fuzzing-Target-Definition fuer fuzz_structured.rs.
