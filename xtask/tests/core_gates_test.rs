use std::fs;
use tempfile::TempDir;

use xtask::core::catalog::Catalog;
use xtask::core::evidence::{save_evidence, EvidenceRecord};
use xtask::core::exec::run_cmd;
use xtask::core::gate::Outcome;

#[test]
fn test_unregistered_due_gate_returns_not_run_and_exit_2() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let gov_dir = temp_dir.path().join("governance");
    fs::create_dir_all(&gov_dir).unwrap();

    let gates_toml = r#"
[meta]
active_phase = "PH0"

[[gate]]
name = "unregistered-builtin-test-gate"
tier = "pr"
from_phase = "PH0"
kind = "builtin"
enforces = ["P1"]
why = "Testing unregistered due gate"
owner = "MP-01"
"#;
    fs::write(gov_dir.join("gates.toml"), gates_toml).unwrap();

    let catalog = Catalog::load_from_path(&gov_dir.join("gates.toml")).unwrap();
    assert!(catalog.gates.contains_key("unregistered-builtin-test-gate"));
}

#[test]
fn test_outcome_not_run_exit_code() {
    let outcome = Outcome::NotRun {
        reason: "Gate not implemented".to_string(),
    };
    assert_eq!(outcome.exit_code(), 2);
    assert_eq!(outcome.status_str(), "NOT RUN (Gate not implemented)");
}

#[test]
fn test_not_due_gate_does_not_fail() {
    let outcome = Outcome::NotDue {
        reason: "Phase PH2 > PH0".to_string(),
    };
    assert_eq!(outcome.exit_code(), 0);
    assert_eq!(outcome.status_str(), "NOT DUE (Phase PH2 > PH0)");
}

#[test]
fn test_other_tier_gate_status() {
    let outcome = Outcome::OtherTier {
        tier: "nightly".to_string(),
    };
    assert_eq!(outcome.exit_code(), 0);
    assert_eq!(outcome.status_str(), "OTHER TIER (nightly)");
}

#[test]
fn test_save_evidence_readonly_dir_propagates_error() {
    let temp_dir = TempDir::new().unwrap();
    let readonly_dir = temp_dir.path().join("readonly_evidence");
    fs::create_dir_all(&readonly_dir).unwrap();

    let mut permissions = fs::metadata(&readonly_dir).unwrap().permissions();
    permissions.set_readonly(true);
    let _ = fs::set_permissions(&readonly_dir, permissions.clone());

    let record = EvidenceRecord {
        gate: "test_gate".to_string(),
        cmd: vec!["cargo".to_string(), "check".to_string()],
        toolchain: "stable".to_string(),
        commit: "abc1234".to_string(),
        exit_code: 0,
        duration_ms: 10,
        tests_run: 5,
        tests_skipped: 1,
        log_sha256: "hash123".to_string(),
        started_utc: "2026-10-27T00:00:00Z".to_string(),
    };

    let result = save_evidence(&readonly_dir, &record);
    // Cleanup permissions so tempdir can be removed
    permissions.set_readonly(false);
    let _ = fs::set_permissions(&readonly_dir, permissions);

    assert!(
        result.is_err(),
        "save_evidence on read-only directory must return an Err"
    );
}

#[test]
fn test_log_sha256_different_raw_outputs() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("script1.sh");
    let file2 = temp_dir.path().join("script2.sh");

    fs::write(&file1, "#!/bin/sh\necho 'hello'\n").unwrap();
    fs::write(&file2, "#!/bin/sh\necho 'world'\n").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&file1, fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(&file2, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let res1 = run_cmd(file1.to_str().unwrap(), &[], temp_dir.path());
    let res2 = run_cmd(file2.to_str().unwrap(), &[], temp_dir.path());
    let res1_repeat = run_cmd(file1.to_str().unwrap(), &[], temp_dir.path());

    assert_ne!(
        res1.log_sha256, res2.log_sha256,
        "Different outputs must yield different log_sha256 hashes"
    );
    assert_eq!(
        res1.log_sha256, res1_repeat.log_sha256,
        "Identical outputs must yield identical log_sha256 hashes"
    );
}

#[test]
fn test_test_counter_parsing_libtest_output() {
    let output = r#"
running 15 tests
test test_foo ... ok
test test_bar ... ok
test test_baz ... ignored

test result: ok. 14 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.12s
"#;

    let (tests_run, tests_skipped) = xtask::core::exec::parse_test_counts(output).unwrap();
    assert_eq!(tests_run, 14);
    assert_eq!(tests_skipped, 1);
}
