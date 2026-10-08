#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    block_data: Vec<u8>,
    key: Vec<u8>,
    offsets_start: usize,
    num_offsets: usize,
    // FIX(2026-10-07): Pass is_v3 parameter required by block binary search APIs
    is_v3: bool,
}

fuzz_target!(|input: FuzzInput| {
    // FIX(2026-10-07): Supply input.is_v3 as 5th argument
    let _ = contextra_store::sstable::binary_search_index_in_block(
        &input.block_data,
        input.offsets_start,
        input.num_offsets,
        &input.key,
        input.is_v3,
    );
    let _ = contextra_store::sstable::binary_search_entry_in_block(
        &input.block_data,
        input.offsets_start,
        input.num_offsets,
        &input.key,
        input.is_v3,
    );
    let _ = contextra_store::sstable::block_binary_search(
        &input.block_data,
        input.offsets_start,
        input.num_offsets,
        &input.key,
        input.is_v3,
    );
});
