//! Fuzzing-Target: `fuzz_safe_xml.rs`
//!
//! # Zweck & Invariante
//! Ueberprueft die Zero-Panic-Invariante und XXE-/Entity-Schutz von `acta-ingest`
//! gegenueber beliebig manipulierten XML-Bytefolgen via LibFuzzer.
//!
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = acta_ingest::limits::IngestLimits::default();
    let budget = acta_ingest::extractor::ResourceBudget::new(limits);
    let _ = acta_ingest::safe::xml::parse_xml(data, &budget);

    let mut reader = acta_ingest::safe::xml::SafeXmlReader::new(data, 256);
    while let Ok(event) = reader.next_event(&budget) {
        if matches!(event, quick_xml::events::Event::Eof) {
            break;
        }
    }
});

// ANCHOR[acta-ingest:FUZZ-0460]
// Fuzzing-Target-Definition fuer fuzz_safe_xml.rs.
