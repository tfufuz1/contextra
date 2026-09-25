//! Integration Tests for `check-commit-diff-integrity`

#[path = "../src/check_commit_diff_integrity.rs"]
mod check_commit_diff_integrity;

use check_commit_diff_integrity::{
    check_single_commit, count_claim_points, parse_git_stat_output,
};
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
fn test_phantom_empty_commit_fails() {
    let repo = create_test_repo();
    let p = repo.path();

    let phantom_msg = r#"feat(core): major multi-feature enhancement

- Implementiert Feature A
- Fügt neue Datenstrukturen in Crate B hinzu
- Behebt Speicherleck in Storage Engine
- Refaktoriert Modul C
- Entfernt veraltete API-Methoden
"#;

    run_git(p, &["commit", "--allow-empty", "-m", phantom_msg]);

    let res = check_single_commit("HEAD", Some(p));
    assert!(res.is_err(), "Empty commit with 5 claims must fail");
    let err = res.unwrap_err();
    assert!(
        err.contains("PHANTOM COMMIT DETECTED"),
        "Error message must report PHANTOM COMMIT DETECTED: {}",
        err
    );
}

#[test]
fn test_valid_multi_file_commit_passes() {
    let repo = create_test_repo();
    let p = repo.path();

    fs::write(p.join("file1.rs"), "// file 1\n").unwrap();
    fs::write(p.join("file2.rs"), "// file 2\n").unwrap();
    fs::write(p.join("file3.rs"), "// file 3\n").unwrap();
    run_git(p, &["add", "."]);

    let valid_msg = r#"feat(multi): comprehensive multi-crate updates

- Implementiert Feature A
- Behebt Issue B
- Refaktoriert Crate C
"#;

    run_git(p, &["commit", "-m", valid_msg]);

    let res = check_single_commit("HEAD", Some(p));
    assert!(
        res.is_ok(),
        "Multi-file commit matching claims must pass: {:?}",
        res.err()
    );
}

#[test]
fn test_simple_honest_commit_passes() {
    let repo = create_test_repo();
    let p = repo.path();

    fs::write(p.join("README.md"), "# Documentation\nFix typo\n").unwrap();
    run_git(p, &["add", "."]);

    run_git(p, &["commit", "-m", "fix typo in README"]);

    let res = check_single_commit("HEAD", Some(p));
    assert!(
        res.is_ok(),
        "Simple honest commit with single line message must pass: {:?}",
        res.err()
    );
}

#[test]
fn test_count_claim_points_heuristics() {
    let msg1 = "fix: simple fix";
    assert_eq!(count_claim_points(msg1), 0);

    let msg2 = r#"feat(scope): title

- implementiert A
- behebt B
- fügt C hinzu
"#;
    assert_eq!(count_claim_points(msg2), 3);
}

#[test]
fn test_parse_git_stat_output() {
    let stat_str = " 3 files changed, 20 insertions(+), 5 deletions(-)\n";
    let parsed = parse_git_stat_output(stat_str).unwrap();
    assert_eq!(parsed.changed_files, 3);
    assert_eq!(parsed.insertions, 20);
    assert_eq!(parsed.deletions, 5);
}
