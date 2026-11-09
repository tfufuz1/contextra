//! Tests für Modellsicherheit, Hash-Verifikation und Path-Traversal-Abwehr.

use contextra_infer_onnx::config::{verify_path_safety, ModelHash};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_path_traversal_prevention() {
    let base = Path::new("/tmp/models");
    let safe_relative = Path::new("subdir/model.onnx");
    let res = verify_path_safety(base, safe_relative);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), base.join("subdir/model.onnx"));

    // Traversalversuch mit `..`
    let malicious = Path::new("../etc/passwd");
    let res = verify_path_safety(base, malicious);
    assert!(res.is_err());
    let err_msg = format!("{}", res.unwrap_err());
    assert!(err_msg.contains("Path traversal"));
}

#[test]
fn test_blake3_hash_verification() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("model.bin");
    let content = b"hello contextra onnx model binary";

    let mut file = File::create(&file_path).unwrap();
    file.write_all(content).unwrap();
    drop(file);

    let actual_hash = blake3::hash(content).to_hex().to_string();
    let valid_verifier = ModelHash::Blake3(actual_hash.clone());
    assert!(valid_verifier.verify_file(&file_path).is_ok());

    let invalid_verifier = ModelHash::Blake3("00000000000000000000000000000000".into());
    let err = invalid_verifier.verify_file(&file_path);
    assert!(err.is_err());
}
