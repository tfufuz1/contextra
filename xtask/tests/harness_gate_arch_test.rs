#[path = "../src/harness/gate_arch.rs"]
mod gate_arch;

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn create_temp_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let p = temp_dir.path();

    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(p)
            .args(args)
            .status()
            .expect("failed to execute git command");
        assert!(status.success(), "git command failed: {:?}", args);
    };

    run_git(&["init", "-b", "main"]);
    run_git(&["config", "user.name", "Tester"]);
    run_git(&["config", "user.email", "tester@example.com"]);

    fs::create_dir_all(p.join("crates/contextra-core/src")).expect("failed to create core dir");
    fs::create_dir_all(p.join("crates/contextra-sys/src")).expect("failed to create sys dir");

    fs::write(
        p.join("Cargo.toml"),
        r#"
[workspace]
members = ["crates/contextra-core", "crates/contextra-sys"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
"#,
    )
    .expect("failed to write Cargo.toml");

    fs::write(
        p.join("crates/contextra-core/Cargo.toml"),
        r#"
[package]
name = "contextra-core"
version.workspace = true
edition.workspace = true

[package.metadata.contextra]
ring = "0"
"#,
    )
    .expect("failed to write core Cargo.toml");

    fs::write(
        p.join("crates/contextra-sys/Cargo.toml"),
        r#"
[package]
name = "contextra-sys"
version.workspace = true
edition.workspace = true

[package.metadata.contextra]
ring = "0"
"#,
    )
    .expect("failed to write sys Cargo.toml");

    fs::write(
        p.join("capabilities.toml"),
        r#"
[crates.contextra-core]
ring = "Ring 0"
path = "crates/contextra-core"

[crates.contextra-sys]
ring = "Ring 0"
path = "crates/contextra-sys"
unsafe_island = true
"#,
    )
    .expect("failed to write capabilities.toml");

    fs::write(
        p.join("crates/contextra-core/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub fn clean() {}\n",
    )
    .expect("failed to write core lib.rs");

    fs::write(
        p.join("crates/contextra-sys/src/lib.rs"),
        "pub unsafe fn island_unsafe() {}\n",
    )
    .expect("failed to write sys lib.rs");

    run_git(&["add", "."]);
    run_git(&["commit", "-m", "Initial commit"]);

    temp_dir
}

fn commit_changes(root: &Path, msg: &str) {
    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(root)
            .args(args)
            .status()
            .expect("failed to execute git command");
        assert!(status.success(), "git command failed: {:?}", args);
    };

    run_git(&["add", "."]);
    run_git(&["commit", "-m", msg]);
}

#[test]
fn test_gate_arch_clean_repo_passes() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub fn clean() { println!(\"ok\"); }\n",
    )
    .expect("failed to write core lib.rs");
    commit_changes(root, "feat: clean architecture change");

    let root_str = root.to_string_lossy().to_string();
    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
    ];

    let code = gate_arch::run_gate_arch(&args);
    assert_eq!(code, 0, "gate-arch MUST pass on clean architecture");
}

#[test]
fn test_gate_arch_canary_unsafe_islands_defect() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    // Defect: unsafe code in non-island crate
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub unsafe fn bad_unsafe() {}\n",
    )
    .expect("failed to write unsafe code");
    commit_changes(root, "feat: bad unsafe in non-island");

    let root_str = root.to_string_lossy().to_string();
    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--strict".to_string(),
    ];

    let code = gate_arch::run_gate_arch(&args);
    assert_ne!(code, 0, "gate-arch MUST fail on unsafe-islands defect");
}

#[test]
fn test_gate_arch_canary_capabilities_consistency_defect() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    // Defect: add inconsistent capability entry
    fs::write(
        root.join("capabilities.toml"),
        r#"
[crates.contextra-core]
ring = "Ring 0"
path = "crates/contextra-core"

[crates.nonexistent-crate]
ring = "Ring 0"
path = "crates/nonexistent-crate"
"#,
    )
    .expect("failed to write capabilities.toml");
    commit_changes(root, "feat: invalid capabilities.toml entry");

    let root_str = root.to_string_lossy().to_string();
    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
    ];

    let code = gate_arch::run_gate_arch(&args);
    assert_ne!(
        code, 0,
        "gate-arch MUST fail on ring-capabilities-consistency defect"
    );
}
