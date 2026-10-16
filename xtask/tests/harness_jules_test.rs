use std::fs;
use tempfile::TempDir;

#[path = "../src/harness/jules.rs"]
mod jules;

fn jules_facade_test_setup_repo() -> (TempDir, std::path::PathBuf) {
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

    fs::write(
        root.join("xtask/Cargo.toml"),
        r#"[package]
name = "xtask"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "xtask"
path = "src/main.rs"
"#,
    )
    .unwrap();

    fs::write(
        root.join("xtask/src/main.rs"),
        r#"fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("");
    if cmd == "nonexistent-cmd" {
        eprintln!("Unknown xtask command: {}", cmd);
        std::process::exit(1);
    }
}
"#,
    )
    .unwrap();

    (temp_dir, root)
}

#[test]
fn test_jules_facade_pass_all_steps() {
    let (_temp, root) = jules_facade_test_setup_repo();

    let config = jules::JulesFacadeConfig {
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

    let (code, res) = jules::jules_facade_execute_phase(&root, "start", &[], Some(&config));
    assert_eq!(code, 0);
    assert_eq!(res.get("status").unwrap(), "pass");

    // Exercise functions to satisfy dead-code checks
    let _ = jules::jules_facade_find_root(&root);
    let _ = jules::run_jules(&[
        "start".to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
    ]);
}

#[test]
fn test_jules_facade_missing_required_command_exit_2_with_session_hint() {
    let (_temp, root) = jules_facade_test_setup_repo();

    let config = jules::JulesFacadeConfig {
        phase: jules::JulesFacadePhaseConfig {
            start: Some(vec![jules::JulesFacadeStep {
                cmd: vec![
                    "cargo".to_string(),
                    "run".to_string(),
                    "--manifest-path".to_string(),
                    "xtask/Cargo.toml".to_string(),
                    "--".to_string(),
                    "nonexistent-cmd".to_string(),
                ],
                required: true,
            }]),
            check: None,
            verify: None,
            submit: None,
            stop: None,
        },
    };

    let (code, res) = jules::jules_facade_execute_phase(&root, "start", &[], Some(&config));
    assert_eq!(code, 2);
    assert_eq!(res.get("status").unwrap(), "error");
    let reasons = res.get("reasons").unwrap().as_array().unwrap();
    assert!(!reasons.is_empty());
    let hint = reasons[0].as_str().unwrap();
    assert!(hint.contains("Pflichtkommando 'nonexistent-cmd' existiert noch nicht"));
}

#[test]
fn test_jules_facade_missing_optional_command_warns_only() {
    let (_temp, root) = jules_facade_test_setup_repo();

    let config = jules::JulesFacadeConfig {
        phase: jules::JulesFacadePhaseConfig {
            start: Some(vec![jules::JulesFacadeStep {
                cmd: vec![
                    "cargo".to_string(),
                    "run".to_string(),
                    "--manifest-path".to_string(),
                    "xtask/Cargo.toml".to_string(),
                    "--".to_string(),
                    "env-attest".to_string(),
                ],
                required: false,
            }]),
            check: None,
            verify: None,
            submit: None,
            stop: None,
        },
    };

    let (code, res) = jules::jules_facade_execute_phase(&root, "start", &[], Some(&config));
    assert_eq!(code, 0);
    assert_eq!(res.get("status").unwrap(), "pass");
}
