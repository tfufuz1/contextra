#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = ctx_verify::canonical_json::verify_canonical_json(data);
});
