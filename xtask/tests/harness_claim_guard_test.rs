// xtask/tests/harness_claim_guard_test.rs

use std::path::PathBuf;

#[path = "../src/shell_commit_audit.rs"]
mod shell_commit_audit;

#[path = "../src/audit_integrity_check.rs"]
mod audit_integrity_check;

#[path = "../src/hotspot_report.rs"]
mod hotspot_report;

#[path = "../src/commit_health.rs"]
mod commit_health;

#[path = "../src/workspace_verify.rs"]
mod workspace_verify;

#[path = "../src/env_validate.rs"]
mod env_validate;

#[path = "../src/claim.rs"]
mod claim;

#[path = "../src/agent_lifecycle/mod.rs"]
mod agent_lifecycle;

#[path = "../src/harness/claim_guard.rs"]
mod claim_guard;

fn find_root_dir() -> PathBuf {
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return PathBuf::from(path_str);
            }
        }
    }
    PathBuf::from(".")
}

fn get_repo_root() -> PathBuf {
    find_root_dir()
}

#[test]
fn test_run_claim_guard_quick() {
    let root = get_repo_root();
    let root_str = root.to_string_lossy();

    let args = vec![
        "--root".to_string(),
        root_str.to_string(),
        "--base".to_string(),
        "HEAD".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
    ];

    let exit_code = claim_guard::run_claim_guard(&args);
    // Smoke check exit code is valid (0, 1, or 2)
    assert!(exit_code == 0 || exit_code == 1 || exit_code == 2);
}
