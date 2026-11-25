//! Integration tests for gates-catalog-sync harness command.

#[path = "../src/harness/gates_catalog_sync.rs"]
mod gates_catalog_sync;

use std::fs;
use tempfile::tempdir;

#[test]
fn test_gates_catalog_sync_mismatch() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    fs::create_dir_all(root.join("governance")).expect("governance dir");
    fs::create_dir_all(root.join(".jules")).expect("jules dir");

    fs::write(
        root.join("governance/gates.toml"),
        r#"
[[gate]]
name = "protected-paths"
phase = "check"
required_local = false
required_ci = true
risk_class = "high"
owner = "security"
fixture = "none"
"#,
    )
    .expect("write gates.toml");

    fs::write(
        root.join(".jules/harness-phases.toml"),
        r#"
[[phase.check]]
cmd = ["cargo", "xtask", "protected-paths"]
required = true
"#,
    )
    .expect("write harness-phases.toml");

    fs::write(
        root.join("governance/verdict-required.toml"),
        r#"
[[gate]]
name = "protected-paths"
blocking = true
"#,
    )
    .expect("write verdict-required.toml");

    let root_str = root.to_string_lossy().to_string();
    let args = vec!["--root".to_string(), root_str];
    let exit_code = gates_catalog_sync::run_gates_catalog_sync(&args);
    assert_eq!(exit_code, 1, "gates-catalog-sync should fail on mismatch");
}

#[test]
fn test_gates_catalog_sync_consistent() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    fs::create_dir_all(root.join("governance")).expect("governance dir");
    fs::create_dir_all(root.join(".jules")).expect("jules dir");

    fs::write(
        root.join("governance/gates.toml"),
        r#"
[[gate]]
name = "protected-paths"
phase = "check"
required_local = true
required_ci = true
risk_class = "high"
owner = "security"
fixture = "none"
"#,
    )
    .expect("write gates.toml");

    fs::write(
        root.join(".jules/harness-phases.toml"),
        r#"
[[phase.check]]
cmd = ["cargo", "xtask", "protected-paths"]
required = true
"#,
    )
    .expect("write harness-phases.toml");

    fs::write(
        root.join("governance/verdict-required.toml"),
        r#"
[[gate]]
name = "protected-paths"
blocking = true
"#,
    )
    .expect("write verdict-required.toml");

    let root_str = root.to_string_lossy().to_string();
    let args = vec!["--root".to_string(), root_str];
    let exit_code = gates_catalog_sync::run_gates_catalog_sync(&args);
    assert_eq!(
        exit_code, 0,
        "gates-catalog-sync should pass on consistent config"
    );
}
