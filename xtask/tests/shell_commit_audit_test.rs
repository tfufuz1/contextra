//! Integration test for `shell_commit_audit`

#[path = "../src/shell_commit_audit.rs"]
mod shell_commit_audit;

use shell_commit_audit::run_shell_commit_audit_impl;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn run_git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("Failed to execute git command in test repo");

    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn create_test_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("Failed to create temp dir");
    let p = dir.path();

    run_git(p, &["init"]);
    run_git(p, &["config", "user.name", "Test Agent"]);
    run_git(p, &["config", "user.email", "agent@example.com"]);

    dir
}

#[test]
fn test_shell_commit_audit_detection() {
    let repo = create_test_repo();
    let p = repo.path();

    // Commit 1: Normal commit
    fs::write(p.join("file1.txt"), "hello").unwrap();
    run_git(p, &["add", "."]);
    run_git(p, &["commit", "-m", "feat: initial commit"]);

    // Commit 2: Shell commit with small diff (1 line change)
    fs::write(p.join("file1.txt"), "hello world").unwrap();
    run_git(p, &["add", "."]);
    run_git(p, &["commit", "-m", "Shell-Commit"]);

    // Commit 3: Shell commit with substantial diff (>50 lines)
    let lines: Vec<String> = (0..60).map(|i| format!("line {}", i)).collect();
    fs::write(p.join("large_file.txt"), lines.join("\n")).unwrap();
    run_git(p, &["add", "."]);
    run_git(p, &["commit", "-m", "Shell-Commit"]);

    let report =
        run_shell_commit_audit_impl(p, None, None).expect("Audit should run without error");

    assert_eq!(report.shell_commits.len(), 2);

    let substantial_count = report
        .shell_commits
        .iter()
        .filter(|c| c.has_substantial_diff)
        .count();
    assert_eq!(substantial_count, 1);

    // Test fail_on_count threshold
    let fail_res = run_shell_commit_audit_impl(p, None, Some(0));
    assert!(fail_res.is_err());
    assert!(fail_res.unwrap_err().contains("exceeding threshold of 0"));

    let pass_res = run_shell_commit_audit_impl(p, None, Some(1));
    assert!(pass_res.is_ok());
}
