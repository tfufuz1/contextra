#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    contextra_ingest::fuzz_support::run(contextra_ingest::format::FormatId::Odt, data);
    contextra_ingest::fuzz_support::run(contextra_ingest::format::FormatId::Odp, data);
});
