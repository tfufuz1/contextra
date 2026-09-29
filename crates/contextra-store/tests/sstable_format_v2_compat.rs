// ZWECK: Interne und V2/V3 Compatibility Integrationstests für SSTable Format v3.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use bytes::{BufMut, Bytes, BytesMut};
use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader, SSTABLE_MAGIC_MFSX};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

fn create_block_cache() -> Arc<BlockCache> {
    Arc::new(BlockCache::new(10))
}

#[tokio::test]
async fn test_sstable_v3_write_and_read() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("v3_write_read.sst");
    let bc = create_block_cache();

    let mut builder = SstableBuilder::create(&sst_path)
        .await
        .expect("create builder");

    for i in 0..100 {
        let key = format!("key_{:04}", i);
        let val = format!("val_{:04}", i);
        builder
            .add(key.as_bytes(), val.as_bytes(), i as u64 + 1, 1)
            .await
            .expect("add key");
    }
    builder.finish().await.expect("finish builder");

    let reader = SstableReader::open(&sst_path, bc)
        .await
        .expect("open reader");

    assert!(reader.format_version >= 3, "Written SSTable should be format v3");

    for i in 0..100 {
        let key = format!("key_{:04}", i);
        let expected_val = format!("val_{:04}", i);
        let res = reader
            .get(key.as_bytes())
            .await
            .expect("get key")
            .expect("key must exist");
        assert_eq!(res.0.as_ref(), expected_val.as_bytes());
        assert_eq!(res.1, i as u64 + 1);
    }
}

#[tokio::test]
async fn test_sstable_v3_multi_version_duplicate_keys_in_same_block() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("v3_duplicate_keys_same_block.sst");
    let bc = create_block_cache();

    let mut builder = SstableBuilder::create(&sst_path)
        .await
        .expect("create builder");

    // Add 3 versions of the same key in descending sequence order (newest first)
    builder
        .add(b"shared_key", b"v3_payload", 300, 3)
        .await
        .expect("add v3");
    builder
        .add(b"shared_key", b"v2_payload", 200, 2)
        .await
        .expect("add v2");
    builder
        .add(b"shared_key", b"v1_payload", 100, 1)
        .await
        .expect("add v1");

    builder.finish().await.expect("finish builder");

    let reader = SstableReader::open(&sst_path, bc)
        .await
        .expect("open reader");

    // 1. Query at max_seq = 350 -> Should get v3
    let res3 = reader
        .get_at(b"shared_key", 350, u64::MAX)
        .await
        .expect("get_at 350")
        .expect("found v3");
    assert_eq!(res3.0.as_ref(), b"v3_payload");
    assert_eq!(res3.1, 300);

    // 2. Query at max_seq = 250 -> Should get v2
    let res2 = reader
        .get_at(b"shared_key", 250, u64::MAX)
        .await
        .expect("get_at 250")
        .expect("found v2");
    assert_eq!(res2.0.as_ref(), b"v2_payload");
    assert_eq!(res2.1, 200);

    // 3. Query at max_seq = 150 -> Should get v1
    let res1 = reader
        .get_at(b"shared_key", 150, u64::MAX)
        .await
        .expect("get_at 150")
        .expect("found v1");
    assert_eq!(res1.0.as_ref(), b"v1_payload");
    assert_eq!(res1.1, 100);

    // 4. Query at max_seq = 50 -> Should get None
    let res0 = reader
        .get_at(b"shared_key", 50, u64::MAX)
        .await
        .expect("get_at 50");
    assert!(res0.is_none());

    // 5. Verify iter() returns all 3 versions in sequence
    let all_entries = reader.iter().await.expect("iter entries");
    assert_eq!(all_entries.len(), 3);
    assert_eq!(
        all_entries[0],
        (Bytes::from_static(b"shared_key"), Bytes::from_static(b"v3_payload"), 300)
    );
    assert_eq!(
        all_entries[1],
        (Bytes::from_static(b"shared_key"), Bytes::from_static(b"v2_payload"), 200)
    );
    assert_eq!(
        all_entries[2],
        (Bytes::from_static(b"shared_key"), Bytes::from_static(b"v1_payload"), 100)
    );
}

