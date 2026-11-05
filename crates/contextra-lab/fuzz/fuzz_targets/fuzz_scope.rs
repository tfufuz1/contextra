#![no_main]
use contextra_lab::format::scope::ScopeFile;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let dummy_key = [0x42u8; 32];
    let _ = ScopeFile::parse(data, &dummy_key);
});
