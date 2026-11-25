//! Integration test for `commit_health`

#[path = "../src/shell_commit_audit.rs"]
mod shell_commit_audit;

#[path = "../src/audit_integrity_check.rs"]
mod audit_integrity_check;

#[path = "../src/hotspot_report.rs"]
mod hotspot_report;

#[path = "../src/commit_health.rs"]
mod commit_health;

use commit_health::run_commit_health_impl;
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
fn test_commit_health_report_generation() {
    let repo = create_test_repo();
    let p = repo.path();

    let crate_dir = p.join("crates/contextra-core/src");
    fs::create_dir_all(&crate_dir).unwrap();

    let lib_rs = crate_dir.join("lib.rs");
    fs::write(&lib_rs, "// Initial content").unwrap();

    run_git_env(p, &["add", "."], &[]);
    run_git_env(
        p,
        &["commit", "-m", "feat: initial commit"],
        &[
            ("GIT_AUTHOR_DATE", "2026-09-01T10:00:00Z"),
            ("GIT_COMMITTER_DATE", "2026-09-01T10:00:00Z"),
        ],
    );

    // Add a Shell-Commit
    fs::write(&lib_rs, "// Updated content").unwrap();
    run_git_env(p, &["add", "."], &[]);
    run_git_env(
        p,
        &["commit", "-m", "Shell-Commit"],
        &[
            ("GIT_AUTHOR_DATE", "2026-09-01T11:00:00Z"),
            ("GIT_COMMITTER_DATE", "2026-09-01T11:00:00Z"),
        ],
    );

    let report_out_path = p.join("reports/commit_health.md");

    let report_md = run_commit_health_impl(p, 30, Some(&report_out_path))
        .expect("Commit health report generation should succeed");

    assert!(report_md.contains("# Commit-Gesundheitsbericht — Stand "));
    assert!(report_md.contains("## Zusammenfassung"));
    assert!(report_md.contains("## Top 5 Hotspots (meistgeänderte Dateien)"));
    assert!(report_out_path.exists());

    let written_content = fs::read_to_string(&report_out_path).unwrap();
    assert_eq!(written_content, report_md);
}
