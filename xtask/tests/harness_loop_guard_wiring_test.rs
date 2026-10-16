use std::fs;
use tempfile::TempDir;

#[path = "../src/harness/jules.rs"]
mod jules;

#[path = "../src/harness/loop_guard.rs"]
mod loop_guard;

fn setup_wiring_test_repo() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    fs::create_dir_all(root.join(".jules")).unwrap();
    fs::create_dir_all(root.join("xtask/src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"xtask\"]\n",
    )
    .unwrap();
    fs::write(
        root.join("capabilities.toml"),
        "[crates.contextra-core]\nstatus = \"active\"\n",
    )
    .unwrap();

    (temp_dir, root)
}

#[test]
fn test_loop_guard_wiring_repeated_error_escalates() {
    let (_temp, root) = setup_wiring_test_repo();

    let config = jules::JulesFacadeConfig {
        phase: jules::JulesFacadePhaseConfig {
            start: None,
            check: Some(vec![jules::JulesFacadeStep {
                cmd: vec![
                    "cargo".to_string(),
                    "--invalid-flag-wiring-test".to_string(),
                ],
                required: true,
            }]),
            verify: None,
            submit: None,
            stop: None,
        },
    };

    // First failed run
    let (code1, res1) = jules::jules_facade_execute_phase(&root, "check", &[], Some(&config));
    assert_eq!(code1, 1);
    assert_eq!(res1.get("status").unwrap(), "fail");

    // Second failed run with same error
    let (code2, res2) = jules::jules_facade_execute_phase(&root, "check", &[], Some(&config));
    assert_ne!(code2, 0);

    // After second failure, loop-guard should record count = 2, escalate, and create ESCALATE.md
    let escalate_file = root.join(".jules/local/ESCALATE.md");
    assert!(
        escalate_file.exists(),
        "ESCALATE.md must be written after second identical gate error"
    );

    let escalate_content = fs::read_to_string(&escalate_file).unwrap();
    assert!(escalate_content.contains("ESCALATION REPORT"));

    // Verify JSON response contains loop guard escalation info
    let step = &res2.get("steps").unwrap().as_array().unwrap()[0];
    assert_eq!(step.get("escalated").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(step.get("count").and_then(|v| v.as_u64()), Some(2));
    assert!(step.get("hash").is_some());
}

#[test]
fn test_loop_guard_wiring_different_errors_no_escalation() {
    let (_temp, root) = setup_wiring_test_repo();

    let config1 = jules::JulesFacadeConfig {
        phase: jules::JulesFacadePhaseConfig {
            start: None,
            check: Some(vec![jules::JulesFacadeStep {
                cmd: vec!["cargo".to_string(), "--invalid-flag-1".to_string()],
                required: true,
            }]),
            verify: None,
            submit: None,
            stop: None,
        },
    };

    let config2 = jules::JulesFacadeConfig {
        phase: jules::JulesFacadePhaseConfig {
            start: None,
            check: Some(vec![jules::JulesFacadeStep {
                cmd: vec!["cargo".to_string(), "--invalid-flag-2".to_string()],
                required: true,
            }]),
            verify: None,
            submit: None,
            stop: None,
        },
    };

    let (_code1, _res1) = jules::jules_facade_execute_phase(&root, "check", &[], Some(&config1));
    let (_code2, res2) = jules::jules_facade_execute_phase(&root, "check", &[], Some(&config2));

    let escalate_file = root.join(".jules/local/ESCALATE.md");
    assert!(
        !escalate_file.exists(),
        "ESCALATE.md must NOT be created when error hashes differ"
    );

    let step = &res2.get("steps").unwrap().as_array().unwrap()[0];
    assert_ne!(step.get("escalated").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn test_loop_guard_wiring_success_does_not_increment_count() {
    let (_temp, root) = setup_wiring_test_repo();

    let config_success = jules::JulesFacadeConfig {
        phase: jules::JulesFacadePhaseConfig {
            start: Some(vec![jules::JulesFacadeStep {
                cmd: vec!["cargo".to_string(), "--version".to_string()],
                required: true,
            }]),
            check: None,
            verify: None,
            submit: None,
            stop: None,
        },
    };

    let (code, res) = jules::jules_facade_execute_phase(&root, "start", &[], Some(&config_success));
    assert_eq!(code, 0);
    assert_eq!(res.get("status").unwrap(), "pass");

    let state = loop_guard::loop_guard_load_state(&root);
    assert!(state.error_counts.is_empty());
}
