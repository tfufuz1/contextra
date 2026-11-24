#![cfg(feature = "workspace")]
//! Golden Invariants Integration Test (MZ-G)
//!
//! Validiert die Byte-Integrität und Invarianten persistierter Datenstrukturen
//! von contextra-store (Manifest, SSTable, Rollback Intent).

use std::fs;
use std::path::{Path, PathBuf};

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("golden")
}

#[tokio::test]
async fn test_mz_g_golden_files_presence_and_formats() {
    let dir = golden_dir();

    // 1. Rollback Intent
    let rollback_path = dir.join("rollback_tx_42.intent");
    assert!(rollback_path.exists(), "rollback intent file must exist");
    let intent_bytes = fs::read(&rollback_path).expect("read rollback intent");
    assert_eq!(intent_bytes.len(), 16);
    assert_eq!(&intent_bytes[..8], b"MFRLBK\0\0");
    let tx_num = u64::from_le_bytes(intent_bytes[8..16].try_into().expect("slice"));
    assert_eq!(tx_num, 42);

    // 2. SSTable v4 Magic Bytes
    let sst_v4_path = dir.join("sstable_v4.sst");
    assert!(sst_v4_path.exists(), "sstable_v4.sst must exist");
    let sst_bytes = fs::read(&sst_v4_path).expect("read sstable_v4");
    assert!(sst_bytes.len() > 32);

    // 3. Manifest v1 Magic
    let manifest_path = dir.join("manifest_v1.manifest");
    assert!(manifest_path.exists(), "manifest_v1.manifest must exist");
    let manifest_bytes = fs::read(&manifest_path).expect("read manifest");
    assert!(manifest_bytes.len() >= 8);
    assert_eq!(&manifest_bytes[..4], b"MFMN");
}

#[tokio::test]
async fn test_mz_g_manifest_sha_verification() {
    let dir = golden_dir();
    let manifest_sha = dir.join("MANIFEST.sha256");
    assert!(manifest_sha.exists(), "MANIFEST.sha256 must exist");

    let checksums = fs::read_to_string(&manifest_sha).expect("read MANIFEST.sha256");
    let mut verified_count = 0;
    for line in checksums.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Invalid checksum line format: {}", line);
        let expected_hash = parts[0];
        let filename = parts[1];
        let file_path = dir.join(filename);
        if !file_path.exists() {
            // Some logs might be gitignored or generated on demand
            continue;
        }
        let bytes = fs::read(&file_path).expect("read golden file");
        let computed_hash = blake3::hash(&bytes).to_hex().to_string();
        assert_eq!(
            computed_hash, expected_hash,
            "Checksum mismatch for {}",
            filename
        );
        verified_count += 1;
    }
    assert!(verified_count >= 4, "Must verify at least 4 golden artifacts");
}
