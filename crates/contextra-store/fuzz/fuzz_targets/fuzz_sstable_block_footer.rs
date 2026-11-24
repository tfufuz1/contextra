#![no_main]

use contextra_store::sstable::parse_block_trailer;
use contextra_store::sstable::BloomFilter;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // 1. Fuzz Block Trailer Parser across format versions 0..=4
    for version in 0..=4 {
        let _ = parse_block_trailer(data, version);
    }

    // 2. Fuzz BloomFilter deserializer
    let _ = BloomFilter::from_bytes(data);
});
