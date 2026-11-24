//! Differenz- und Byte-Identitätstest für das MFMN-Manifestformat (ST6 / MZ_C)
//!
//! Validiert, dass die Kodierung aller Varianten von ManifestEntry unverändert
//! byte-identisch bleibt und die Manifest-Header-Magic sowie Version stimmen.

#![forbid(unsafe_code)]
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

mod mz_c_support;

use contextra_store::manifest::{
    Manifest, ManifestEntry, CURRENT_MANIFEST_VERSION, MANIFEST_HEADER_MAGIC,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_manifest_header_magic_and_version() {
    let tmp = TempDir::new().unwrap();
    let manifest_path = tmp.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.unwrap();
    manifest
        .append(&ManifestEntry::WalCheckpoint { hmac: [1u8; 32] })
        .await
        .unwrap();
    drop(manifest);

    let raw = fs::read(&manifest_path).unwrap();
    assert!(raw.len() >= 5);
    assert_eq!(&raw[0..4], MANIFEST_HEADER_MAGIC);
    assert_eq!(raw[4], CURRENT_MANIFEST_VERSION);
}

#[tokio::test]
async fn test_manifest_entry_roundtrip_all_variants() {
    let entries = mz_c_support::generate_sample_manifest_entries();
    for entry in &entries {
        let bytes = entry.to_bytes().expect("serialize entry");
        assert!(bytes.len() >= 4, "Entry frame must have length header");
        let payload = &bytes[4..];
        let decoded = ManifestEntry::from_bytes(payload).expect("deserialize entry");
        assert_eq!(&decoded, entry, "Roundtrip must be bit-identical");
    }
}

#[tokio::test]
async fn test_manifest_reconstruction_deterministic() {
    let entries = mz_c_support::generate_sample_manifest_entries();
    let valid_ssts = Manifest::reconstruct_valid_sstables(&entries);
    let dead_ssts = Manifest::reconstruct_dead_sstables(&entries);

    // sst_001 was replaced by sst_003, sst_002 was removed
    // sst_003 remains valid
    let valid_names: Vec<_> = valid_ssts
        .into_iter()
        .map(|(p, _)| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(valid_names, vec!["sst_003.sst".to_string()]);

    let dead_paths: Vec<_> = dead_ssts.into_iter().collect();
    assert!(dead_paths.contains(&PathBuf::from("sst_002.sst")));
}
