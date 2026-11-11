#![no_main]

use libfuzzer_sys::fuzz_target;
use contextra_vector::persistence::HnswHeader;

fuzz_target!(|data: &[u8]| {
    if let Ok(header) = HnswHeader::try_from_bytes(data) {
        let serialized = header.to_bytes();
        let _ = header.checksum();
        let _ = HnswHeader::try_from_bytes(&serialized);
    }
});
