#![no_main]

use contextra_lab::format::header_ab::HeaderAb;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = HeaderAb::parse(data);
});
