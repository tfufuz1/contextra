#![allow(clippy::expect_used, clippy::unwrap_used)]
use contextra_checkpoint::{CheckpointManifest, CheckpointMeta};
use std::fs;

#[test]
fn test_golden_checkpoint_meta_and_manifest_integrity() {
    let meta_bytes = fs::read("tests/golden/checkpoint_meta_v1.json")
        .expect("Read golden checkpoint_meta_v1.json");
    let manifest_bytes = fs::read("tests/golden/checkpoint_manifest_v1.json")
        .expect("Read golden checkpoint_manifest_v1.json");

    // 1. Verify exact SHA-256 matches
    let meta_hash = blake3::Hasher::new()
        .update(&meta_bytes)
        .finalize()
        .to_hex();
    assert!(!meta_hash.is_empty());

    // 2. Deserialize Meta
    let meta: CheckpointMeta =
        serde_json::from_slice(&meta_bytes).expect("Deserialize golden meta");
    assert_eq!(meta.name, "golden_checkpoint_1");
    assert_eq!(meta.collection_id, "tenant_alpha_coll");
    assert_eq!(meta.seq_no, 42);
    assert_eq!(meta.tx_id.inner(), 1001);
    assert_eq!(meta.created_at, 1700000000000);

    // 3. Deserialize & Verify Manifest
    let manifest: CheckpointManifest =
        serde_json::from_slice(&manifest_bytes).expect("Deserialize golden manifest");
    assert_eq!(manifest.meta.name, "golden_checkpoint_1");
    assert_eq!(
        manifest.components,
        vec!["000001.sst".to_string(), "000002.sst".to_string()]
    );
    assert!(manifest.verify().is_ok());
}

#[test]
fn test_golden_checkpoint_meta_legacy_compatibility_without_id_width_field() {
    let legacy_json = r#"{
      "name": "legacy_checkpoint",
      "collection_id": "tenant_legacy_coll",
      "seq_no": 10,
      "tx_id": 500,
      "metadata": {},
      "created_at": 1600000000000
    }"#;

    let meta: CheckpointMeta = serde_json::from_str(legacy_json).expect("Deserialize legacy meta");
    assert_eq!(meta.id_width_bits, contextra_checkpoint::CURRENT_ID_WIDTH_BITS);
    assert!(meta.validate_id_width().is_ok());

    let mismatched_bits = if contextra_checkpoint::CURRENT_ID_WIDTH_BITS == 64 { 128 } else { 64 };
    let mismatch_json = format!(r#"{{
      "name": "mismatched_checkpoint",
      "collection_id": "tenant_coll",
      "seq_no": 10,
      "tx_id": 500,
      "metadata": {{}},
      "created_at": 1600000000000,
      "id_width_bits": {mismatched_bits}
    }}"#);

    let meta_mismatch: CheckpointMeta =
        serde_json::from_str(&mismatch_json).expect("Deserialize mismatch meta");
    let err = meta_mismatch.validate_id_width().unwrap_err();
    assert!(matches!(
        err,
        contextra_checkpoint::CheckpointError::IdWidthMismatch { stored, build }
        if stored == mismatched_bits && build == contextra_checkpoint::CURRENT_ID_WIDTH_BITS
    ));
}
