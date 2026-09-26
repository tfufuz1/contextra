use contextra_store::sstable::{
    binary_search_entry_in_block, block_binary_search, BlockBuilder, BlockCache, SstableBuilder,
    SstableReader,
};
use proptest::prelude::*;
use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;

/// Constructs a valid raw block buffer with at least 10 entries using `BlockBuilder`.
fn create_valid_raw_block() -> Vec<u8> {
    let mut builder = BlockBuilder::new(4096);
    for i in 0..15 {
        let key = format!("key_{:04}", i);
        let val = format!("value_payload_{:06}", i * 10);
        builder.add(key.as_bytes(), val.as_bytes(), i as u64, (i + 1) as u64);
    }
    builder.build().to_vec()
}

#[derive(Debug, Clone)]
enum BlockMutation {
    BitFlip { pos: usize, bit_idx: u8 },
    Truncate { len: usize },
    InsertRandomBytes { pos: usize, bytes: Vec<u8> },
    OverwriteU32Max { pos: usize },
}

fn mutation_strategy(max_len: usize) -> impl Strategy<Value = BlockMutation> {
    prop_oneof![
        (0..max_len, 0..8u8).prop_map(|(pos, bit_idx)| BlockMutation::BitFlip { pos, bit_idx }),
        (0..max_len).prop_map(|len| BlockMutation::Truncate { len }),
        (0..max_len, prop::collection::vec(any::<u8>(), 1..=32))
            .prop_map(|(pos, bytes)| BlockMutation::InsertRandomBytes { pos, bytes }),
        (0..max_len).prop_map(|pos| BlockMutation::OverwriteU32Max { pos }),
    ]
}

fn apply_mutation(mut data: Vec<u8>, mutation: &BlockMutation) -> Vec<u8> {
    if data.is_empty() {
        return data;
    }
    match mutation {
        BlockMutation::BitFlip { pos, bit_idx } => {
            let idx = pos % data.len();
            data[idx] ^= 1 << (bit_idx % 8);
        }
        BlockMutation::Truncate { len } => {
            let new_len = len % data.len();
            data.truncate(new_len);
        }
        BlockMutation::InsertRandomBytes { pos, bytes } => {
            let idx = pos % (data.len() + 1);
            let mut new_data = Vec::with_capacity(data.len() + bytes.len());
            new_data.extend_from_slice(&data[..idx]);
            new_data.extend_from_slice(bytes);
            new_data.extend_from_slice(&data[idx..]);
            data = new_data;
        }
        BlockMutation::OverwriteU32Max { pos } => {
            let idx = pos % data.len();
            if idx + 4 <= data.len() {
                data[idx..idx + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            }
        }
    }
    data
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn proptest_sstable_zero_copy_block_decode(
        mutations in prop::collection::vec(mutation_strategy(256), 1..=10),
        search_key in prop::collection::vec(any::<u8>(), 0..=32),
        offsets_start in 0..1024usize,
        num_offsets in 0..128usize
    ) {
        let valid_block = create_valid_raw_block();
        let mut mutated_block = valid_block;

        for m in &mutations {
            mutated_block = apply_mutation(mutated_block, m);
        }

        // Test block_binary_search (must never panic regardless of mutated bytes)
        let _ = block_binary_search(&mutated_block, offsets_start, num_offsets, &search_key);

        // Test binary_search_entry_in_block (must never panic)
        let _ = binary_search_entry_in_block(&mutated_block, offsets_start, num_offsets, &search_key);

        // Calculate derived num_offsets and offsets_start if block ends with u16 num_offsets
        if mutated_block.len() >= 2 {
            let n = mutated_block.len();
            let derived_num_offsets = u16::from_le_bytes([mutated_block[n - 2], mutated_block[n - 1]]) as usize;
            let offsets_len = derived_num_offsets.saturating_mul(2);
            if n >= 2 + offsets_len + 8 {
                let derived_offsets_start = n - 2 - offsets_len;
                let _ = block_binary_search(&mutated_block, derived_offsets_start, derived_num_offsets, &search_key);
                let _ = binary_search_entry_in_block(&mutated_block, derived_offsets_start, derived_num_offsets, &search_key);
            }
        }
    }
}

/// Targeted regression test proving that a block with `v_len = u32::MAX`
/// (boundary condition for integer overflow `ep + v_len`) results in a clean `Err`
/// and NEVER causes an arithmetic overflow panic or silent data corruption.
#[tokio::test]
async fn test_u32_max_v_len_boundary_no_panic() {
    let valid_block = create_valid_raw_block();
    assert!(valid_block.len() >= 30, "Valid block must contain entries");

    // Locating v_len in the first entry:
    // entry 0: key_len(u16, 2 bytes) + key + seq_no(8) + tx_id(8) + val_len(u32, 4)
    let k_len = u16::from_le_bytes([valid_block[0], valid_block[1]]) as usize;
    let v_len_offset = 2 + k_len + 8 + 8;
    assert!(
        v_len_offset + 4 <= valid_block.len(),
        "v_len_offset out of bounds in test setup"
    );

    // Mutate v_len of first entry to u32::MAX
    let mut corrupted_block = valid_block;
    corrupted_block[v_len_offset..v_len_offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());

    // Create an actual SSTable file containing the corrupted data block using SstableBuilder
    let tmp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = tmp_file.path().to_path_buf();

    let mut builder = SstableBuilder::create(&file_path)
        .await
        .expect("Failed to create SstableBuilder");

    for i in 0..15 {
        let key = format!("key_{:04}", i);
        let val = format!("value_payload_{:06}", i * 10);
        builder
            .add(key.as_bytes(), val.as_bytes(), i as u64, (i + 1) as u64)
            .await
            .expect("Add key");
    }
    let _meta = builder.finish().await.expect("Finish SSTable");

    // Overwrite the first data block in the written SSTable file with corrupted_block
    let mut raw_file_bytes = std::fs::read(&file_path).expect("Read SSTable file");
    let raw_len = raw_file_bytes.len();
    assert!(
        raw_len > corrupted_block.len(),
        "SSTable file size smaller than block"
    );

    // Overwrite block payload starting after 4-byte CRC header
    if raw_len >= 4 + corrupted_block.len() {
        raw_file_bytes[4..4 + corrupted_block.len()].copy_from_slice(&corrupted_block);
        // Recompute CRC for the block so SstableReader CRC check passes and reaches block decode
        let new_crc = crc32fast::hash(&corrupted_block);
        raw_file_bytes[0..4].copy_from_slice(&new_crc.to_le_bytes());
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open(&file_path)
            .expect("Open SSTable for corrupt write");
        file.write_all(&raw_file_bytes).expect("Write corrupted file");
        file.sync_all().expect("Sync corrupted file");
    }

    let cache = Arc::new(BlockCache::new(10));
    let reader_res = SstableReader::open(&file_path, cache).await;

    if let Ok(reader) = reader_res {
        // Attempt point lookups and iter on the corrupted SSTable
        let get_res = reader.get(b"key_0000").await;
        assert!(
            get_res.is_err(),
            "Expected Err due to u32::MAX v_len out of bounds, got: {:?}",
            get_res
        );

        let iter_res = reader.iter().await;
        assert!(
            iter_res.is_err(),
            "Expected Err due to u32::MAX v_len during iter, got: {:?}",
            iter_res
        );
    }
}
