#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let format = match data[0] % 6 {
        0 => contextra_ingest::format::FormatId::Xlsx,
        1 => contextra_ingest::format::FormatId::Xls,
        2 => contextra_ingest::format::FormatId::Xlsb,
        3 => contextra_ingest::format::FormatId::Ods,
        4 => contextra_ingest::format::FormatId::Csv,
        _ => contextra_ingest::format::FormatId::Tsv,
    };
    contextra_ingest::fuzz_support::run(format, &data[1..]);
});
