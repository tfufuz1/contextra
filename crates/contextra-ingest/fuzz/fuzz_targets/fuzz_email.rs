#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let (fmt, payload) = if data[0] % 2 == 0 {
        (contextra_ingest::format::FormatId::Eml, &data[1..])
    } else {
        (contextra_ingest::format::FormatId::Mbox, &data[1..])
    };
    contextra_ingest::fuzz_support::run(fmt, payload);
});
