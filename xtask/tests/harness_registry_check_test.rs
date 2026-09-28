//! Integration tests for registry-check harness command.

#[path = "../src/harness/registry_check.rs"]
mod registry_check;

use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn setup_temp_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    let output = Command::new("git")
        .args(&["init"])
        .current_dir(root)
        .output()
        .expect("git init");
    assert!(output.status.success());

    let xtask_src = root.join("xtask/src");
    let harness_dir = xtask_src.join("harness");
    fs::create_dir_all(&harness_dir).expect("create_dir_all");

    fs::write(
        xtask_src.join("main.rs"),
        r#"
fn main() {
    match subcommand {
        "sync-docs" => {}
        "check-compile" => {}
        _ => {}
    }
}
"#,
    )
    .expect("write main.rs");

    fs::write(
        root.join("justfile"),
        r#"
check-compile:
    cargo xtask check-compile
"#,
    )
    .expect("write justfile");

    fs::write(
        root.join("xtask/registry.toml"),
        r#"
[[commands]]
name = "sync-docs"
owner_tier = "legacy"
summary = "Sync docs"
blocking = true
phase = "Phase 0"

[[commands]]
name = "check-compile"
owner_tier = "legacy"
summary = "Check compile"
blocking = true
phase = "Phase 0"
"#,
    )
    .expect("write registry.toml");

    dir
}

#[test]
fn test_registry_check_consistent() {
    let dir = setup_temp_repo();
    let root_str = dir.path().to_string_lossy().to_string();

    let args = vec!["--root".to_string(), root_str];
    let exit_code = registry_check::run_registry_check(&args);
    assert_eq!(
        exit_code, 0,
        "registry-check should pass on consistent repo"
    );
}

#[test]
fn test_registry_check_missing_entry() {
    let dir = setup_temp_repo();
    let root = dir.path();

    fs::write(
        root.join("xtask/registry.toml"),
        r#"
[[commands]]
name = "sync-docs"
owner_tier = "legacy"
summary = "Sync docs"
blocking = true
phase = "Phase 0"
"#,
    )
    .expect("write incomplete registry.toml");

    let root_str = root.to_string_lossy().to_string();
    let args = vec!["--root".to_string(), root_str];
    let exit_code = registry_check::run_registry_check(&args);
    assert_eq!(exit_code, 1, "registry-check should fail on missing entry");
}

#[test]
fn test_registry_check_orphaned_entry() {
    let dir = setup_temp_repo();
    let root = dir.path();

    fs::write(
        root.join("xtask/registry.toml"),
        r#"
[[commands]]
name = "sync-docs"
owner_tier = "legacy"
summary = "Sync docs"
blocking = true
phase = "Phase 0"

[[commands]]
name = "check-compile"
owner_tier = "legacy"
summary = "Check compile"
blocking = true
phase = "Phase 0"

[[commands]]
name = "orphaned-cmd"
owner_tier = "legacy"
summary = "Orphaned"
blocking = true
phase = "Phase 0"
"#,
    )
    .expect("write registry.toml with orphan");

    let root_str = root.to_string_lossy().to_string();
    let args = vec!["--root".to_string(), root_str];
    let exit_code = registry_check::run_registry_check(&args);
    assert_eq!(exit_code, 1, "registry-check should fail on orphaned entry");
}

#[test]
fn test_registry_check_harness_collision() {
    let dir = setup_temp_repo();
    let root = dir.path();

    let harness_file = root.join("xtask/src/harness/sync_docs.rs");
    fs::write(
        harness_file,
        r#"//! Sync docs harness
pub fn run_sync_docs(_args: &[String]) -> i32 { 0 }
"#,
    )
    .expect("write harness collision file");

    let root_str = root.to_string_lossy().to_string();
    let args = vec!["--root".to_string(), root_str];
    let exit_code = registry_check::run_registry_check(&args);
    assert_eq!(
        exit_code, 1,
        "registry-check should fail on harness/main collision"
    );
}
