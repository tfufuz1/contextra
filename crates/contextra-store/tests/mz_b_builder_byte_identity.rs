//! Differenztest: Byte-Identität des neuen modularisierten SstableBuilder (Gate 11).
//! Vergleicht die binäre Ausgabe des neuen Builders byte-für-byte mit der Referenzkopie aus origin/main.

mod mz_b_support;

use contextra_store::sstable::SstableBuilder;
use mz_b_support::legacy_builder::SstableBuilder as LegacySstableBuilder;
use tempfile::tempdir;

#[tokio::test]
async fn test_builder_byte_identity_v4_single_block() {
    let dir = tempdir().expect("tempdir");
    let path_new = dir.path().join("new_v4.sst");
    let path_legacy = dir.path().join("legacy_v4.sst");

    let mut b_new = SstableBuilder::create(&path_new)
        .await
        .expect("create new builder");
    let mut b_leg = LegacySstableBuilder::create(&path_legacy)
        .await
        .expect("create legacy builder");

    for i in 1..=50 {
        let key = format!("key_{:04}", i).into_bytes();
        let val = format!("val_{:04}", i).into_bytes();
        b_new
            .add(&key, &val, i as u64, i as u64)
            .await
            .expect("add new");
        b_leg
            .add(&key, &val, i as u64, i as u64)
            .await
            .expect("add legacy");
    }

    let meta_new = b_new.finish().await.expect("finish new");
    let meta_leg = b_leg.finish().await.expect("finish legacy");

    assert_eq!(meta_new.first_key, meta_leg.first_key);
    assert_eq!(meta_new.last_key, meta_leg.last_key);
    assert_eq!(meta_new.file_size, meta_leg.file_size);

    let bytes_new = std::fs::read(&path_new).expect("read new");
    let bytes_legacy = std::fs::read(&path_legacy).expect("read legacy");

    assert_eq!(
        bytes_new.len(),
        bytes_legacy.len(),
        "File lengths must match exactly"
    );
    assert_eq!(
        bytes_new, bytes_legacy,
        "Byte-for-byte binary identity required!"
    );
}

#[tokio::test]
async fn test_builder_byte_identity_v4_multiblock_with_bloom() {
    let dir = tempdir().expect("tempdir");
    let path_new = dir.path().join("new_mb.sst");
    let path_legacy = dir.path().join("legacy_mb.sst");

    let mut b_new = SstableBuilder::create(&path_new)
        .await
        .expect("create new builder")
        .with_block_size(512);
    let mut b_leg = LegacySstableBuilder::create(&path_legacy)
        .await
        .expect("create legacy builder")
        .with_block_size(512);

    for i in 1..=300 {
        let key = format!("key_mb_{:05}", i).into_bytes();
        let val = format!("payload_mb_{:05}_padding_data_to_fill_blocks", i).into_bytes();
        b_new
            .add(&key, &val, i as u64, (i * 2) as u64)
            .await
            .expect("add new");
        b_leg
            .add(&key, &val, i as u64, (i * 2) as u64)
            .await
            .expect("add legacy");
    }

    let meta_new = b_new.finish().await.expect("finish new");
    let meta_leg = b_leg.finish().await.expect("finish legacy");

    assert_eq!(meta_new.first_key, meta_leg.first_key);
    assert_eq!(meta_new.last_key, meta_leg.last_key);
    assert_eq!(meta_new.file_size, meta_leg.file_size);

    let bytes_new = std::fs::read(&path_new).expect("read new");
    let bytes_legacy = std::fs::read(&path_legacy).expect("read legacy");

    assert_eq!(
        bytes_new.len(),
        bytes_legacy.len(),
        "Multiblock file lengths must match exactly"
    );
    assert_eq!(
        bytes_new, bytes_legacy,
        "Multiblock binary identity required!"
    );
}

#[tokio::test]
async fn test_builder_byte_identity_v3_legacy() {
    let dir = tempdir().expect("tempdir");
    let path_new = dir.path().join("new_v3.sst");
    let path_legacy = dir.path().join("legacy_v3.sst");

    let mut b_new = SstableBuilder::create(&path_new)
        .await
        .expect("create new builder");
    b_new.set_format_version(3);
    let mut b_leg = LegacySstableBuilder::create(&path_legacy)
        .await
        .expect("create legacy builder");
    b_leg.set_format_version(3);

    for i in 1..=40 {
        let key = format!("v3_key_{:03}", i).into_bytes();
        let val = format!("v3_val_{:03}", i).into_bytes();
        b_new
            .add(&key, &val, i as u64, 0)
            .await
            .expect("add new v3");
        b_leg
            .add(&key, &val, i as u64, 0)
            .await
            .expect("add legacy v3");
    }

    let meta_new = b_new.finish().await.expect("finish new v3");
    let meta_leg = b_leg.finish().await.expect("finish legacy v3");

    assert_eq!(meta_new.file_size, meta_leg.file_size);

    let bytes_new = std::fs::read(&path_new).expect("read new v3");
    let bytes_legacy = std::fs::read(&path_legacy).expect("read legacy v3");

    assert_eq!(bytes_new.len(), bytes_legacy.len());
    assert_eq!(
        bytes_new, bytes_legacy,
        "Legacy V3 binary identity required!"
    );
}
