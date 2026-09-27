//! Integration test for `hotspot_report`

#[path = "../src/hotspot_report.rs"]
mod hotspot_report;

use hotspot_report::{run_hotspot_report_impl, HotspotRisk};
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
fn test_hotspot_report_aggregation() {
    let repo = create_test_repo();
    let p = repo.path();

    let crate_dir = p.join("crates/contextra-core/src");
    fs::create_dir_all(&crate_dir).unwrap();

    let file_a = crate_dir.join("lib.rs");
    let file_b = crate_dir.join("utils.rs");

    // File A content with 2 distinct SESSION author hashes
    let content_a = r#"
// AI-TAG[SMELL] (SESSION: 1111a111)
// AI-TAG[PERF] (SESSION: 2222b222)
pub fn add(a: i32, b: i32) -> i32 { a + b }
"#;
    fs::write(&file_a, content_a).unwrap();
    fs::write(&file_b, "// utils").unwrap();

    run_git(p, &["add", "."]);
    run_git(p, &["commit", "-m", "initial commit"]);

    // Make 21 commits modifying file_a to trigger Elevated risk (>20 changes)
    for i in 0..21 {
        fs::write(&file_a, format!("{}\n// edit {}", content_a, i)).unwrap();
        run_git(p, &["add", "."]);
        run_git(p, &["commit", "-m", &format!("update file_a {}", i)]);
    }

    // Make 2 commits modifying file_b
    for i in 0..2 {
        fs::write(&file_b, format!("// utils edit {}", i)).unwrap();
        run_git(p, &["add", "."]);
        run_git(p, &["commit", "-m", &format!("update file_b {}", i)]);
    }

    let hotspots =
        run_hotspot_report_impl(p, 30, 10, false).expect("Hotspot report should succeed");

    assert_eq!(hotspots.len(), 2);

    let top = &hotspots[0];
    assert_eq!(top.file, "crates/contextra-core/src/lib.rs");
    assert_eq!(top.changes_in_window, 22);
    assert_eq!(top.distinct_authors, 2);
    assert_eq!(top.risk_level, HotspotRisk::Elevated);

    let second = &hotspots[1];
    assert_eq!(second.file, "crates/contextra-core/src/utils.rs");
    assert_eq!(second.changes_in_window, 3);
    assert_eq!(second.risk_level, HotspotRisk::Normal);
}
