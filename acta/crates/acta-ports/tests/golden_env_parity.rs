#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Golden data byte parity test for acta-env (FND.1)

use std::fs;

#[test]
fn test_golden_env_manifest_checksums() {
    let golden_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../acta-env/tests/golden");
    let manifest_path = format!("{}/MANIFEST.sha256", golden_dir);
    let manifest_content = fs::read_to_string(&manifest_path)
        .expect("MANIFEST.sha256 must exist for acta-env golden tests");

    for line in manifest_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let expected_hash = parts.next().expect("hash part");
        let filename = parts.next().expect("filename part");
        let file_path = format!("{}/{}", golden_dir, filename);

        let output = std::process::Command::new("sha256sum")
            .arg(&file_path)
            .output()
            .expect("sha256sum execution");
        let sum_output = String::from_utf8_lossy(&output.stdout);
        let actual_sha256 = sum_output.split_whitespace().next().expect("sha256 hex");
        assert_eq!(
            actual_sha256, expected_hash,
            "Checksum mismatch for {}",
            filename
        );
    }
}
