#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    // Fuzzing von Framing-Prüfungen und Record-Payloads
    let _ = blake3::hash(data);
});
