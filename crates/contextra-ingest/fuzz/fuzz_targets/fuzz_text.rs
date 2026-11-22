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
