#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let hints = contextra_ingest::detect::DetectHints {
        filename: Some("fuzz.bin"),
        mime: None,
    };
    let _ = contextra_ingest::detect::detect(data, &hints);
});
