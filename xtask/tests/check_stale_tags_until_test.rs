//! Integration tests for `check_stale_tags` `until=YYYY-MM-DD` expiration functionality.

#[path = "../src/check_stale_tags.rs"]
mod check_stale_tags;

use check_stale_tags::{check_stale_tags_impl, TAG_UNTIL_MANDATORY_CUTOFF_DATE};
use chrono::{DateTime, Utc};
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

fn create_test_repo_with_code(code_content: &str) -> tempfile::TempDir {
    let dir = tempdir().expect("Failed to create temp dir");
    let p = dir.path();

    run_git_env(p, &["init"], &[]);
    run_git_env(p, &["config", "user.name", "Test Agent"], &[]);
    run_git_env(p, &["config", "user.email", "agent@example.com"], &[]);

    let crate_dir = p.join("crates/contextra-test/src");
    fs::create_dir_all(&crate_dir).unwrap();

    let lib_rs = crate_dir.join("lib.rs");
    fs::write(&lib_rs, code_content).unwrap();

    run_git_env(p, &["add", "."], &[]);
    run_git_env(
        p,
        &["commit", "-m", "initial commit with tags"],
        &[
            ("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z"),
            ("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z"),
        ],
    );

    dir
}

#[test]
fn test_until_expired_is_red() {
    let code = r#"
// Line 1
// AI-TAG[SMELL][MINOR] TODO(audit-1.1): Expired tag until=2026-01-01 (ID: AGT-TEST-0001) (TS: 2026-01-01T00:00:00Z)
"#;
    let repo = create_test_repo_with_code(code);
    let now = DateTime::parse_from_rfc3339("2026-09-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let result = check_stale_tags_impl(repo.path(), 60, false, now);
    assert!(
        result.is_err(),
        "Expected Err for expired tag with until=2026-01-01, got Ok"
    );
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("stale/expired audit tag candidate(s) detected"),
        "Unexpected error message: {}",
        err_msg
    );
}

#[test]
fn test_until_today_is_green() {
    let code = r#"
// Line 1
// AI-TAG[SMELL][MINOR] TODO(audit-1.2): Active tag today until=2026-09-01 (ID: AGT-TEST-0002) (TS: 2026-01-01T00:00:00Z)
"#;
    let repo = create_test_repo_with_code(code);
    let now = DateTime::parse_from_rfc3339("2026-09-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let result = check_stale_tags_impl(repo.path(), 60, false, now);
    assert!(
        result.is_ok(),
        "Expected Ok for tag with until=2026-09-01 on same date, got Err: {:?}",
        result
    );
}

#[test]
fn test_without_until_is_warning_before_cutoff() {
    let code = r#"
// Line 1
// AI-TAG[SMELL][MINOR] TODO(audit-1.3): Tag without until (ID: AGT-TEST-0003) (TS: 2026-08-15T00:00:00Z)
"#;
    let repo = create_test_repo_with_code(code);
    let now = DateTime::parse_from_rfc3339("2026-09-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    // threshold 60 days means 17 days age is not stale yet, but missing until is logged as warning before cutoff
    let result = check_stale_tags_impl(repo.path(), 60, false, now);
    assert!(
        result.is_ok(),
        "Expected Ok (Warning) for tag without until before cutoff date, got Err: {:?}",
        result
    );
}

#[test]
fn test_invalid_until_date_is_red() {
    let code = r#"
// Line 1
// AI-TAG[SMELL][MINOR] TODO(audit-1.4): Invalid until date until=2026-13-45 (ID: AGT-TEST-0004) (TS: 2026-01-01T00:00:00Z)
"#;
    let repo = create_test_repo_with_code(code);
    let now = DateTime::parse_from_rfc3339("2026-09-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let result = check_stale_tags_impl(repo.path(), 60, false, now);
    assert!(
        result.is_err(),
        "Expected Err for tag with invalid until date 2026-13-45, got Ok"
    );
}

#[test]
fn test_without_until_after_cutoff_is_red() {
    let code = r#"
// Line 1
// AI-TAG[SMELL][MINOR] TODO(audit-1.5): Tag without until after cutoff (ID: AGT-TEST-0005) (TS: 2026-01-01T00:00:00Z)
"#;
    let repo = create_test_repo_with_code(code);
    let now = DateTime::parse_from_rfc3339("2027-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let result = check_stale_tags_impl(repo.path(), 60, false, now);
    assert!(
        result.is_err(),
        "Expected Err for tag without until after mandatory cutoff date ({}), got Ok",
        TAG_UNTIL_MANDATORY_CUTOFF_DATE
    );
}
