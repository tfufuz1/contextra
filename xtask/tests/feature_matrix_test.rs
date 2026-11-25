//! Tests for feature_matrix module.

#[path = "../src/feature_matrix.rs"]
mod feature_matrix;

use feature_matrix::{scan_crate_features, FeatureDefault};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_feature_matrix_scan_crate_features() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();

    let crates_dir = root.join("crates");

    // Crate A
    let crate_a_dir = crates_dir.join("crate-a");
    fs::create_dir_all(&crate_a_dir).unwrap();

    let crate_a_cargo = r#"
[package]
name = "crate-a"
version = "0.1.0"

[features]
default = ["feat1"]
feat1 = []
feat2 = []
dev-sim = []
"#;
    fs::write(crate_a_dir.join("Cargo.toml"), crate_a_cargo).unwrap();

    let entries = scan_crate_features(&crates_dir).unwrap();

    assert_eq!(entries.len(), 3);

    let feat1 = entries.iter().find(|e| e.feature == "feat1").unwrap();
    assert_eq!(feat1.default, FeatureDefault::On);

    let feat2 = entries.iter().find(|e| e.feature == "feat2").unwrap();
    assert_eq!(feat2.default, FeatureDefault::Off);

    let dev_feat = entries.iter().find(|e| e.feature == "dev-sim").unwrap();
    assert_eq!(dev_feat.default, FeatureDefault::Dev);
}
