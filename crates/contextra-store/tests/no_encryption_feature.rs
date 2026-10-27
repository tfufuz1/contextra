use std::fs;
use std::path::Path;

#[test]
fn test_no_encryption_or_legacy_features() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = fs::read_to_string(&manifest_path).expect("failed to read Cargo.toml");

    let doc: toml::Value = toml::from_str(&content).expect("failed to parse Cargo.toml");
    if let Some(features) = doc.get("features").and_then(|f| f.as_table()) {
        assert!(
            !features.contains_key("encryption-at-rest"),
            "Feature 'encryption-at-rest' must be resolved and removed from Cargo.toml"
        );
        assert!(
            !features.contains_key("deletion-proof"),
            "Feature 'deletion-proof' must be resolved and removed from Cargo.toml"
        );
        assert!(
            !features.contains_key("wal-integrity"),
            "Feature 'wal-integrity' must be resolved and removed from Cargo.toml"
        );
    }
}
