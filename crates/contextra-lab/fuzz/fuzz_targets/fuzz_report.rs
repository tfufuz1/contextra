#![no_main]

use contextra_lab::format::report::ErasureReport;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = ErasureReport::from_json_str(s);
    }
});
