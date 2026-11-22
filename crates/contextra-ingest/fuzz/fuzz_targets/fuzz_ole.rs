#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let format = match data[0] % 3 {
        0 => contextra_ingest::format::FormatId::Doc,
        1 => contextra_ingest::format::FormatId::Ppt,
        _ => contextra_ingest::format::FormatId::Msg,
    };

    contextra_ingest::fuzz_support::run(format, &data[1..]);
});
