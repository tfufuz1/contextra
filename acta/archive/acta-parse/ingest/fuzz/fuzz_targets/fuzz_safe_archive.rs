//! Fuzzing-Target: `fuzz_safe_archive.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante und DoS-/Zip-Slip-Schutz von `acta-ingest`
//! gegenueber beliebig manipulierten Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let limits = acta_ingest::limits::IngestLimits::default();
    let _ = acta_ingest::safe::archive::validate_zip_archive(Cursor::new(data), &limits);

    let budget = acta_ingest::extractor::ResourceBudget::new(limits);
    if let Ok(mut archive) = acta_ingest::safe::archive::SafeArchive::new(data, &budget) {
        let _ = archive.list_entries();
        let _ = archive.read_entry("test.xml", &budget);
    }
});

// ANCHOR[acta-ingest:FUZZ-0459]
// Fuzzing-Target-Definition fuer fuzz_safe_archive.rs.
