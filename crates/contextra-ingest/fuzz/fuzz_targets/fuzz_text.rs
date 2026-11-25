//! Fuzzing-Target: `fuzz_text.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante von `contextra-ingest` gegenueber
//! beliebig manipulierten, bösartigen oder unerwarteten Bytefolgen via LibFuzzer.
//!
#![no_main]
use contextra_ingest::format::FormatId;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let (first, payload) = data.split_at(1);
    let format = match first[0] % 6 {
        0 => FormatId::Txt,
        1 => FormatId::Markdown,
        2 => FormatId::Rst,
        3 => FormatId::AsciiDoc,
        4 => FormatId::Tex,
        _ => FormatId::Rtf,
    };
    contextra_ingest::fuzz_support::run(format, payload);
});

// ANCHOR[contextra-ingest:FUZZ-0463]
// Fuzzing-Target-Definition fuer fuzz_text.rs.
