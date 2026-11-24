//! # Golden-Korpus-Verifikation und Rückwärts-Lesetest für `contextra-erasure`
//!
//! Stellt sicher, dass kanonische Formate, Signaturen und Typ-Serialisierungen
//! byteweise stabil bleiben und ohne Panic deserialisiert werden können.

#![cfg(feature = "signing")]

use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

use contextra_erasure::deletion_proof::DeletionProof;
use contextra_erasure::error::ErasureError;

#[test]
fn test_golden_manifest_checksums() {
    let manifest_path = Path::new("tests/golden/MANIFEST.sha256");
    let golden_dir = if manifest_path.exists() {
        Path::new("tests/golden")
    } else {
        Path::new("crates/contextra-erasure/tests/golden")
    };
    assert!(
        golden_dir.join("MANIFEST.sha256").exists(),
        "MANIFEST.sha256 missing"
    );

    let manifest_content = fs::read_to_string(golden_dir.join("MANIFEST.sha256"))
        .expect("Failed to read MANIFEST.sha256");

    for line in manifest_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Invalid line in MANIFEST.sha256: {line}");
        let expected_hash = parts[0];
        let filename = parts[1];

        let file_path = golden_dir.join(filename);
        assert!(file_path.exists(), "Golden file missing: {filename}");

        let file_bytes =
            fs::read(&file_path).unwrap_or_else(|_| panic!("Failed to read {filename}"));
        let mut hasher = Sha256::new();
        hasher.update(&file_bytes);
        let actual_hash = format!("{:x}", hasher.finalize());

        assert_eq!(
            actual_hash, expected_hash,
            "Checksum mismatch for golden file {filename}"
        );
    }
}

#[test]
fn test_golden_backward_read() {
    let golden_dir = if Path::new("tests/golden").exists() {
        Path::new("tests/golden")
    } else {
        Path::new("crates/contextra-erasure/tests/golden")
    };

    // 1. DeletionProof v1/v2/v3 JSON rückwärts lesen
    for version in [1, 2, 3] {
        let filename = format!("deletion_proof_v{version}.json");
        let path = golden_dir.join(&filename);
        let content =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {filename}"));
        let proof: DeletionProof = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to deserialize {filename}: {e}"));
        assert_eq!(proof.signature_version, version);
    }

    // 2. ErasureError Display JSON
    let err_path = golden_dir.join("erasure_error_display.json");
    let err_content =
        fs::read_to_string(&err_path).expect("Failed to read erasure_error_display.json");
    let err_map: std::collections::BTreeMap<String, String> =
        serde_json::from_str(&err_content).expect("Failed to parse erasure_error_display.json");
    assert!(!err_map.is_empty());
    assert!(err_map.contains_key("Corrupt"));
}

#[test]
fn test_golden_corruption_bitflip_and_truncation() {
    let golden_dir = if Path::new("tests/golden").exists() {
        Path::new("tests/golden")
    } else {
        Path::new("crates/contextra-erasure/tests/golden")
    };

    // 1. Bitflip Test auf DeletionProof JSON
    let path = golden_dir.join("deletion_proof_v3.json");
    let content = fs::read(&path).expect("Failed to read deletion_proof_v3.json");

    // An deterministischer Position ein Byte flippen
    let mut corrupted = content.clone();
    if let Some(byte) = corrupted.get_mut(50) {
        *byte ^= 0xFF;
    }
    let res: Result<DeletionProof, _> = serde_json::from_slice(&corrupted);
    // Muss entweder Err liefern oder ungültige Signatur/Werte; niemals Panic
    if let Ok(proof) = res {
        let dummy_key = [0u8; 32];
        let verify_res = proof.verify(&dummy_key);
        assert!(verify_res.is_err() || !verify_res.unwrap_or(true));
    }

    // 2. Truncation Test
    let truncated = &content[..content.len() / 2];
    let res_trunc: Result<DeletionProof, _> = serde_json::from_slice(truncated);
    assert!(res_trunc.is_err(), "Truncated payload must return Err");
}

#[test]
#[ignore]
fn generate_golden_files() {
    use contextra_erasure::deletion_proof::{
        DeletionLayer, DeletionProof, DeletionScope, ExcludedScope, LayerCleanupProof,
    };
    use contextra_erasure::ed25519_proof::DeletionProofKeyPair;
    use contextra_erasure::types_local::{DocId, TenantId, TxId};
    use rand::SeedableRng;

    let golden_dir = if Path::new("tests/golden").exists() {
        Path::new("tests/golden").to_path_buf()
    } else {
        Path::new("crates/contextra-erasure/tests/golden").to_path_buf()
    };
    fs::create_dir_all(&golden_dir).unwrap();

    let tenant_id = TenantId(42);
    let scope = DeletionScope::Document {
        doc_id: DocId(101),
        tenant_id,
    };

    // V1
    let golden_v1_json = r#"{
  "scope": {
    "Document": {
      "doc_id": 101,
      "tenant_id": 42
    }
  },
  "deleted_keys_hash": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
  "deleted_after_tx": 10,
  "timestamp": 0,
  "signature": [1,2,3,4],
  "covered_layers": [],
  "excluded_scopes": [],
  "signature_version": 1
}"#;
    let proof_v1: DeletionProof = serde_json::from_str(golden_v1_json).unwrap();
    let json_v1 = serde_json::to_string_pretty(&proof_v1).unwrap();
    fs::write(golden_dir.join("deletion_proof_v1.json"), json_v1).unwrap();

    // V2
    let proof_v2 = DeletionProof::create(
        scope.clone(),
        vec![b"key_a".to_vec()],
        TxId(20),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        b"hmac_key_32_bytes_long_secret!!",
    )
    .unwrap();
    let json_v2 = serde_json::to_string_pretty(&proof_v2).unwrap();
    fs::write(golden_dir.join("deletion_proof_v2.json"), json_v2).unwrap();

    // V3
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keypair = DeletionProofKeyPair::generate_with_rng(&mut rng);
    let proof_v3 = DeletionProof::create_v3(
        scope,
        vec![b"key_a".to_vec()],
        TxId(30),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();
    let json_v3 = serde_json::to_string_pretty(&proof_v3).unwrap();
    fs::write(golden_dir.join("deletion_proof_v3.json"), json_v3).unwrap();

    // Error Display
    let errors: Vec<(&str, ErasureError)> = vec![
        (
            "UnsupportedProofVersion",
            ErasureError::UnsupportedProofVersion(99),
        ),
        (
            "InvalidProofSignature",
            ErasureError::InvalidProofSignature,
        ),
        (
            "KeyRevoked",
            ErasureError::KeyRevoked("key-xyz".to_string()),
        ),
        ("Corrupt", ErasureError::Corrupt),
        ("Capacity", ErasureError::Capacity),
        ("SlotMissing", ErasureError::SlotMissing),
        ("DiskReserve", ErasureError::DiskReserve),
        ("Refused", ErasureError::Refused),
        (
            "ReplacementMissing",
            ErasureError::ReplacementMissing,
        ),
        ("UpdateMissing", ErasureError::UpdateMissing),
        ("SameIdentity", ErasureError::SameIdentity),
        (
            "RestrictedReadAuditRequired",
            ErasureError::RestrictedReadAuditRequired,
        ),
    ];

    let mut err_map = std::collections::BTreeMap::new();
    for (name, err) in errors {
        err_map.insert(name.to_string(), err.to_string());
    }
    let err_json = serde_json::to_string_pretty(&err_map).unwrap();
    fs::write(golden_dir.join("erasure_error_display.json"), err_json).unwrap();
}
