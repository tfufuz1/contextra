// ZWECK: Regressionstests für SSTable Multi-Version Point-Lookups (P02 Review F-02).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::TOMBSTONE_BIT;
use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
use std::sync::Arc;
use tempfile::TempDir;

fn create_block_cache() -> Arc<BlockCache> {
    Arc::new(BlockCache::new(10))
}

#[tokio::test]
async fn test_single_block_multi_version_v3_and_v4() {
    for version in [3, 4] {
        let tmp = TempDir::new().expect("temp dir");
        let sst_path = tmp.path().join(format!("multi_version_v{}.sst", version));
        let bc = create_block_cache();

        // 1. Build SSTable with duplicate versions for same user key
        {
            let mut builder = SstableBuilder::create(&sst_path)
                .await
                .expect("create builder");
            builder.set_format_version(version);

            // Add duplicate key versions in descending sequence order (as enforced by SstableBuilder / Invariant I-1)
            builder
                .add(b"user_key_1", b"val_v3", 300, 1)
                .await
                .expect("add v3");
            builder
                .add(b"user_key_1", b"val_v2", 200, 1)
                .await
                .expect("add v2");
            builder
                .add(b"user_key_1", b"val_v1", 100, 1)
                .await
                .expect("add v1");

            builder.finish().await.expect("finish builder");
        }

        // 2. Open reader
        let reader = SstableReader::open(&sst_path, bc)
            .await
            .expect("open reader");
        assert_eq!(reader.format_version, version);

        // 3. Unbounded lookup MUST yield newest version (seq 300)
        let latest = reader
            .get(b"user_key_1")
            .await
            .expect("get user_key_1")
            .expect("found user_key_1");
        assert_eq!(latest.0.as_ref(), b"val_v3");
        assert_eq!(latest.1, 300);

        // 4. Point lookups with snapshot seq filtering
        let snapshot_250 = reader
            .get_at(b"user_key_1", 250, u64::MAX)
            .await
            .expect("get_at 250")
            .expect("found at 250");
        assert_eq!(snapshot_250.0.as_ref(), b"val_v2");
        assert_eq!(snapshot_250.1, 200);

        let snapshot_150 = reader
            .get_at(b"user_key_1", 150, u64::MAX)
            .await
            .expect("get_at 150")
            .expect("found at 150");
        assert_eq!(snapshot_150.0.as_ref(), b"val_v1");
        assert_eq!(snapshot_150.1, 100);

        let snapshot_50 = reader
            .get_at(b"user_key_1", 50, u64::MAX)
            .await
            .expect("get_at 50");
        assert!(snapshot_50.is_none(), "seq 50 precedes all versions");
    }
}

#[tokio::test]
async fn test_multi_version_with_tombstone() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("tombstone_multi_version.sst");
    let bc = create_block_cache();

    {
        let mut builder = SstableBuilder::create(&sst_path).await.unwrap();
        builder.set_format_version(4);

        // Tombstone version at seq 200 (newer)
        builder
            .add(b"user_key_del", b"", 200 | TOMBSTONE_BIT, 1)
            .await
            .unwrap();
        // Active version at seq 100 (older)
        builder
            .add(b"user_key_del", b"initial_val", 100, 1)
            .await
            .unwrap();

        builder.finish().await.unwrap();
    }

    let reader = SstableReader::open(&sst_path, bc).await.unwrap();

    // Latest lookup MUST observe the tombstone
    let latest = reader.get(b"user_key_del").await.unwrap().unwrap();
    assert!(latest.0.is_empty());
    assert_ne!(latest.1 & TOMBSTONE_BIT, 0);
    assert_eq!(latest.1 & !TOMBSTONE_BIT, 200);

    // Snapshot lookup at seq 150 MUST observe active version
    let snap_150 = reader
        .get_at(b"user_key_del", 150, u64::MAX)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(snap_150.0.as_ref(), b"initial_val");
    assert_eq!(snap_150.1, 100);
}

#[tokio::test]
async fn test_multi_block_multi_version_span() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("multi_block_span.sst");
    let bc = create_block_cache();

    {
        let mut builder = SstableBuilder::create(&sst_path).await.unwrap();
        builder.set_format_version(4);

        // Fill keys before target key
        for i in 0..100 {
            let k = format!("a_key_{:04}", i);
            let v = "x".repeat(500); // force block splits
            builder
                .add(k.as_bytes(), v.as_bytes(), 10, 1)
                .await
                .unwrap();
        }

        // Add newer version of span_key first (seq 200)
        builder
            .add(b"span_key", b"new_span_val", 200, 1)
            .await
            .unwrap();

        // Add older version of span_key second (seq 100)
        builder
            .add(b"span_key", b"old_span_val", 100, 1)
            .await
            .unwrap();

        // Fill trailing keys (must be > "span_key")
        for i in 0..100 {
            let k = format!("z_key_{:04}", i);
            let v = "z".repeat(500);
            builder
                .add(k.as_bytes(), v.as_bytes(), 10, 1)
                .await
                .unwrap();
        }

        builder.finish().await.unwrap();
    }

    let reader = SstableReader::open(&sst_path, bc).await.unwrap();

    // Latest lookup MUST find newer version
    let latest = reader.get(b"span_key").await.unwrap().unwrap();
    assert_eq!(latest.0.as_ref(), b"new_span_val");
    assert_eq!(latest.1, 200);

    // Snapshot lookup at seq 150 MUST find older version
    let snap_150 = reader
        .get_at(b"span_key", 150, u64::MAX)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(snap_150.0.as_ref(), b"old_span_val");
    assert_eq!(snap_150.1, 100);
}
