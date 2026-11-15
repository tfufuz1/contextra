//! Fuzzing-Target: `fuzz_sheets.rs`
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
    let format = match data[0] % 6 {
        0 => acta_ingest::format::FormatId::Xlsx,
        1 => acta_ingest::format::FormatId::Xls,
        2 => acta_ingest::format::FormatId::Xlsb,
        3 => acta_ingest::format::FormatId::Ods,
        4 => acta_ingest::format::FormatId::Csv,
        _ => acta_ingest::format::FormatId::Tsv,
    };
    acta_ingest::fuzz_support::run(format, &data[1..]);
});

// ANCHOR[acta-ingest:FUZZ-0461]
// Fuzzing-Target-Definition fuer fuzz_sheets.rs.
