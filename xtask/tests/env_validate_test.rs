use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

pub fn find_root_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CONTEXTRA_ROOT_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut dir = PathBuf::from(manifest_dir);
        loop {
            let cargo_path = dir.join("Cargo.toml");
            if cargo_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                    if content.contains("[workspace]") && content.contains("members") {
                        return dir;
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
    }
    PathBuf::from(".")
}

#[path = "../src/env_validate.rs"]
mod env_validate;

use env_validate::{run_env_validate, ToolStatus};

#[test]
fn test_run_env_validate_returns_checked_tools() {
    let checks = run_env_validate();
    assert_eq!(checks.len(), 9);

    let rustc_check = checks.iter().find(|c| c.name == "rustc").unwrap();
    assert!(rustc_check.required);
    assert_eq!(rustc_check.command, "rustc");
    assert!(matches!(rustc_check.status, ToolStatus::Ok(_)));

    let clippy_check = checks.iter().find(|c| c.name == "cargo clippy").unwrap();
    assert!(clippy_check.required);

    let rustfmt_check = checks.iter().find(|c| c.name == "rustfmt").unwrap();
    assert!(rustfmt_check.required);
}

#[test]
fn test_read_required_rustc_version_from_fixture() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    std::env::set_var("CONTEXTRA_ROOT_DIR", root);

    let toolchain_toml = r#"[toolchain]
channel = "1.89.0"
components = ["rustfmt", "clippy"]
"#;
    fs::write(root.join("rust-toolchain.toml"), toolchain_toml).unwrap();

    let checks = run_env_validate();
    let rustc_check = checks.iter().find(|c| c.name == "rustc").unwrap();
    assert!(matches!(rustc_check.status, ToolStatus::Ok(_)));

    std::env::remove_var("CONTEXTRA_ROOT_DIR");
}
