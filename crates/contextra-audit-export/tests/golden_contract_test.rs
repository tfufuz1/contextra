#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

//! Golden format verification and backward compatibility tests for `contextra-audit-export`.

use std::fs;
use std::path::Path;

use contextra_audit_export::bsi_mapping::BsiMappingEntry;

#[test]
fn test_golden_audit_export_manifest_and_payload_exact() {
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let manifest_path = golden_dir.join("MANIFEST.sha256");
    assert!(manifest_path.exists(), "MANIFEST.sha256 must exist");

    let manifest_str = fs::read_to_string(&manifest_path).expect("read manifest");
    for line in manifest_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Expected '<hash> <filename>'");
        let expected_hash = parts[0];
        let filename = parts[1];

        let _content = fs::read(golden_dir.join(filename)).expect("read golden file");
        let output = std::process::Command::new("sha256sum")
            .arg(golden_dir.join(filename))
            .output()
            .expect("sha256sum failed");
        let out_str = String::from_utf8_lossy(&output.stdout);
        let actual_hash = out_str.split_whitespace().next().unwrap_or("");
        assert_eq!(actual_hash, expected_hash, "Hash mismatch for {filename}");
    }
}

#[test]
fn test_backward_read_audit_export_golden_files() {
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let json_bytes = fs::read(golden_dir.join("golden_register.json")).expect("read json");
    let md_bytes = fs::read(golden_dir.join("golden_register.md")).expect("read md");
    let bsi_json_bytes = fs::read(golden_dir.join("bsi_mapping.json")).expect("read bsi json");
    let bsi_md_bytes = fs::read(golden_dir.join("bsi_mapping.md")).expect("read bsi md");

    assert!(!json_bytes.is_empty());
    assert!(!md_bytes.is_empty());

    let bsi_entries: Vec<BsiMappingEntry> =
        serde_json::from_slice(&bsi_json_bytes).expect("deserialize bsi mapping");
    assert_eq!(bsi_entries.len(), 4);

    let md_str = String::from_utf8(md_bytes).expect("utf8 md");
    assert!(md_str.contains("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)"));

    let bsi_md_str = String::from_utf8(bsi_md_bytes).expect("utf8 bsi md");
    assert!(bsi_md_str.contains("# BSI Grundschutz / TR-02102 Kryptographische Zuordnung"));
}

#[test]
fn test_golden_audit_export_tamper_and_truncation_rejects_without_panic() {
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let json_bytes = fs::read(golden_dir.join("golden_register.json")).expect("read json");
    let bsi_json_bytes = fs::read(golden_dir.join("bsi_mapping.json")).expect("read bsi json");

    // 1. Corrupted json
    let mut corrupted_json = json_bytes.clone();
    if corrupted_json.len() > 10 {
        corrupted_json[5] = b'X';
        let res: Result<serde_json::Value, _> = serde_json::from_slice(&corrupted_json);
        assert!(res.is_err(), "Corrupted JSON must return Err, never panic");
    }

    // 2. Truncated bsi json
    if bsi_json_bytes.len() > 10 {
        let truncated = &bsi_json_bytes[..bsi_json_bytes.len() / 2];
        let res: Result<Vec<BsiMappingEntry>, _> = serde_json::from_slice(truncated);
        assert!(
            res.is_err(),
            "Truncated BSI JSON must return Err, never panic"
        );
    }
}
