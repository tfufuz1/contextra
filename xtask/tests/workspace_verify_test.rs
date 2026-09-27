//! Tests for workspace_verify module.

#[path = "../src/feature_matrix.rs"]
mod feature_matrix;
#[path = "../src/panic_inventory.rs"]
mod panic_inventory;
#[path = "../src/security_scan.rs"]
mod security_scan;
#[path = "../src/workspace_verify.rs"]
mod workspace_verify;

use std::fs;
use tempfile::tempdir;
use workspace_verify::{
    get_bottom_up_crate_order_from_metadata, run_workspace_verify_in_root, VerifyMode,
    WorkspaceVerifyConfig,
};

#[test]
fn test_bottom_up_crate_order_resolution() {
    let mock_json = r#"{
        "packages": [
            {
                "name": "crate-c",
                "dependencies": [
                    { "name": "crate-b" }
                ]
            },
            {
                "name": "crate-b",
                "dependencies": [
                    { "name": "crate-a" }
                ]
            },
            {
                "name": "crate-a",
                "dependencies": []
            }
        ],
        "workspace_members": ["crate-a", "crate-b", "crate-c"]
    }"#;

    let order = get_bottom_up_crate_order_from_metadata(mock_json).unwrap();
    assert_eq!(order, vec!["crate-a", "crate-b", "crate-c"]);
}

#[test]
fn test_workspace_verify_synthetic_fixture() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();
    let output_dir = root.join("target").join("workspace-verify");

    let crates_dir = root.join("crates");

    // Create a minimal synthetic workspace
    let crate_a_dir = crates_dir.join("mini-crate-a");
    fs::create_dir_all(&crate_a_dir.join("src")).unwrap();

    let root_cargo = format!("[workspace]\nmembers = [\"crates/mini-crate-a\"]\n");
    fs::write(root.join("Cargo.toml"), root_cargo).unwrap();
    fs::write(root.join("capabilities.toml"), "").unwrap();

    let crate_a_cargo = r#"
[package]
name = "mini-crate-a"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    fs::write(crate_a_dir.join("Cargo.toml"), crate_a_cargo).unwrap();
    fs::write(
        crate_a_dir.join("src").join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
    )
    .unwrap();

    let config = WorkspaceVerifyConfig {
        mode: VerifyMode::Fast,
        only: vec!["mini-crate-a".to_string()],
        resume_from: None,
        stop_on_fail: true,
        run_clippy: false,
    };

    let res = run_workspace_verify_in_root(root, config, &output_dir);
    assert!(res.is_ok(), "Expected workspace_verify to pass: {:?}", res);

    assert!(output_dir.join("summary.md").exists());
    assert!(output_dir.join("diagnostics.jsonl").exists());
    assert!(output_dir.join("FEHLERBERICHT.md").exists());
}
