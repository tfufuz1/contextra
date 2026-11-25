// Test for harness module ledger

#[path = "../src/harness/ledger.rs"]
mod ledger;

use std::fs;
use std::process::Command;

fn setup_temp_repo() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = temp_dir.path();

    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(path)
            .args(args)
            .status()
            .expect("failed to run git");
        assert!(status.success());
    };

    run_git(&["init"]);
    run_git(&["config", "user.name", "Test User"]);
    run_git(&["config", "user.email", "test@example.com"]);

    fs::write(path.join(".gitignore"), "/.jules/local/\n").unwrap();
    fs::write(path.join("file1.txt"), "hello world\n").unwrap();
    run_git(&["add", ".gitignore", "file1.txt"]);
    run_git(&["commit", "-m", "initial commit"]);

    temp_dir
}

#[test]
fn test_ledger_tree_hash_subcommand() {
    let repo = setup_temp_repo();
    let root_str = repo.path().to_str().unwrap();

    let args = vec![
        "tree-hash".to_string(),
        "--root".to_string(),
        root_str.to_string(),
    ];
    let code = ledger::run_ledger(&args);
    assert_eq!(code, 0);

    let hash = ledger::ledger_compute_tree_hash(repo.path()).unwrap();
    assert!(!hash.is_empty());

    // Add untracked file -> hash must change
    fs::write(repo.path().join("untracked.txt"), "untracked\n").unwrap();
    let new_hash = ledger::ledger_compute_tree_hash(repo.path()).unwrap();
    assert_ne!(hash, new_hash);
}

#[test]
fn test_ledger_write_and_verify_pass() {
    let repo = setup_temp_repo();
    let root_str = repo.path().to_str().unwrap();

    let external_dir = tempfile::tempdir().expect("failed external tempdir");
    let results_dir = external_dir.path().join("results");
    fs::create_dir_all(&results_dir).unwrap();

    let gate1 = serde_json::json!({
        "gate": "fmt",
        "status": "pass",
        "summary": "fmt ok",
        "findings": []
    });
    fs::write(
        results_dir.join("gate1.json"),
        serde_json::to_string_pretty(&gate1).unwrap(),
    )
    .unwrap();

    let ledger_file = external_dir.path().join("ledger.json");
    let ledger_str = ledger_file.to_str().unwrap();

    let write_args = vec![
        "write".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--results-dir".to_string(),
        results_dir.to_str().unwrap().to_string(),
        "--out".to_string(),
        ledger_str.to_string(),
    ];

    let write_code = ledger::run_ledger(&write_args);
    assert_eq!(write_code, 0);

    // Call ledger_write directly to cover all pub fn
    let direct_write = ledger::ledger_write(repo.path(), &results_dir, &ledger_file);
    assert!(direct_write.is_ok());

    // Call ledger_get_head_commit directly to cover all pub fn
    let head = ledger::ledger_get_head_commit(repo.path());
    assert!(head.is_ok());

    // Verify
    let verify_args = vec![
        "verify".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--ledger".to_string(),
        ledger_str.to_string(),
    ];

    let verify_code = ledger::run_ledger(&verify_args);
    assert_eq!(verify_code, 0);

    // Call ledger_verify directly
    let verify_res = ledger::ledger_verify(repo.path(), &ledger_file);
    assert!(verify_res.is_ok());
}

#[test]
fn test_ledger_verify_hash_mismatch_and_fail_status() {
    let repo = setup_temp_repo();
    let root_str = repo.path().to_str().unwrap();

    let external_dir = tempfile::tempdir().expect("failed external tempdir");
    let results_dir = external_dir.path().join("results");
    fs::create_dir_all(&results_dir).unwrap();

    let gate_fail = serde_json::json!({
        "gate": "clippy",
        "status": "fail",
        "summary": "clippy warning",
        "findings": []
    });
    fs::write(
        results_dir.join("gate_fail.json"),
        serde_json::to_string_pretty(&gate_fail).unwrap(),
    )
    .unwrap();

    let ledger_file = external_dir.path().join("ledger_fail.json");
    let ledger_str = ledger_file.to_str().unwrap();

    let write_args = vec![
        "write".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--results-dir".to_string(),
        results_dir.to_str().unwrap().to_string(),
        "--out".to_string(),
        ledger_str.to_string(),
    ];
    assert_eq!(ledger::run_ledger(&write_args), 0);

    // Verification fails because status is FAIL
    let verify_args = vec![
        "verify".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--ledger".to_string(),
        ledger_str.to_string(),
    ];
    assert_eq!(ledger::run_ledger(&verify_args), 1);

    // Now test hash mismatch
    // Modify repo state after writing valid PASS ledger
    let results_dir_pass = external_dir.path().join("results_pass");
    fs::create_dir_all(&results_dir_pass).unwrap();

    let gate_pass = serde_json::json!({
        "gate": "fmt",
        "status": "pass",
        "summary": "fmt ok",
        "findings": []
    });
    fs::write(
        results_dir_pass.join("gate.json"),
        serde_json::to_string_pretty(&gate_pass).unwrap(),
    )
    .unwrap();

    let ledger_pass = external_dir.path().join("ledger_pass.json");
    ledger::ledger_write(repo.path(), &results_dir_pass, &ledger_pass).unwrap();

    // Modify repo (add untracked file)
    fs::write(repo.path().join("modified.txt"), "mod").unwrap();

    // Verify should fail due to hash mismatch
    let verify_code_mismatch = ledger::run_ledger(&[
        "verify".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--ledger".to_string(),
        ledger_pass.to_str().unwrap().to_string(),
    ]);
    assert_eq!(verify_code_mismatch, 1);
}

#[test]
fn test_ledger_json_output() {
    let repo = setup_temp_repo();
    let root_str = repo.path().to_str().unwrap();

    let args = vec![
        "tree-hash".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--json".to_string(),
    ];
    let code = ledger::run_ledger(&args);
    assert_eq!(code, 0);
}
