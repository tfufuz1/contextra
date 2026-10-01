//! Integration test for protected_paths harness module.

#[path = "../src/harness/protected_paths.rs"]
mod protected_paths;

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn setup_temp_repo() -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let path = temp_dir.path();

    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(path)
            .args(args)
            .status()
            .expect("failed to execute git command");
        assert!(status.success());
    };

    run_git(&["init"]);
    run_git(&["config", "user.name", "Test User"]);
    run_git(&["config", "user.email", "test@example.com"]);

    // Create governance and docs directory structure
    fs::create_dir_all(path.join("governance")).unwrap();
    fs::create_dir_all(path.join("docs/decisions")).unwrap();
    fs::create_dir_all(path.join("xtask/src/harness")).unwrap();
    fs::create_dir_all(path.join("src")).unwrap();

    fs::write(
        path.join("governance/protected-paths.toml"),
        r#"
[[protected]]
glob = "governance/**"
reason = "Governance configuration"

[[protected]]
glob = "xtask/src/harness/protected_paths.rs"
reason = "Self protection"

[[protected]]
glob = "AGENTS.md"
reason = "Agent guidance"
"#,
    )
    .unwrap();

    fs::write(path.join("AGENTS.md"), "agents config\n").unwrap();
    fs::write(path.join("src/lib.rs"), "pub fn hello() {}\n").unwrap();
    fs::write(
        path.join("docs/decisions/ADR-001-test.md"),
        "# ADR 001\nApproved change\n",
    )
    .unwrap();

    run_git(&["add", "."]);
    run_git(&["commit", "-m", "Initial commit"]);

    let output = Command::new("git")
        .current_dir(path)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let base_rev = String::from_utf8(output.stdout).unwrap().trim().to_string();

    (temp_dir, base_rev)
}

#[test]
fn test_protected_paths_pass_no_protected_changes() {
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    fs::write(
        path.join("src/lib.rs"),
        "pub fn hello() { println!(\"hi\"); }\n",
    )
    .unwrap();
    let status = Command::new("git")
        .current_dir(path)
        .args(["add", "."])
        .status()
        .unwrap();
    assert!(status.success());
    let status = Command::new("git")
        .current_dir(path)
        .args(["commit", "-m", "feat: update non-protected file"])
        .status()
        .unwrap();
    assert!(status.success());

    let root = path.to_str().unwrap();
    let code = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
        "--json".to_string(),
    ]);
    assert_eq!(code, 0);
}

#[test]
fn test_protected_paths_fail_without_trailer_or_label() {
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    fs::write(path.join("AGENTS.md"), "modified agents config\n").unwrap();
    let status = Command::new("git")
        .current_dir(path)
        .args(["add", "."])
        .status()
        .unwrap();
    assert!(status.success());
    let status = Command::new("git")
        .current_dir(path)
        .args(["commit", "-m", "chore: modify agents file"])
        .status()
        .unwrap();
    assert!(status.success());

    let root = path.to_str().unwrap();
    let code = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
    ]);
    assert_eq!(code, 1);
}

#[test]
fn test_protected_paths_pass_with_valid_adr_trailer_and_label() {
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    fs::write(path.join("AGENTS.md"), "modified agents config\n").unwrap();
    let status = Command::new("git")
        .current_dir(path)
        .args(["add", "."])
        .status()
        .unwrap();
    assert!(status.success());
    let status = Command::new("git")
        .current_dir(path)
        .args([
            "commit",
            "-m",
            "chore: modify agents\n\nProtected-Change: ADR-001",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    std::env::set_var("PR_LABELS", "protected-change,enhancement");

    let root = path.to_str().unwrap();
    let code = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
        "--json".to_string(),
    ]);

    std::env::remove_var("PR_LABELS");

    assert_eq!(code, 0);
}

#[test]
fn test_protected_paths_error_on_invalid_rev() {
    let (temp_dir, _) = setup_temp_repo();
    let root = temp_dir.path().to_str().unwrap();

    let code = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        "invalid_revision_hash_1234".to_string(),
    ]);
    assert_eq!(code, 2);
}

#[test]
fn test_protected_paths_self_protection_check() {
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    fs::write(
        path.join("governance/protected-paths.toml"),
        r#"
[[protected]]
glob = "AGENTS.md"
reason = "Agent guidance"
"#,
    )
    .unwrap();

    let status = Command::new("git")
        .current_dir(path)
        .args(["add", "."])
        .status()
        .unwrap();
    assert!(status.success());
    let status = Command::new("git")
        .current_dir(path)
        .args(["commit", "-m", "chore: bad config remove self protection"])
        .status()
        .unwrap();
    assert!(status.success());

    let root = path.to_str().unwrap();
    let code = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
    ]);
    assert_eq!(code, 1);
}
