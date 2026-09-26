use bytes::Bytes;
use contextra_store::sstable::{
    binary_search_entry_in_block, binary_search_index_in_block, block_binary_search, BlockBuilder,
    BlockCache, SstableBuilder, SstableReader,
};
use proptest::prelude::*;
use std::sync::Arc;
use tempfile::TempDir;

fn make_valid_block() -> Bytes {
    let mut builder = BlockBuilder::new(4096);
    for i in 0..15 {
        let k = format!("key_{:04}", i);
        let v = format!("value_{:04}_payload_data", i);
        builder.add(k.as_bytes(), v.as_bytes(), i as u64, i as u64);
    }
    builder.build()
}

#[derive(Debug, Clone)]
enum Mutation {
    BitFlip { bit: usize },
    Truncate { len: usize },
    Insert { pos: usize, bytes: Vec<u8> },
    OverwriteField { pos: usize, val: u32 },
    ArbitrarySlice { pos: usize, bytes: Vec<u8> },
}

fn mutation_strategy(max_len: usize) -> impl Strategy<Value = Mutation> {
    prop_oneof![
        (0..max_len * 8).prop_map(|bit| Mutation::BitFlip { bit }),
        (0..=max_len).prop_map(|len| Mutation::Truncate { len }),
        (0..=max_len, prop::collection::vec(any::<u8>(), 1..=32))
            .prop_map(|(pos, bytes)| Mutation::Insert { pos, bytes }),
        (0..max_len, any::<u32>()).prop_map(|(pos, val)| Mutation::OverwriteField { pos, val }),
        (0..max_len, prop::collection::vec(any::<u8>(), 1..=16))
            .prop_map(|(pos, bytes)| Mutation::ArbitrarySlice { pos, bytes }),
    ]
}

fn apply_mutations(base: &[u8], mutations: &[Mutation]) -> Vec<u8> {
    let mut data = base.to_vec();
    for m in mutations {
        if data.is_empty() {
            break;
        }
        match m {
            Mutation::BitFlip { bit } => {
                let byte_idx = (*bit / 8) % data.len();
                let bit_idx = *bit % 8;
                data[byte_idx] ^= 1 << bit_idx;
            }
            Mutation::Truncate { len } => {
                let target_len = *len % (data.len() + 1);
                data.truncate(target_len);
            }
            Mutation::Insert { pos, bytes } => {
                let insert_pos = *pos % (data.len() + 1);
                data.splice(insert_pos..insert_pos, bytes.iter().copied());
            }
            Mutation::OverwriteField { pos, val } => {
                let p = *pos % data.len();
                if p + 4 <= data.len() {
                    let b = val.to_le_bytes();
                    let end = p + 4;
                    data[p..end].copy_from_slice(&b);
                }
            }
            Mutation::ArbitrarySlice { pos, bytes } => {
                let start = *pos % data.len();
                let count = bytes.len().min(data.len() - start);
                data[start..start + count].copy_from_slice(&bytes[..count]);
            }
        }
    }
    data
}

fn test_block_decoders(block_data: &[u8], search_key: &[u8]) {
    let n = block_data.len();
    if n < 2 {
        let _ = block_binary_search(block_data, 0, 0, search_key);
        let _ = binary_search_entry_in_block(block_data, 0, 0, search_key);
        let _ = binary_search_index_in_block(block_data, 0, 0, search_key);
        return;
    }

    let num_offsets = u16::from_le_bytes([block_data[n - 2], block_data[n - 1]]) as usize;
    let offsets_len = num_offsets.saturating_mul(2);
    let offsets_start = n.saturating_sub(2 + offsets_len + 8);

    let _ = block_binary_search(block_data, offsets_start, num_offsets, search_key);
    let _ = binary_search_entry_in_block(block_data, offsets_start, num_offsets, search_key);
    let _ = binary_search_index_in_block(block_data, offsets_start, num_offsets, search_key);

    // Also test with randomized offsets_start / num_offsets parameters
    let arbitrary_start = if n > 0 { search_key.len() % n } else { 0 };
    let arbitrary_num = if n > 0 { search_key.len() % (n + 1) } else { 0 };
    let _ = block_binary_search(block_data, arbitrary_start, arbitrary_num, search_key);
    let _ = binary_search_entry_in_block(block_data, arbitrary_start, arbitrary_num, search_key);
    let _ = binary_search_index_in_block(block_data, arbitrary_start, arbitrary_num, search_key);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn prop_sstable_zero_copy_block_decode_no_panics(
        mutations in prop::collection::vec(mutation_strategy(2048), 1..=10),
        search_key in prop::collection::vec(any::<u8>(), 0..=64)
    ) {
        let base_block = make_valid_block();
        let mutated = apply_mutations(&base_block, &mutations);

        // Prove that block decoding functions NEVER panic under arbitrary byte corruption
        let res = std::panic::catch_unwind(|| {
            test_block_decoders(&mutated, &search_key);
        });
        prop_assert!(res.is_ok(), "Block decoding panicked on mutated byte buffer!");
    }
}

