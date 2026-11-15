#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Golden data byte parity test for acta-ids (FND.1)

use std::fs;

#[test]
fn test_golden_ids_manifest_checksums() {
    let golden_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../acta-ids/tests/golden");
    let manifest_path = format!("{}/MANIFEST.sha256", golden_dir);
    let manifest_content = fs::read_to_string(&manifest_path)
        .expect("MANIFEST.sha256 must exist for acta-ids golden tests");

    for line in manifest_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let expected_hash = parts.next().expect("hash part");
        let filename = parts.next().expect("filename part");
        let file_path = format!("{}/{}", golden_dir, filename);
        let bytes =
            fs::read(&file_path).unwrap_or_else(|e| panic!("Failed to read {}: {}", file_path, e));
        let actual_hash = format!("{:x}", <sha2::Sha256 as sha2::Digest>::digest(&bytes));
        assert_eq!(
            actual_hash, expected_hash,
            "Checksum mismatch for {}",
            filename
        );
    }
}
