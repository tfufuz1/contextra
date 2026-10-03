#[path = "../src/agent_lifecycle/branch_overlap.rs"]
mod branch_overlap;

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn setup_temp_repo_with_remote() -> (TempDir, TempDir) {
    let remote_dir = TempDir::new().expect("failed to create remote temp dir");
    let local_dir = TempDir::new().expect("failed to create local temp dir");

    let remote_path = remote_dir.path();
    let local_path = local_dir.path();

    // Init bare remote repository
    let status = Command::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_path)
        .status()
        .expect("git init bare failed");
    assert!(status.success());

    // Init local repository and add origin remote
    let status = Command::new("git")
        .args(["init"])
        .current_dir(local_path)
        .status()
        .expect("git init failed");
    assert!(status.success());

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(local_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(local_path)
        .output()
        .unwrap();

    let remote_url = format!("{}", remote_path.display());
    Command::new("git")
        .args(["remote", "add", "origin", &remote_url])
        .current_dir(local_path)
        .output()
        .unwrap();

    // Initial commit on main
    fs::write(local_path.join("README.md"), "# Test Repo\n").unwrap();
    fs::create_dir_all(local_path.join("crates/crate1/src")).unwrap();
    fs::create_dir_all(local_path.join("crates/crate2/src")).unwrap();
    fs::write(local_path.join("crates/crate1/src/lib.rs"), "// crate1\n").unwrap();
    fs::write(local_path.join("crates/crate2/src/lib.rs"), "// crate2\n").unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(local_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(local_path)
        .output()
        .unwrap();

    // Branch main -> push to origin
    Command::new("git")
        .args(["branch", "-M", "main"])
        .current_dir(local_path)
        .output()
        .unwrap();

    let status = Command::new("git")
        .args(["push", "origin", "main"])
        .current_dir(local_path)
        .status()
        .expect("git push failed");
    assert!(status.success());

    (remote_dir, local_dir)
}

#[test]
fn test_branch_overlap_overlapping_scopes_returns_red() {
    let (_remote_dir, local_dir) = setup_temp_repo_with_remote();
    let local_path = local_dir.path();

    // Create a remote branch claim/T-2026-0001 modifying crates/crate1/src/lib.rs
    Command::new("git")
        .args(["checkout", "-b", "claim/T-2026-0001"])
        .current_dir(local_path)
        .output()
        .unwrap();
    fs::write(
        local_path.join("crates/crate1/src/lib.rs"),
        "// crate1 edited in claim/T-2026-0001\n",
    )
    .unwrap();
    Command::new("git")
        .args(["commit", "-am", "change in crate1"])
        .current_dir(local_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["push", "origin", "claim/T-2026-0001"])
        .current_dir(local_path)
        .output()
        .unwrap();

    // Switch back to main for local work
    Command::new("git")
        .args(["checkout", "main"])
        .current_dir(local_path)
        .output()
        .unwrap();

    // Create active task card in local repo with scope including crate1
    fs::create_dir_all(local_path.join(".jules/tasks")).unwrap();
    fs::write(
        local_path.join(".jules/tasks/T-2026-0002.toml"),
        r#"
id = "T-2026-0002"
title = "Task 2"
scope = ["crates/crate1/**"]
forbidden = []
"#,
    )
    .unwrap();

    let root_arg = format!("{}", local_path.display());
    let card_arg = format!("{}/.jules/tasks/T-2026-0002.toml", local_path.display());

    let args = vec![
        "--root".to_string(),
        root_arg,
        "--card".to_string(),
        card_arg,
    ];

    let success = branch_overlap::run_check_branch_overlap_with_args(&args);
    assert!(
        !success,
        "Expected overlap check to fail (return false / red) due to scope overlap"
    );
}

#[test]
fn test_branch_overlap_disjoint_scopes_returns_green() {
    let (_remote_dir, local_dir) = setup_temp_repo_with_remote();
    let local_path = local_dir.path();

    // Create remote branch claim/T-2026-0001 modifying crates/crate1/src/lib.rs
    Command::new("git")
        .args(["checkout", "-b", "claim/T-2026-0001"])
        .current_dir(local_path)
        .output()
        .unwrap();
    fs::write(
        local_path.join("crates/crate1/src/lib.rs"),
        "// crate1 edited in claim/T-2026-0001\n",
    )
    .unwrap();
    Command::new("git")
        .args(["commit", "-am", "change in crate1"])
        .current_dir(local_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["push", "origin", "claim/T-2026-0001"])
        .current_dir(local_path)
        .output()
        .unwrap();

    // Switch back to main
    Command::new("git")
        .args(["checkout", "main"])
        .current_dir(local_path)
        .output()
        .unwrap();

    // Create active task card for crate2 (disjoint scope)
    fs::create_dir_all(local_path.join(".jules/tasks")).unwrap();
    fs::write(
        local_path.join(".jules/tasks/T-2026-0003.toml"),
        r#"
id = "T-2026-0003"
title = "Task 3"
scope = ["crates/crate2/**"]
forbidden = []
"#,
    )
    .unwrap();

    let root_arg = format!("{}", local_path.display());
    let card_arg = format!("{}/.jules/tasks/T-2026-0003.toml", local_path.display());

    let args = vec![
        "--root".to_string(),
        root_arg,
        "--card".to_string(),
        card_arg,
    ];

    let success = branch_overlap::run_check_branch_overlap_with_args(&args);
    assert!(
        success,
        "Expected overlap check to succeed (return true / green) for disjoint scopes"
    );
}

#[test]
fn test_branch_overlap_ls_remote_error_returns_red() {
    let (_remote_dir, local_dir) = setup_temp_repo_with_remote();
    let local_path = local_dir.path();

    let root_arg = format!("{}", local_path.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "--remote".to_string(),
        "invalid_remote_name_999".to_string(),
    ];

    let success = branch_overlap::run_check_branch_overlap_with_args(&args);
    assert!(
        !success,
        "Expected ls-remote error to fail-closed (return false / red)"
    );
}

#[test]
fn test_branch_overlap_no_active_card_returns_green() {
    let (_remote_dir, local_dir) = setup_temp_repo_with_remote();
    let local_path = local_dir.path();

    let root_arg = format!("{}", local_path.display());
    let args = vec!["--root".to_string(), root_arg];

    let success = branch_overlap::run_check_branch_overlap_with_args(&args);
    assert!(
        success,
        "Expected check with no active card to return true (green) with clear message"
    );
}