async fn test_corrupted_sstable_file_reader(mutated_block: &[u8]) {
    let tmp = match TempDir::new() {
        Ok(t) => t,
        Err(_) => return,
    };
    let path = tmp.path().join("corrupted.sst");

    // Create a baseline valid SSTable file first
    let mut builder = match SstableBuilder::create(&path).await {
        Ok(b) => b,
        Err(_) => return,
    };
    for i in 0..10 {
        let _ = builder
            .add(
                format!("key_{:04}", i).as_bytes(),
                format!("val_{:04}", i).as_bytes(),
                i as u64,
                i as u64,
            )
            .await;
    }
    if builder.finish().await.is_err() {
        return;
    }

    // Overwrite the first block in the SSTable file with mutated block bytes
    let crc = crc32fast::hash(mutated_block);
    let mut block_with_crc = Vec::with_capacity(4 + mutated_block.len());
    block_with_crc.extend_from_slice(&crc.to_le_bytes());
    block_with_crc.extend_from_slice(mutated_block);

    if let Ok(mut file) = tokio::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .await
    {
        use tokio::io::AsyncWriteExt;
        let _ = file.write_all(&block_with_crc).await;
        let _ = file.sync_all().await;
    }

    let cache = Arc::new(BlockCache::new(1024));
    // Attempt to open and read from the corrupted SSTable
    if let Ok(reader) = SstableReader::open(&path, cache).await {
        let reader_arc = Arc::new(reader);
        let _ = reader_arc.get(b"key_0005").await;
        let _ = reader_arc.iter().await;

        if let Ok(mut stream) = reader_arc.stream().await {
            let _ = stream.next().await;
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    #[test]
    fn prop_sstable_file_reader_no_panics_under_corruption(
        mutations in prop::collection::vec(mutation_strategy(2048), 1..=5)
    ) {
        let base_block = make_valid_block();
        let mutated = apply_mutations(&base_block, &mutations);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            test_corrupted_sstable_file_reader(&mutated).await;
        });
    }
}

#[test]
fn test_v_len_u32_max_overflow_handling() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // Construct an entry with v_len = u32::MAX (0xFFFFFFFF)
        // Entry structure:
        // key_len (u16): 4 -> "key0"
        // key: b"key0"
        // seq_no (u64): 1
        // tx_id (u64): 1
        // val_len (u32): 0xFFFFFFFF (u32::MAX boundary)
        let mut entry_bytes = Vec::new();
        entry_bytes.extend_from_slice(&4u16.to_le_bytes()); // key_len
        entry_bytes.extend_from_slice(b"key0"); // key
        entry_bytes.extend_from_slice(&1u64.to_le_bytes()); // seq_no
        entry_bytes.extend_from_slice(&1u64.to_le_bytes()); // tx_id
        entry_bytes.extend_from_slice(&u32::MAX.to_le_bytes()); // v_len = u32::MAX!

        let mut block_data = entry_bytes;
        // Add bloom filter (8 bytes)
        block_data.extend_from_slice(&0u64.to_le_bytes());
        // Add offset to first entry (u16 offset 0)
        block_data.extend_from_slice(&0u16.to_le_bytes());
        // Add num_offsets (1)
        block_data.extend_from_slice(&1u16.to_le_bytes());

        // Test decoder functions directly with v_len = u32::MAX block
        let num_offsets = 1usize;
        let offsets_start = block_data.len() - 2 - 2 - 8;

        let res_bsearch = block_binary_search(&block_data, offsets_start, num_offsets, b"key0");
        assert!(
            res_bsearch.is_ok(),
            "block_binary_search must return Ok(...) or Err without panic"
        );

        let res_entry = binary_search_entry_in_block(&block_data, offsets_start, num_offsets, b"key0");
        assert!(
            res_entry.is_ok(),
            "binary_search_entry_in_block must return Ok(...) or Err without panic"
        );

        // Test via SstableReader / SstableStream file reading
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("overflow_test.sst");

        let mut builder = SstableBuilder::create(&path).await.unwrap();
        builder.add(b"key0", b"dummy_val", 1, 1).await.unwrap();
        builder.finish().await.unwrap();

        // Overwrite block with v_len = u32::MAX constructed block
        let crc = crc32fast::hash(&block_data);
        let mut block_with_crc = Vec::with_capacity(4 + block_data.len());
        block_with_crc.extend_from_slice(&crc.to_le_bytes());
        block_with_crc.extend_from_slice(&block_data);

        {
            use tokio::io::AsyncWriteExt;
            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .await
                .unwrap();
            file.write_all(&block_with_crc).await.unwrap();
            file.sync_all().await.unwrap();
        }

        let cache = Arc::new(BlockCache::new(1024));
        if let Ok(reader) = SstableReader::open(&path, cache).await {
            let reader_arc = Arc::new(reader);
            // SstableReader::get must handle v_len = u32::MAX gracefully and return Err or None
            let get_res = reader_arc.get(b"key0").await;
            assert!(
                get_res.is_err() || get_res.as_ref().unwrap().is_none(),
                "Expected Err or None for u32::MAX v_len, got: {:?}",
                get_res
            );

            if let Ok(mut stream) = reader_arc.stream().await {
                let stream_res = stream.next().await;
                assert!(
                    stream_res.is_err() || stream_res.as_ref().unwrap().is_none(),
                    "Expected Err or None from stream for u32::MAX v_len, got: {:?}",
                    stream_res
                );
            }
        }
    });
}
