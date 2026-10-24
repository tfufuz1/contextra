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
        "# ADR 001\n\n* **Status**: Accepted\nApproved change\n",
    )
    .unwrap();
    fs::write(
        path.join("docs/decisions/ADR-002-proposed.md"),
        "# ADR 002\n\n* **Status**: Proposed\nProposed change\n",
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
    // Scenario 6: Kein geschützter Pfad berührt: grün ohne Trailer
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
    // Scenario 2: ADR auf Basis, accepted: grün
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
fn test_protected_paths_fail_adr_created_in_same_pr() {
    // Scenario 1: ADR im selben PR angelegt: rot
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    fs::write(path.join("AGENTS.md"), "modified agents config\n").unwrap();
    fs::write(
        path.join("docs/decisions/ADR-099-new.md"),
        "# ADR 099\n\n* **Status**: Accepted\nBrand new ADR\n",
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
            "chore: modify agents with new adr in same pr\n\nProtected-Change: ADR-099",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    std::env::set_var("PR_LABELS", "protected-change");

    let root = path.to_str().unwrap();
    let code = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
        "--json".to_string(),
    ]);

    std::env::remove_var("PR_LABELS");

    assert_eq!(code, 1);
}

#[test]
fn test_protected_paths_fail_adr_proposed_status() {
    // Scenario 3: ADR auf Basis, Status proposed: rot
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
            "chore: modify agents using proposed adr\n\nProtected-Change: ADR-002",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    std::env::set_var("PR_LABELS", "protected-change");

    let root = path.to_str().unwrap();
    let code = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
        "--json".to_string(),
    ]);

    std::env::remove_var("PR_LABELS");

    assert_eq!(code, 1);
}

#[test]
fn test_protected_paths_fail_adr_modified_in_pr() {
    // Scenario 4: ADR auf Basis, aber im PR verändert: rot
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    fs::write(path.join("AGENTS.md"), "modified agents config\n").unwrap();
    fs::write(
        path.join("docs/decisions/ADR-001-test.md"),
        "# ADR 001\n\n* **Status**: Accepted\nApproved change modified in PR\n",
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
            "chore: modify agents and adr\n\nProtected-Change: ADR-001",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    std::env::set_var("PR_LABELS", "protected-change");

    let root = path.to_str().unwrap();
    let code = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
        "--json".to_string(),
    ]);

    std::env::remove_var("PR_LABELS");

    assert_eq!(code, 1);
}

#[test]
fn test_protected_paths_fail_non_existent_adr_trailer() {
    // Scenario 5: Trailer verweist auf nicht vorhandene ADR: rot
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
            "chore: modify agents invalid adr\n\nProtected-Change: ADR-999",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    std::env::set_var("PR_LABELS", "protected-change");

    let root = path.to_str().unwrap();
    let code = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
        "--json".to_string(),
    ]);

    std::env::remove_var("PR_LABELS");

    assert_eq!(code, 1);
}

#[test]
fn test_protected_paths_fail_shallow_repository() {
    // Scenario 7: Shallow Repository: Exit 2 mit Hinweis
    let (temp_dir, base_rev) = setup_temp_repo();
    let path = temp_dir.path();

    // Create .git/shallow file to simulate shallow repo
    fs::write(path.join(".git/shallow"), format!("{base_rev}\n")).unwrap();

    let root = path.to_str().unwrap();
    let code = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        root.to_string(),
        "--base".to_string(),
        base_rev,
        "--json".to_string(),
    ]);

    assert_eq!(code, 2);
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
