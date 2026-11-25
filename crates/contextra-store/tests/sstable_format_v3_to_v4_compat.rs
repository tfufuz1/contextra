// ZWECK: Format v3 und v4 Kompatibilitäts-Integrationstests (S-04 Phase 3).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::TOMBSTONE_BIT;
use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

fn fixture_v3_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sst_v3_multi_block.sst")
}

fn fixture_v1_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sst_v1_single_version.sst")
}

fn create_block_cache() -> Arc<BlockCache> {
    Arc::new(BlockCache::new(10))
}

#[tokio::test]
async fn test_v3_fixture_multi_block_readable() {
    let path = fixture_v3_path();
    assert!(path.exists(), "v3 fixture file must exist");
    let bc = create_block_cache();
    let reader = SstableReader::open(&path, bc)
        .await
        .expect("open v3 fixture");
    assert_eq!(reader.format_version, 3, "Fixture format version must be 3");

    // Read first key and last key
    assert_eq!(reader.first_key().as_ref(), b"v3_key_0000");

    // Query key with duplicate versions
    let res = reader.get(b"v3_key_0050").await.unwrap().unwrap();
    assert_eq!(res.0.as_ref(), b"v3_value_0050_version_2");
    assert_eq!(res.1, 1000);

    let res_at_old = reader
        .get_at(b"v3_key_0050", 100, u64::MAX)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        res_at_old.0.as_ref(),
        b"v3_value_0050_version_1_padding_payload_extra_bytes"
    );
    assert_eq!(res_at_old.1, 51);

    // Query tombstone key v3_key_0100
    let res_tomb = reader.get(b"v3_key_0100").await.unwrap().unwrap();
    assert!(res_tomb.0.is_empty());
    assert_ne!(res_tomb.1 & TOMBSTONE_BIT, 0);

    // Iteration over fixture
    let all_entries = reader.iter().await.expect("iter v3 fixture");
    assert!(
        all_entries.len() >= 500,
        "v3 fixture must contain at least 500 entries"
    );
}

#[tokio::test]
async fn test_v1_v2_legacy_fixture_readable() {
    let path = fixture_v1_path();
    if path.exists() {
        let bc = create_block_cache();
        let reader = SstableReader::open(&path, bc)
            .await
            .expect("open v1 fixture");
        assert!(reader.format_version <= 2, "Format version should be <= 2");
        let all_entries = reader.iter().await.expect("iter v1 fixture");
        assert!(!all_entries.is_empty(), "v1 fixture should contain entries");
    }
}

#[tokio::test]
async fn test_mixed_v3_and_v4_sstables_in_same_store() {
    let tmp = TempDir::new().expect("temp dir");
    let v3_path = tmp.path().join("mixed_v3.sst");
    let v4_path = tmp.path().join("mixed_v4.sst");
    let bc = create_block_cache();

    // 1. Build v3 SSTable
    {
        let mut builder_v3 = SstableBuilder::create(&v3_path)
            .await
            .expect("create v3 builder");
        builder_v3.set_format_version(3);
        for i in 0..100 {
            let k = format!("shared_key_{:04}", i);
            let v = format!("v3_val_{:04}", i);
            builder_v3
                .add(k.as_bytes(), v.as_bytes(), i as u64 + 1, 1)
                .await
                .expect("add v3 key");
        }
        builder_v3.finish().await.expect("finish v3");
    }

    // 2. Build v4 SSTable
    {
        let mut builder_v4 = SstableBuilder::create(&v4_path)
            .await
            .expect("create v4 builder");
        // Default format_version is 4
        for i in 50..150 {
            let k = format!("shared_key_{:04}", i);
            let v = format!("v4_val_{:04}", i);
            builder_v4
                .add(k.as_bytes(), v.as_bytes(), (i as u64) + 1000, 2)
                .await
                .expect("add v4 key");
        }
        builder_v4.finish().await.expect("finish v4");
    }

    // 3. Open both readers in same store
    let reader_v3 = SstableReader::open(&v3_path, bc.clone())
        .await
        .expect("open v3 reader");
    let reader_v4 = SstableReader::open(&v4_path, bc)
        .await
        .expect("open v4 reader");

    assert_eq!(reader_v3.format_version, 3);
    assert_eq!(reader_v4.format_version, 4);

    // Verify v3 lookup
    let res_v3 = reader_v3.get(b"shared_key_0010").await.unwrap().unwrap();
    assert_eq!(res_v3.0.as_ref(), b"v3_val_0010");

    // Verify v4 lookup
    let res_v4 = reader_v4.get(b"shared_key_0060").await.unwrap().unwrap();
    assert_eq!(res_v4.0.as_ref(), b"v4_val_0060");
}

#[tokio::test]
async fn test_compaction_simulation_mixed_v3_v4_merges_to_v4() {
    let tmp = TempDir::new().expect("temp dir");
    let v3_path = tmp.path().join("input_v3.sst");
    let output_v4_path = tmp.path().join("compacted_v4.sst");
    let bc = create_block_cache();

    // Create v3 SSTable
    {
        let mut builder_v3 = SstableBuilder::create(&v3_path).await.expect("create v3");
        builder_v3.set_format_version(3);
        builder_v3.add(b"k1", b"v3_val1", 10, 1).await.unwrap();
        builder_v3.add(b"k2", b"v3_val2", 10, 1).await.unwrap();
        builder_v3.finish().await.unwrap();
    }

    let reader_v3 = SstableReader::open(&v3_path, bc.clone()).await.unwrap();
    assert_eq!(reader_v3.format_version, 3);

    // Merge v3 entries into a new v4 SSTable
    let entries_v3 = reader_v3.iter().await.unwrap();
    let mut builder_v4 = SstableBuilder::create(&output_v4_path).await.unwrap();
    for (k, v, seq) in entries_v3 {
        builder_v4.add(&k, &v, seq, 1).await.unwrap();
    }
    builder_v4.finish().await.unwrap();

    // Verify compacted SSTable is format v4 and contains identical data
    let reader_compacted = SstableReader::open(&output_v4_path, bc).await.unwrap();
    assert_eq!(reader_compacted.format_version, 4);

    let k1 = reader_compacted.get(b"k1").await.unwrap().unwrap();
    assert_eq!(k1.0.as_ref(), b"v3_val1");
    assert_eq!(k1.1, 10);

    let k2 = reader_compacted.get(b"k2").await.unwrap().unwrap();
    assert_eq!(k2.0.as_ref(), b"v3_val2");
    assert_eq!(k2.1, 10);
}
