

#![no_main]

use contextra_vector::persistence::HnswHeader;
use libfuzzer_sys::fuzz_target;

/// Reines Fuzz-Primitiv zur Entkopplung für Smoke-Treiber ohne libFuzzer.
pub fn fuzz_one(data: &[u8]) {
    if let Ok(header) = HnswHeader::try_from_bytes(data) {
        let bytes = header.to_bytes();
        let _ = header.checksum();
        let _ = HnswHeader::try_from_bytes(&bytes);
    }
}

fuzz_target!(|data: &[u8]| {
    fuzz_one(data);
});
