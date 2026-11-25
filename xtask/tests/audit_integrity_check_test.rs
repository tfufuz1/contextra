//! Integration test for `audit_integrity_check`

#[path = "../src/audit_integrity_check.rs"]
mod audit_integrity_check;

use audit_integrity_check::{run_audit_integrity_check_impl, GapSeverity};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn run_git_env(dir: &Path, args: &[&str], envs: &[(&str, &str)]) {
    let mut cmd = Command::new("git");
    cmd.current_dir(dir);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.args(args);
    let output = cmd
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

    run_git_env(p, &["init"], &[]);
    run_git_env(p, &["config", "user.name", "Test Agent"], &[]);
    run_git_env(p, &["config", "user.email", "agent@example.com"], &[]);

    dir
}

#[test]
fn test_audit_integrity_check_detection() {
    let repo = create_test_repo();
    let p = repo.path();

    let crate_dir = p.join("crates/contextra-core");
    fs::create_dir_all(&crate_dir).unwrap();
    let target_file = crate_dir.join("lib.rs");

    fs::write(&target_file, "// Initial content").unwrap();
    run_git_env(p, &["add", "."], &[]);
    run_git_env(
        p,
        &["commit", "-m", "initial commit"],
        &[
            ("GIT_AUTHOR_DATE", "2026-09-01T10:00:00Z"),
            ("GIT_COMMITTER_DATE", "2026-09-01T10:00:00Z"),
        ],
    );

    // Audit commit at 10:00
    fs::write(&target_file, "// Audited content").unwrap();
    run_git_env(p, &["add", "."], &[]);
    run_git_env(
        p,
        &[
            "commit",
            "-m",
            "audit(core): verification\n\nVERDICT: GO/APPROVED",
        ],
        &[
            ("GIT_AUTHOR_DATE", "2026-09-01T10:00:00Z"),
            ("GIT_COMMITTER_DATE", "2026-09-01T10:00:00Z"),
        ],
    );

    // Fix commit 1 hour later (11:00) -> Should be Contradicted (<=2h)
    fs::write(&target_file, "// Fixed content").unwrap();
    run_git_env(p, &["add", "."], &[]);
    run_git_env(
        p,
        &["commit", "-m", "fix(core): immediate bugfix after approval"],
        &[
            ("GIT_AUTHOR_DATE", "2026-09-01T11:00:00Z"),
            ("GIT_COMMITTER_DATE", "2026-09-01T11:00:00Z"),
        ],
    );

    let gaps = run_audit_integrity_check_impl(p, None, false).expect("Integrity check should run");
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].verdict, "GO/APPROVED");
    assert_eq!(gaps[0].affected_crate, "contextra-core");

    match gaps[0].severity {
        GapSeverity::Contradicted(gap_h) => {
            assert!((gap_h - 1.0).abs() < 0.1);
        }
        _ => panic!("Expected Contradicted severity"),
    }

    // Check fail_on_contradicted
    let fail_res = run_audit_integrity_check_impl(p, None, true);
    assert!(fail_res.is_err());
    assert!(fail_res
        .unwrap_err()
        .contains("contradicted audit-to-fix gap"));
}
