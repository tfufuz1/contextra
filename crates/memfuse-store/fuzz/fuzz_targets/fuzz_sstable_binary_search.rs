#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    block_data: Vec<u8>,
    key: Vec<u8>,
    offsets_start: usize,
    num_offsets: usize,
}

fuzz_target!(|input: FuzzInput| {
    let _ = memfuse_store::sstable::binary_search_index_in_block(
        &input.block_data,
        input.offsets_start,
        input.num_offsets,
        &input.key,
    );
    let _ = memfuse_store::sstable::binary_search_entry_in_block(
        &input.block_data,
        input.offsets_start,
        input.num_offsets,
        &input.key,
    );
    let _ = memfuse_store::sstable::block_binary_search(
        &input.block_data,
        input.offsets_start,
        input.num_offsets,
        &input.key,
    );
});
