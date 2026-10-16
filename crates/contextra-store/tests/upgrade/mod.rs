use contextra_store::erasure_ledger::{ErasureLedger, LedgerKey};
use contextra_store::format::{check_openable, FileKind, FormatError, FormatHeader};
use contextra_store::manifest::{Manifest, ManifestFormatTracker};
use contextra_store::sstable::{BlockCache, SstableReader};
use contextra_store::wal::Wal;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[tokio::test]
async fn test_upgrade_on_open_all_fixtures() {
    let base = fixture_dir();

    // 1. WAL Fixture Upgrade Open
    let wal_path = base.join("wal_v3.wal");
    assert!(wal_path.exists());
    let tmp_dir = tempdir().expect("tempdir");
    let test_wal = tmp_dir.path().join("wal_v3.wal");
    std::fs::copy(&wal_path, &test_wal).expect("copy wal");

    let km = std::sync::Arc::new(contextra_crypto::CryptoKey::from_raw_key([0x55u8; 32]).unwrap());
    let wal = Wal::open_with_key_manager(&test_wal, Some(km)).await.expect("open WAL v3");
    let (entries, _ver) = wal.replay_stream_with_version().await.expect("replay_stream_with_version");
    assert_eq!(entries.len(), 2);

    // 2. SSTable Fixture Upgrade Open
    let sst_path = base.join("sstable_v4.sst");
    assert!(sst_path.exists());
    let bc = Arc::new(BlockCache::new(10));
    let reader = SstableReader::open(&sst_path, bc).await.expect("open sstable v4");
    assert_eq!(reader.format_version, 4);
    let val = reader.get(b"sst_key_1").await.expect("get").expect("exists");
    assert_eq!(val.0.as_ref(), b"sst_value_1");

    // 3. Ledger Fixture Upgrade Open
    let ledger_path = base.join("erasure_v1.ledger");
    assert!(ledger_path.exists());
    let test_ledger_dir = tmp_dir.path().join("ledger");
    std::fs::create_dir_all(&test_ledger_dir).expect("create ledger dir");
    std::fs::copy(&ledger_path, test_ledger_dir.join("erasure.ledger")).expect("copy ledger");

    let key = LedgerKey::new([0x42u8; 32]);
    let (ledger_opt, status) = ErasureLedger::open_or_fail_closed(&test_ledger_dir, key, false, None)
        .await
        .expect("open ledger");
    assert!(ledger_opt.is_some());
    assert_eq!(status, contextra_store::erasure_ledger::LedgerOpenStatus::Ok);
    let ledger = ledger_opt.unwrap();
    assert_eq!(ledger.entries().len(), 1);

    // 4. Manifest Fixture Upgrade Open
    let manifest_path = base.join("manifest_v1.manifest");
    assert!(manifest_path.exists());
    let test_manifest_dir = tmp_dir.path().join("manifest");
    std::fs::create_dir_all(&test_manifest_dir).expect("create manifest dir");
    let test_manifest_path = test_manifest_dir.join("MANIFEST");
    std::fs::copy(&manifest_path, &test_manifest_path).expect("copy manifest v1");

    let manifest = Manifest::open(&test_manifest_path).await.expect("open upgraded manifest v1");
    assert_eq!(manifest.erasure_high_water_mark(), contextra_ports::erasure::ErasureMode::Off);
    let (seq, mac) = manifest.ledger_head().await;
    assert_eq!(seq, 0);
    assert_eq!(mac, [0u8; 32]);
    assert_ne!(manifest.backend_id(), 0);

    let entries = Manifest::load(&test_manifest_path).await.expect("load manifest");
    assert!(!entries.is_empty());
    let tracker = ManifestFormatTracker::from_entries(&entries);
    assert_eq!(tracker.max_sstable_version, 4);
    assert_eq!(tracker.max_wal_version, 3);
}

#[tokio::test]
async fn test_downgrade_rejection_newer_version_and_tracker() {
    // 1. Unsupported newer version check
    let header_newer = FormatHeader::new(*b"MFSX", 99);
    let err_newer = check_openable(FileKind::Sstable, &header_newer, 4);
    assert!(matches!(
        err_newer,
        Err(FormatError::UnsupportedNewerVersion { .. })
    ));

    // 2. Downgrade attempt rejection via ManifestFormatTracker
    let tracker = ManifestFormatTracker {
        max_sstable_version: 4,
        max_wal_version: 3,
        max_ledger_version: 1,
        id_width: Some(8),
        ..Default::default()
    };

    assert!(matches!(
        tracker.verify_file_version(FileKind::Sstable, 2),
        Err(FormatError::DowngradeAttempt { .. })
    ));
    assert!(matches!(
        tracker.verify_file_version(FileKind::Wal, 1),
        Err(FormatError::DowngradeAttempt { .. })
    ));
    assert!(matches!(
        tracker.verify_file_version(FileKind::Ledger, 0),
        Err(FormatError::DowngradeAttempt { .. })
    ));

    // 3. IdWidthMismatch rejection
    assert!(matches!(
        tracker.verify_id_width(16),
        Err(FormatError::IdWidthMismatch { expected: 16, found: 8 })
    ));
    assert!(tracker.verify_id_width(8).is_ok());
}
