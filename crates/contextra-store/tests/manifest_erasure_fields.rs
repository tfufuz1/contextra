use contextra_ports::erasure::ErasureMode;
use contextra_store::manifest::{Manifest, ManifestEntry, MANIFEST_HEADER_MAGIC};
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn test_manifest_upgrade_on_open_defaults_and_persistent_backend_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    // 1. Create a version 1 MANIFEST file manually on disk
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&manifest_path)
        .await
        .expect("create file");

    let mut header = Vec::new();
    header.extend_from_slice(MANIFEST_HEADER_MAGIC);
    header.push(1u8); // Version 1
    file.write_all(&header).await.expect("write header");

    let add_entry = ManifestEntry::Add {
        path: "sst-1.sst".into(),
        max_tx: 10,
    };
    file.write_all(&add_entry.to_bytes().unwrap())
        .await
        .expect("write add entry");
    file.flush().await.expect("flush");
    drop(file);

    // 2. Open version 1 manifest with current Manifest (v2) handle
    let manifest = Manifest::open(&manifest_path)
        .await
        .expect("open upgraded manifest");

    // Default values for upgraded v1 manifest:
    // - erasure_high_water_mark: ErasureMode::Off
    assert_eq!(manifest.erasure_high_water_mark(), ErasureMode::Off);
    // - ledger_head: (0, [0u8; 32])
    let (seq, mac) = manifest.ledger_head().await;
    assert_eq!(seq, 0);
    assert_eq!(mac, [0u8; 32]);
    // - backend_id: newly generated u64
    let backend_id_1 = manifest.backend_id();
    assert_ne!(backend_id_1, 0, "Generated backend_id must be non-zero");

    drop(manifest);

    // 3. Re-open manifest and verify backend_id remains identical
    let manifest_reopened = Manifest::open(&manifest_path)
        .await
        .expect("reopen manifest");

    assert_eq!(
        manifest_reopened.backend_id(),
        backend_id_1,
        "backend_id must remain unchanged on subsequent open"
    );
    assert_eq!(
        manifest_reopened.erasure_high_water_mark(),
        ErasureMode::Off
    );
}

#[tokio::test]
async fn test_manifest_downgrade_rejection_newer_version() {
    let dir = tempfile::tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    // Create a MANIFEST file with unsupported future version 99
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&manifest_path)
        .await
        .expect("create file");

    let mut header = Vec::new();
    header.extend_from_slice(MANIFEST_HEADER_MAGIC);
    header.push(99u8); // Version 99
    file.write_all(&header).await.expect("write header");
    file.flush().await.expect("flush");
    drop(file);

    // Opening must fail with UnsupportedNewerVersion error
    let res = Manifest::open(&manifest_path).await;
    assert!(res.is_err(), "Opening manifest with version 99 must fail");

    let err = res.err().unwrap();
    match err {
        contextra_types::ContextraError::Storage(msg) => {
            assert!(msg.contains("Manifest") || msg.contains("version"));
        }
        other => panic!("Expected Storage error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_erasure_high_water_mark_monotonicity() {
    let dir = tempfile::tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    assert_eq!(manifest.erasure_high_water_mark(), ErasureMode::Off);

    // Raise to Assured
    manifest
        .set_erasure_high_water_mark(ErasureMode::Assured)
        .await
        .expect("set Assured");
    assert_eq!(manifest.erasure_high_water_mark(), ErasureMode::Assured);

    // Raise to Attested
    manifest
        .set_erasure_high_water_mark(ErasureMode::Attested)
        .await
        .expect("set Attested");
    assert_eq!(manifest.erasure_high_water_mark(), ErasureMode::Attested);

    // Attempt to lower back to Assured (should be no-op)
    manifest
        .set_erasure_high_water_mark(ErasureMode::Assured)
        .await
        .expect("attempt set Assured");
    assert_eq!(
        manifest.erasure_high_water_mark(),
        ErasureMode::Attested,
        "Watermark must not be lowered"
    );

    // Attempt to lower back to Off (should be no-op)
    manifest
        .set_erasure_high_water_mark(ErasureMode::Off)
        .await
        .expect("attempt set Off");
    assert_eq!(
        manifest.erasure_high_water_mark(),
        ErasureMode::Attested,
        "Watermark must not be lowered"
    );

    // Rollover test: verify watermark is preserved across rollover
    let live_entries = vec![ManifestEntry::Add {
        path: "sst-1.sst".into(),
        max_tx: 10,
    }];
    manifest
        .rollover(&live_entries)
        .await
        .expect("rollover manifest");

    drop(manifest);

    // Reopen after rollover and check watermark
    let manifest_reopened = Manifest::open(&manifest_path)
        .await
        .expect("reopen manifest");

    assert_eq!(
        manifest_reopened.erasure_high_water_mark(),
        ErasureMode::Attested,
        "Watermark must be preserved across rollover and reopen"
    );

    // Lowering attempt after reopen should still be rejected
    manifest_reopened
        .set_erasure_high_water_mark(ErasureMode::Off)
        .await
        .expect("attempt set Off after reopen");
    assert_eq!(
        manifest_reopened.erasure_high_water_mark(),
        ErasureMode::Attested
    );
}

#[tokio::test]
async fn test_ledger_head_and_crc_corruption_detection() {
    let dir = tempfile::tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    let test_mac = [0x55u8; 32];
    manifest
        .set_ledger_head(42, test_mac)
        .await
        .expect("set ledger head");

    let (seq, mac) = manifest.ledger_head().await;
    assert_eq!(seq, 42);
    assert_eq!(mac, test_mac);

    drop(manifest);

    // Reopen and check ledger head
    let manifest_reopened = Manifest::open(&manifest_path)
        .await
        .expect("reopen manifest");
    let (seq2, mac2) = manifest_reopened.ledger_head().await;
    assert_eq!(seq2, 42);
    assert_eq!(mac2, test_mac);

    drop(manifest_reopened);

    // Corrupt byte in ledger head record
    let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");
    let len = file_bytes.len();
    file_bytes[len - 5] ^= 0xFF; // Flip byte inside payload
    tokio::fs::write(&manifest_path, file_bytes)
        .await
        .expect("write corrupt");

    // Loading corrupted manifest must return Err
    let load_res = Manifest::load(&manifest_path).await;
    assert!(
        load_res.is_err(),
        "Corrupted LedgerHeadRecord byte must trigger CRC mismatch error"
    );
}
