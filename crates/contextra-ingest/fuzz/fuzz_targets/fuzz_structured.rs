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
