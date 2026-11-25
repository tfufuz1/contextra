// xtask/tests/harness_repo_doctor_test.rs

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

#[path = "../src/harness/repo_doctor.rs"]
mod repo_doctor;

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
fn test_run_repo_doctor_quick_json() {
    let root = get_repo_root();
    let root_str = root.to_string_lossy();

    let args = vec![
        "--quick".to_string(),
        "--json".to_string(),
        "--root".to_string(),
        root_str.to_string(),
    ];

    let exit_code = repo_doctor::run_repo_doctor(&args);
    // Smoke check exit code is valid (0, 1, or 2)
    assert!(exit_code == 0 || exit_code == 1 || exit_code == 2);
}
