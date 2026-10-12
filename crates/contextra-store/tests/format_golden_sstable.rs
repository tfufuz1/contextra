mod golden;

use contextra_store::sstable::{BlockCache, SstableReader};
use golden::generate::{golden_dir, generate_all, verify_file_hash};
use std::fs;
use std::sync::Arc;

#[tokio::test]
async fn test_format_golden_sstable() {
    let dir = golden_dir();
    if !dir.join("MANIFEST.txt").exists() {
        generate_all().await;
    }

    // 1. Check file exists and verify hash in MANIFEST.txt
    let sst_filename = "sstable_mfsx.sst";
    let sst_path = dir.join(sst_filename);
    assert!(sst_path.exists(), "Golden SSTable MFSX file must exist");

    let is_valid_hash = verify_file_hash(&dir, sst_filename);
    assert!(
        is_valid_hash,
        "Golden SSTable MFSX file hash must match MANIFEST.txt entry"
    );

    // 2. Check binary trailer/header magic (MFSX)
    let bytes = fs::read(&sst_path).expect("read golden sstable");
    assert!(bytes.len() > 54, "SSTable file must be larger than 54-byte trailer");
    let trailer_magic = &bytes[bytes.len() - 4..];
    assert_eq!(
        trailer_magic, b"MFSX",
        "SSTable trailer magic must match MFSX"
    );

    // 3. Open SSTable via SstableReader
    let block_cache = Arc::new(BlockCache::new(10));
    let reader = SstableReader::open(&sst_path, block_cache)
        .await
        .expect("open golden sstable");

    assert_eq!(reader.format_version, 4, "Golden SSTable format version must be 4");

    // 4. Verify record lookups
    for i in 1..=5 {
        let k = format!("golden_sst_k{:02}", i);
        let expected_v = format!("golden_sst_v{:02}", i);

        let res = reader
            .get(k.as_bytes())
            .await
            .expect("get key from golden sstable");

        assert!(res.is_some(), "Key {} must exist in golden sstable", k);
        let (val_bytes, seq_no, tx_id) = res.unwrap();
        assert_eq!(val_bytes.as_ref(), expected_v.as_bytes());
        assert_eq!(seq_no, i as u64);
        assert_eq!(tx_id, i as u64);
    }

    // 5. Non-existent key lookup returns None
    let missing = reader
        .get(b"non_existent_key_999")
        .await
        .expect("get missing key");
    assert!(missing.is_none(), "Non-existent key must return None");
}