#[tokio::test]
async fn test_sstable_v2_legacy_fixture_reading() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("v2_legacy.sst");
    let bc = create_block_cache();

    // Construct a legacy v2 SSTable block and file
    // Block v2 layout: [entries][8-byte bloom][u16 offsets...][u16 num_offsets]
    let mut block_data = BytesMut::new();

    // Entry 1: key=k1 (2 bytes len + k1), seq=1, tx=1, val=v1 (4 bytes len + v1)
    let off0 = block_data.len() as u16;
    block_data.put_u16_le(2);
    block_data.put_slice(b"k1");
    block_data.put_u64_le(1); // seq
    block_data.put_u64_le(1); // tx
    block_data.put_u32_le(2); // val_len
    block_data.put_slice(b"v1");

    // Entry 2: key=k2
    let off1 = block_data.len() as u16;
    block_data.put_u16_le(2);
    block_data.put_slice(b"k2");
    block_data.put_u64_le(2); // seq
    block_data.put_u64_le(1); // tx
    block_data.put_u32_le(2); // val_len
    block_data.put_slice(b"v2");

    // 8-byte bloom filter (all bits set so lookups pass)
    block_data.put_u64_le(u64::MAX);
    block_data.put_u16_le(off0);
    block_data.put_u16_le(off1);
    block_data.put_u16_le(2); // 2 offsets

    let raw_block = block_data.freeze();
    let crc = crc32fast::hash(&raw_block);

    let mut file = File::create(&sst_path).await.expect("create v2 file");
    let mut block_with_crc = Vec::new();
    block_with_crc.extend_from_slice(&crc.to_le_bytes());
    block_with_crc.extend_from_slice(&raw_block);
    file.write_all(&block_with_crc).await.expect("write block");

    let block_len = block_with_crc.len() as u64;

    // Index: (k2, 0)
    let mut index_data = BytesMut::new();
    index_data.put_u16_le(2);
    index_data.put_slice(b"k2");
    index_data.put_u64_le(0); // offset
    let raw_index = index_data.freeze();
    let index_crc = crc32fast::hash(&raw_index);

    let mut index_with_crc = Vec::new();
    index_with_crc.extend_from_slice(&index_crc.to_le_bytes());
    index_with_crc.extend_from_slice(&raw_index);
    file.write_all(&index_with_crc).await.expect("write index");

    let bloom_offset = block_len + index_with_crc.len() as u64;
    let index_offset = block_len;

    // Whole-SSTable Bloom Filter: 8 bytes num_hashes (1), 8 bytes num_bits (64), 8 bytes word
    let mut bloom_filter_bytes = BytesMut::new();
    bloom_filter_bytes.put_u64_le(1);
    bloom_filter_bytes.put_u64_le(64);
    bloom_filter_bytes.put_u64_le(u64::MAX);
    let raw_bloom = bloom_filter_bytes.freeze();
    let bloom_crc = crc32fast::hash(&raw_bloom);

    let mut bloom_with_crc = Vec::new();
    bloom_with_crc.extend_from_slice(&bloom_crc.to_le_bytes());
    bloom_with_crc.extend_from_slice(&raw_bloom);
    file.write_all(&bloom_with_crc).await.expect("write bloom");

    // Trailer v2: [min_tx(8)][max_tx(8)][min_seq(8)][max_seq(8)][bloom_off(8)][index_off(8)][version=2(2)][magic(4)] = 54 bytes
    file.write_u64_le(1).await.unwrap(); // min_tx
    file.write_u64_le(1).await.unwrap(); // max_tx
    file.write_u64_le(1).await.unwrap(); // min_seq
    file.write_u64_le(2).await.unwrap(); // max_seq
    file.write_u64_le(bloom_offset).await.unwrap();
    file.write_u64_le(index_offset).await.unwrap();
    file.write_u16_le(2).await.unwrap(); // version = 2
    file.write_u32_le(SSTABLE_MAGIC_MFSX).await.unwrap(); // magic = SSTABLE_MAGIC_MFSX
    file.sync_all().await.unwrap();

    let reader = SstableReader::open(&sst_path, bc)
        .await
        .expect("open v2 reader");

    assert_eq!(reader.format_version, 2, "Reader must detect format v2");

    let res_k1 = reader.get(b"k1").await.expect("get k1").expect("k1 exists");
    assert_eq!(res_k1.0.as_ref(), b"v1");

    let res_k2 = reader.get(b"k2").await.expect("get k2").expect("k2 exists");
    assert_eq!(res_k2.0.as_ref(), b"v2");
}

