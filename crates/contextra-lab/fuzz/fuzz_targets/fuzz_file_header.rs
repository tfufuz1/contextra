#![no_main]

use contextra_lab::format::file_header::FileHeader;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = FileHeader::parse(data);
});
