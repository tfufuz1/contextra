use std::fs;
use std::path::Path;

#[test]
fn test_no_encryption_or_legacy_features() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = fs::read_to_string(&manifest_path).expect("failed to read Cargo.toml");

    assert!(
        !content.contains("encryption-at-rest"),
        "Feature 'encryption-at-rest' must be resolved and removed from Cargo.toml"
    );
    assert!(
        !content.contains("deletion-proof"),
        "Feature 'deletion-proof' must be resolved and removed from Cargo.toml"
    );
    assert!(
        !content.contains("wal-integrity"),
        "Feature 'wal-integrity' must be resolved and removed from Cargo.toml"
    );
}
