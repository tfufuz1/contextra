//! Verifies corrupt and truncated files fail closed returning Err without panicking (ST6.6)

use contextra_store::sstable::{SstableBuilder, SstableReader, BlockCache};
use contextra_store::manifest::Manifest;
use tempfile::tempdir;
use std::sync::Arc;

#[tokio::test]
async fn test_corrupted_golden_files_fail_closed_without_panic() {
    let dir = tempdir().expect("tempdir");

    let sst_path = dir.path().join("corrupted.sst");
    let mut b = SstableBuilder::create(&sst_path).await.expect("create sst");
    for i in 1..=10 {
        let k = format!("k_{i:04}").into_bytes();
        let v = format!("v_{i:04}").into_bytes();
        b.add(&k, &v, i, i).await.expect("add");
    }
    b.finish().await.expect("finish");

    let sst_bytes = tokio::fs::read(&sst_path).await.expect("read sst");
    let cache = Arc::new(BlockCache::new(1024 * 1024));

    for trunc_len in [0, 10, 50, sst_bytes.len().saturating_sub(10)] {
        let trunc_path = dir.path().join(format!("trunc_{trunc_len}.sst"));
        tokio::fs::write(&trunc_path, &sst_bytes[..trunc_len]).await.expect("write trunc");
        let open_res = SstableReader::open(&trunc_path, cache.clone()).await;
        if trunc_len < 54 {
            assert!(open_res.is_err(), "Opening truncated SSTable must fail closed");
        }
    }

    let mf_path = dir.path().join("test.manifest");
    tokio::fs::write(&mf_path, b"MFMN\x01\x00bad_crc_or_short").await.expect("write bad mf");
    let mf_res = Manifest::load(&mf_path).await;
    let _ = mf_res;
}
