//! Fuzzing-Target: `fuzz_structured.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `contextra-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let format = match data[0] % 5 {
        0 => contextra_ingest::format::FormatId::Json,
        1 => contextra_ingest::format::FormatId::Jsonl,
        2 => contextra_ingest::format::FormatId::Xml,
        3 => contextra_ingest::format::FormatId::Yaml,
        _ => contextra_ingest::format::FormatId::Toml,
    };

    contextra_ingest::fuzz_support::run(format, &data[1..]);
});

// ANCHOR[contextra-ingest:FUZZ-0462]
// Fuzzing-Target-Definition fuer fuzz_structured.rs.
