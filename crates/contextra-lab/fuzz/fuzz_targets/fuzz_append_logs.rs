#![no_main]

use contextra_lab::format::append_logs::AppendLogFile;
use contextra_lab::format::file_header::FileKind;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = AppendLogFile::parse(data, FileKind::Journal);
    let _ = AppendLogFile::parse(data, FileKind::GateLog);
    let _ = AppendLogFile::parse(data, FileKind::Ledger);
});
