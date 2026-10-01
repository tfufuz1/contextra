//! Integration test for gate_weakening harness module.

#[path = "../src/harness/gate_weakening.rs"]
mod gate_weakening;

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

    fs::create_dir_all(path.join(".github/workflows")).unwrap();
    fs::create_dir_all(path.join("docs/decisions")).unwrap();
    fs::create_dir_all(path.join("src")).unwrap();

    fs::write(
        path.join(".github/workflows/ci.yml"),
        "name: CI\nsteps:\n  - run: cargo clippy -- -D warnings\n",
    )
    .unwrap();

    fs::write(
        path.join(".github/unwrap_baseline.txt"),
        "100\n",
    )
    .unwrap();

    fs::write(
        path.join("src/lib.rs"),
        "#![forbid(unsafe_code)]\n#[test]\nfn test_foo() { assert!(true); }\n",
    )
    .unwrap();

    fs::write(
        path.join("docs/decisions/ADR-002-test.md"),
        "# ADR 002\nGate weakening allowed\n",
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
fn test_gate_weakening_pass_clean_diff() {
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    fs::write(
        path.join("src/lib.rs"),
        "#![forbid(unsafe_code)]\n#[test]\nfn test_foo() { assert!(true); }\n#[test]\nfn test_bar() { assert_eq!(1, 1); }\n",
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
        .args(["commit", "-m", "test: add new test"])
        .status()
        .unwrap();
    assert!(status.success());

    let root = path.to_str().unwrap();
    let code = gate_weakening::run_gate_weakening(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
        "--json".to_string(),
    ]);
    assert_eq!(code, 0);
}

#[test]
fn test_gate_weakening_fail_on_continue_on_error() {
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    fs::write(
        path.join(".github/workflows/ci.yml"),
        "name: CI\nsteps:\n  - run: cargo clippy -- -D warnings\n    continue-on-error: true\n",
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
        .args(["commit", "-m", "ci: allow continue on error"])
        .status()
        .unwrap();
    assert!(status.success());

    let root = path.to_str().unwrap();
    let code = gate_weakening::run_gate_weakening(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
    ]);
    assert_eq!(code, 1);
}

#[test]
fn test_gate_weakening_fail_on_allow_attr() {
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    fs::write(
        path.join("src/lib.rs"),
        "#![forbid(unsafe_code)]\n#[allow(dead_code)]\n#[test]\nfn test_foo() { assert!(true); }\n",
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
        .args(["commit", "-m", "style: add allow dead_code"])
        .status()
        .unwrap();
    assert!(status.success());

    let root = path.to_str().unwrap();
    let code = gate_weakening::run_gate_weakening(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
    ]);
    assert_eq!(code, 1);
}

#[test]
fn test_gate_weakening_fail_on_baseline_increase() {
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    fs::write(path.join(".github/unwrap_baseline.txt"), "150\n").unwrap();

    let status = Command::new("git")
        .current_dir(path)
        .args(["add", "."])
        .status()
        .unwrap();
    assert!(status.success());
    let status = Command::new("git")
        .current_dir(path)
        .args(["commit", "-m", "chore: increase unwrap baseline"])
        .status()
        .unwrap();
    assert!(status.success());

    let root = path.to_str().unwrap();
    let code = gate_weakening::run_gate_weakening(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
    ]);
    assert_eq!(code, 1);
}

#[test]
fn test_gate_weakening_pass_with_valid_adr_trailer() {
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    fs::write(
        path.join(".github/workflows/ci.yml"),
        "name: CI\nsteps:\n  - run: cargo clippy -- -D warnings\n    continue-on-error: true\n",
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
        .args([
            "commit",
            "-m",
            "ci: allow continue on error\n\nGate-Weakening: ADR-002",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let root = path.to_str().unwrap();
    let code = gate_weakening::run_gate_weakening(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
        "--json".to_string(),
    ]);
    assert_eq!(code, 0);
}
