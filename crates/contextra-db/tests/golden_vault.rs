//! Golden test for contextra-db / volatile-vault

use std::fs;
use std::path::PathBuf;

#[test]
fn test_golden_vault_manifest() {
    let manifest_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/MANIFEST.sha256");
    assert!(manifest_path.exists(), "Manifest must exist");
    let content = fs::read_to_string(&manifest_path).expect("read manifest");
    assert!(content.contains("vault_errors.json"));
}

#[test]
fn test_golden_vault_backward_read() {
    let file_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/vault_errors.json");
    let bytes = fs::read(&file_path).expect("read file");
    let val: serde_json::Value = serde_json::from_slice(&bytes).expect("valid json");
    assert_eq!(val["version"], "1.0");
}
