//! Integration test for `tag_health`

#[path = "../src/check_stale_tags.rs"]
mod check_stale_tags;

#[path = "../src/tag_health.rs"]
mod tag_health;

use std::fs;
use std::path::Path;
use std::process::Command;
use tag_health::{run_tag_health_impl, TagHealthConcern};
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
fn test_tag_health_file_modified_after_tag() {
    let repo = create_test_repo();
    let p = repo.path();

    let crate_dir = p.join("crates/contextra-core/src");
    fs::create_dir_all(&crate_dir).unwrap();

    let lib_rs = crate_dir.join("lib.rs");

    let initial_content = r#"// Line 1
// AI-TAG[SMELL][MINOR] TODO(audit-1): Fix me (ID: AGT-001) (TS: 2026-09-01T00:00:00Z)
// Line 3
"#;
    fs::write(&lib_rs, initial_content).unwrap();

    run_git_env(p, &["add", "."], &[]);
    run_git_env(
        p,
        &["commit", "-m", "initial commit with tag"],
        &[
            ("GIT_AUTHOR_DATE", "2026-09-01T00:00:00Z"),
            ("GIT_COMMITTER_DATE", "2026-09-01T00:00:00Z"),
        ],
    );

    // Modify file line 1 at a later date (2026-09-05) without modifying tag line 2
    let updated_content = r#"// Line 1 updated
// AI-TAG[SMELL][MINOR] TODO(audit-1): Fix me (ID: AGT-001) (TS: 2026-09-01T00:00:00Z)
// Line 3
"#;
    fs::write(&lib_rs, updated_content).unwrap();

    run_git_env(p, &["add", "."], &[]);
    run_git_env(
        p,
        &["commit", "-m", "update line 1"],
        &[
            ("GIT_AUTHOR_DATE", "2026-09-05T00:00:00Z"),
            ("GIT_COMMITTER_DATE", "2026-09-05T00:00:00Z"),
        ],
    );

    let findings = run_tag_health_impl(p, 30).expect("Tag health check should succeed");

    assert!(!findings.is_empty());
    let finding = &findings[0];
    assert_eq!(finding.tag_id.as_deref(), Some("AGT-001"));

    match &finding.concern {
        TagHealthConcern::FileModifiedAfterTag {
            changes_since,
            last_change,
        } => {
            assert_eq!(*changes_since, 1);
            assert!(last_change.contains("2026-09-05"));
        }
        _ => panic!("Expected FileModifiedAfterTag concern"),
    }
}
