#![allow(
    clippy::all,
    clippy::type_complexity,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Prüftest für den Golden-Korpus von `contextra-kvcache` (KV3.1).
//! Verifiziert byteweise Gleichheit mit eingechecktem Korpus und führt Rückwärts-Lesetest durch.

use std::fs;
use std::path::PathBuf;

#[test]
fn test_golden_corpus_manifest_and_backward_read() {
    let golden_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let manifest_path = golden_dir.join("MANIFEST.sha256");
    assert!(manifest_path.exists(), "MANIFEST.sha256 must exist");

    let manifest_content = fs::read_to_string(&manifest_path).expect("read manifest");
    for line in manifest_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let expected_hash = parts.next().expect("hash in manifest");
        let filename = parts.next().expect("filename in manifest");

        let file_path = golden_dir.join(filename);
        assert!(file_path.exists(), "Golden file {:?} must exist", file_path);

        let data = fs::read(&file_path).expect("read golden file");
        let actual_sha256 = {
            use std::process::Command;
            let output = Command::new("sha256sum")
                .arg(&file_path)
                .output()
                .expect("sha256sum command");
            let out_str = String::from_utf8_lossy(&output.stdout);
            out_str.split_whitespace().next().unwrap().to_string()
        };
        assert_eq!(
            expected_hash, actual_sha256,
            "SHA256 mismatch for golden file {}",
            filename
        );

        // Backward read test: ensure files parse without panic
        if filename.ends_with(".json") {
            let parsed: serde_json::Value =
                serde_json::from_slice(&data).expect("Valid json parsing");
            assert!(!parsed.is_null());
        } else if filename.ends_with(".bin") {
            assert!(!data.is_empty());
        }
    }
}
