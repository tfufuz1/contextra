mod golden;

use contextra_store::manifest::{Manifest, ManifestEntry, MANIFEST_HEADER_MAGIC};
use golden::generate::{golden_dir, generate_all, verify_file_hash};
use std::fs;
use std::path::PathBuf;

#[tokio::test]
async fn test_format_golden_manifest() {
    let dir = golden_dir();
    if !dir.join("MANIFEST.txt").exists() {
        generate_all().await;
    }

    // 1. Check file exists and verify hash in MANIFEST.txt
    let manifest_filename = "manifest_mfmn.manifest";
    let manifest_path = dir.join(manifest_filename);
    assert!(manifest_path.exists(), "Golden Manifest MFMN file must exist");

    let is_valid_hash = verify_file_hash(&dir, manifest_filename);
    assert!(
        is_valid_hash,
        "Golden Manifest MFMN file hash must match MANIFEST.txt entry"
    );

    // 2. Check binary header magic (MFMN)
    let bytes = fs::read(&manifest_path).expect("read golden manifest");
    assert!(bytes.len() >= 4, "Manifest file must have at least header length");
    assert_eq!(
        &bytes[0..4],
        MANIFEST_HEADER_MAGIC,
        "Manifest header must match MFMN"
    );

    // 3. Load entries via Manifest::load
    let entries = Manifest::load(&manifest_path)
        .await
        .expect("load golden manifest");

    assert_eq!(entries.len(), 4, "Golden manifest must contain 4 entries");

    // Entry 0: FormatVersionRecord
    match &entries[0] {
        ManifestEntry::FormatVersionRecord {
            file_kind,
            max_version,
            migrated_from,
        } => {
            assert_eq!(*file_kind, 0);
            assert_eq!(*max_version, 4);
            assert_eq!(*migrated_from, 3);
        }
        other => panic!("Expected FormatVersionRecord, got {:?}", other),
    }

    // Entry 1: Add
    match &entries[1] {
        ManifestEntry::Add { path, max_tx } => {
            assert_eq!(path, &PathBuf::from("sstable_mfsx.sst"));
            assert_eq!(*max_tx, 100);
        }
        other => panic!("Expected Add, got {:?}", other),
    }

    // Entry 2: WalCheckpoint
    match &entries[2] {
        ManifestEntry::WalCheckpoint { hmac } => {
            assert_eq!(hmac, &[0x55u8; 32]);
        }
        other => panic!("Expected WalCheckpoint, got {:?}", other),
    }

    // Entry 3: RollbackComplete
    match &entries[3] {
        ManifestEntry::RollbackComplete { target_tx } => {
            assert_eq!(*target_tx, 99);
        }
        other => panic!("Expected RollbackComplete, got {:?}", other),
    }
}
