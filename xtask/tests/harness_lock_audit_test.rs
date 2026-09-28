#[path = "../src/harness/lock_audit.rs"]
mod lock_audit;

use std::fs;
use tempfile::TempDir;

#[test]
fn test_lock_audit_harness() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let _ = std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output();

    let crate_dir = root.join("crates/contextra-store");
    let src_dir = crate_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    fs::write(
        crate_dir.join("AGENTS.md"),
        "# Lock Hierarchy\nwrite_lock > memtable > sstables\n",
    )
    .unwrap();

    fs::write(
        src_dir.join("locks.rs"),
        "pub async fn test_lock() {\n  a.lock().await;\n  b.lock().await;\n}\n",
    )
    .unwrap();

    let mut findings = vec![];
    lock_audit::lock_audit_scan_file_fallback(
        &src_dir.join("locks.rs"),
        "crates/contextra-store/src/locks.rs",
        &mut findings,
    );
    assert!(!findings.is_empty());

    let mut agent_findings = vec![];
    lock_audit::lock_audit_check_agents_md(&crate_dir, "contextra-store", &mut agent_findings);
    assert!(!agent_findings.is_empty());

    let touched = lock_audit::lock_audit_get_touched_crates(root, "HEAD", "HEAD").unwrap();
    assert!(touched.is_empty());

    let args = vec![
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--json".to_string(),
    ];
    let exit_code = lock_audit::run_lock_audit(&args);
    assert_eq!(exit_code, 0); // Not applicable when no crates touched in git diff
}