#[tokio::test]
async fn test_block_bloom_fpr_measurement() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("bloom_fpr.sst");
    let bc = create_block_cache();

    let mut builder = SstableBuilder::create(&sst_path)
        .await
        .expect("create builder");

    // Insert 50 keys in block
    for i in 0..50 {
        let key = format!("contained_key_{:04}", i);
        let val = format!("val_{:04}", i);
        builder
            .add(key.as_bytes(), val.as_bytes(), i as u64 + 1, 1)
            .await
            .expect("add key");
    }
    builder.finish().await.expect("finish builder");

    let reader = SstableReader::open(&sst_path, bc)
        .await
        .expect("open reader");

    // Measure block bloom FPR with 1000 negative keys
    let mut fp_count = 0;
    for i in 0..1000 {
        let key = format!("negative_key_{:04}", i);
        let (_whole_bloom, _in_range, block_bloom_passed, _found) =
            reader.lookup_metrics(key.as_bytes()).await;
        if block_bloom_passed {
            fp_count += 1;
        }
    }

    let fpr = fp_count as f64 / 1000.0;
    assert!(
        fpr <= 0.01,
        "Block Bloom FPR {:.4} must be <= 1% (fp_count: {})",
        fpr,
        fp_count
    );
}

#[tokio::test]
async fn test_block_size_near_and_exceeding_u16_boundary() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("large_block.sst");
    let bc = create_block_cache();

    let mut builder = SstableBuilder::create(&sst_path)
        .await
        .expect("create builder");

    // Create a large key/value entry where offset will exceed u16::MAX (65535 bytes)
    // Write 700 entries of ~100 bytes each into one large block
    let large_val = vec![b'x'; 100];
    for i in 0..700 {
        let key = format!("large_block_key_{:04}", i);
        builder
            .add(key.as_bytes(), &large_val, i as u64 + 1, 1)
            .await
            .expect("add entry");
    }
    builder.finish().await.expect("finish builder");

    let reader = SstableReader::open(&sst_path, bc)
        .await
        .expect("open reader");

    for i in (0..700).step_by(50) {
        let key = format!("large_block_key_{:04}", i);
        let res = reader
            .get(key.as_bytes())
            .await
            .expect("get entry")
            .expect("entry must exist");
        assert_eq!(res.0.len(), 100);
    }
}

#[tokio::test]
async fn test_chaos_bitflip_corruption_returns_error_no_panic() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("bitflip_test.sst");
    let bc = create_block_cache();

    let mut builder = SstableBuilder::create(&sst_path)
        .await
        .expect("create builder");

    for i in 0..50 {
        let key = format!("key_{:02}", i);
        let val = format!("val_{:02}", i);
        builder
            .add(key.as_bytes(), val.as_bytes(), i as u64 + 1, 1)
            .await
            .expect("add key");
    }
    builder.finish().await.expect("finish builder");

    // Read full file
    let mut data = tokio::fs::read(&sst_path).await.expect("read file");

    // Perform bitflips across multiple locations
    let file_len = data.len();
    for offset in [10, file_len / 2, file_len - 15] {
        if offset < file_len {
            data[offset] ^= 0xFF;
        }
    }
    tokio::fs::write(&sst_path, data).await.expect("write corrupted file");

    // Opening corrupted reader must return Result::Err or fail safely on get() without panicking
    let open_res = SstableReader::open(&sst_path, bc).await;
    match open_res {
        Ok(reader) => {
            let get_res = reader.get(b"key_10").await;
            // Either returns checksum error or Ok(None) due to corrupted bloom/index, but NO panic
            if let Err(e) = get_res {
                println!("Safely caught expected error on get(): {:?}", e);
            }
        }
        Err(e) => {
            println!("Safely caught expected error on open(): {:?}", e);
        }
    }
}
