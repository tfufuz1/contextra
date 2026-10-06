#[path = "../src/harness/gate_integrity.rs"]
mod gate_integrity;

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

    fs::create_dir_all(p.join("governance")).expect("failed to create governance dir");
    fs::create_dir_all(p.join(".github")).expect("failed to create .github dir");
    fs::create_dir_all(p.join("crates/contextra-core/src")).expect("failed to create core dir");
    fs::create_dir_all(p.join("tests")).expect("failed to create tests dir");

    fs::write(
        p.join("Cargo.toml"),
        r#"
[workspace]
members = ["crates/contextra-core"]
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
        p.join("capabilities.toml"),
        r#"
[crates.contextra-core]
ring = "Ring 0"
path = "crates/contextra-core"
"#,
    )
    .expect("failed to write capabilities.toml");

    let ratchet_content = format!(
        "{}_legacy_grep = 0\n{}_exact = 0\nexpect_exact = 0\nallow_attrs = 0\nignore_attrs = 0\nforbid_unsafe_crates = 1\n[unsafe_blocks]\n",
        "unwrap", "unwrap"
    );

    fs::write(p.join("governance/ratchet.toml"), ratchet_content)
        .expect("failed to write ratchet.toml");

    fs::write(p.join(".github/unwrap_baseline.txt"), "0\n")
        .expect("failed to write unwrap baseline");

    fs::write(
        p.join("crates/contextra-core/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub fn clean() {}\n",
    )
    .expect("failed to write lib.rs");

    fs::write(
        p.join("tests/dummy_test.rs"),
        "#[test]\nfn test_valid() {\n    assert_eq!(1, 1);\n}\n",
    )
    .expect("failed to write dummy_test.rs");

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
fn test_gate_integrity_clean_repo_passes() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::write(
        root.join("tests/dummy_test.rs"),
        "#[test]\nfn test_valid() {\n    assert_eq!(1, 1);\n    assert_eq!(2, 2);\n}\n",
    )
    .expect("failed to update test");
    commit_changes(root, "feat: update valid test");

    let root_str = root.to_string_lossy().to_string();
    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
    ];

    let code = gate_integrity::run_gate_integrity(&args);
    assert_eq!(code, 0, "gate-integrity MUST pass on clean repository");
}

#[test]
fn test_gate_integrity_canary_test_integrity_defect() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    // Defect: delete test file without replacement
    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(root)
            .args(args)
            .status()
            .expect("failed to execute git command");
        assert!(status.success(), "git command failed: {:?}", args);
    };
    run_git(&["rm", "tests/dummy_test.rs"]);
    commit_changes(root, "chore: delete test file");

    let root_str = root.to_string_lossy().to_string();
    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
    ];

    let code = gate_integrity::run_gate_integrity(&args);
    assert_ne!(code, 0, "gate-integrity MUST fail on test-integrity defect");
}

#[test]
fn test_gate_integrity_canary_ratchet_defect() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    // Defect: add #[allow(unused)] exceeding ratchet baseline (0)
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "#![forbid(unsafe_code)]\n#[allow(unused)]\npub fn degraded() {}\n",
    )
    .expect("failed to write degraded code");
    commit_changes(root, "chore: introduce allow attribute exceeding ratchet");

    let root_str = root.to_string_lossy().to_string();
    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
    ];

    let code = gate_integrity::run_gate_integrity(&args);
    assert_ne!(code, 0, "gate-integrity MUST fail on ratchet defect");
}

#[test]
fn test_gate_integrity_canary_determinism_defect() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    // Defect: introduce SystemTime::now() in Ring 0 code
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "pub fn bad_clock() {\n    let _ = std::time::SystemTime::now();\n}\n",
    )
    .expect("failed to write bad clock code");
    commit_changes(root, "feat: non-deterministic clock in Ring 0");

    let root_str = root.to_string_lossy().to_string();
    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
    ];

    let code = gate_integrity::run_gate_integrity(&args);
    assert_ne!(code, 0, "gate-integrity MUST fail on determinism defect");
}
