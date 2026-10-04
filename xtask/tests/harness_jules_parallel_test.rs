use std::fs;
use tempfile::TempDir;

#[path = "../src/harness/jules.rs"]
mod jules;

fn setup_parallel_test_repo() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    fs::create_dir_all(root.join(".jules")).unwrap();
    fs::create_dir_all(root.join("xtask/src")).unwrap();
    fs::create_dir_all(root.join("crates/contextra-core/src")).unwrap();
    fs::create_dir_all(root.join("crates/contextra-types/src")).unwrap();

    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/contextra-core\", \"crates/contextra-types\", \"xtask\"]\n",
    )
    .unwrap();

    fs::write(
        root.join("capabilities.toml"),
        r#"[crates.contextra-core]
ring = "Ring 1"
test = "cargo test -p contextra-core"
may_depend_on = ["contextra-types"]

[crates.contextra-types]
ring = "Ring 0"
test = "cargo test -p contextra-types"
may_depend_on = []
"#,
    )
    .unwrap();

    fs::write(
        root.join("crates/contextra-core/Cargo.toml"),
        "[package]\nname = \"contextra-core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    fs::write(
        root.join("crates/contextra-types/Cargo.toml"),
        "[package]\nname = \"contextra-types\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
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
    if args.len() > 1 {
        match args[1].as_str() {
            "pass-a" => println!("PASS_A"),
            "pass-b" => println!("PASS_B"),
            "fail-c" => {
                eprintln!("FAIL_C");
                std::process::exit(1);
            }
            _ => {
                println!("OK");
            }
        }
    }
}
"#,
    )
    .unwrap();

    (temp_dir, root)
}

#[test]
fn test_parallel_group_deterministic_order_and_no_early_exit() {
    let (_temp, root) = setup_parallel_test_repo();

    let config = jules::JulesFacadeConfig {
        phase: jules::JulesFacadePhaseConfig {
            start: None,
            check: Some(vec![
                jules::JulesFacadeStep {
                    cmd: vec!["cargo".to_string(), "--version".to_string()],
                    required: true,
                    parallel_group: Some("group1".to_string()),
                    when_changed: None,
                    serial: false,
                },
                jules::JulesFacadeStep {
                    cmd: vec!["cargo".to_string(), "nonexistent_subcmd_test_xyz".to_string()],
                    required: true,
                    parallel_group: Some("group1".to_string()),
                    when_changed: None,
                    serial: false,
                },
                jules::JulesFacadeStep {
                    cmd: vec!["cargo".to_string(), "--help".to_string()],
                    required: true,
                    parallel_group: Some("group1".to_string()),
                    when_changed: None,
                    serial: false,
                },
            ]),
            verify: None,
            submit: None,
            stop: None,
        },
    };

    let (code, res) = jules::jules_facade_execute_phase(&root, "check", &[], Some(&config));
    assert!(code != 0);

    let steps = res.get("steps").unwrap().as_array().unwrap();
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0]["command"], "cargo --version");
    assert_eq!(steps[1]["command"], "cargo nonexistent_subcmd_test_xyz");
    assert_eq!(steps[2]["command"], "cargo --help");
}

#[test]
fn test_when_changed_skip_and_failsafe() {
    let (_temp, root) = setup_parallel_test_repo();

    // 1. Without git repo (fail-safe test): when_changed step MUST NOT skip when diff cannot be determined
    let config_failsafe = jules::JulesFacadeConfig {
        phase: jules::JulesFacadePhaseConfig {
            start: None,
            check: Some(vec![
                jules::JulesFacadeStep {
                    cmd: vec!["cargo".to_string(), "--version".to_string()],
                    required: true,
                    parallel_group: None,
                    when_changed: Some(vec!["non_existent_path/**/*.rs".to_string()]),
                    serial: false,
                },
            ]),
            verify: None,
            submit: None,
            stop: None,
        },
    };

    let (code1, res1) = jules::jules_facade_execute_phase(&root, "check", &[], Some(&config_failsafe));
    assert_eq!(code1, 0);
    let steps1 = res1.get("steps").unwrap().as_array().unwrap();
    assert_eq!(steps1[0]["status"], "pass", "Fail-safe behavior should run the step when git diff fails");

    // 2. With git repo: initialize git and commit initial files on 'main'
    let _ = std::process::Command::new("git").args(["init", "-b", "main"]).current_dir(&root).output();
    let _ = std::process::Command::new("git").args(["config", "user.name", "Test"]).current_dir(&root).output();
    let _ = std::process::Command::new("git").args(["config", "user.email", "test@example.com"]).current_dir(&root).output();
    let _ = std::process::Command::new("git").args(["add", "."]).current_dir(&root).output();
    let _ = std::process::Command::new("git").args(["commit", "-m", "initial"]).current_dir(&root).output();

    // Create feature branch and modify file
    let _ = std::process::Command::new("git").args(["checkout", "-b", "feature"]).current_dir(&root).output();
    fs::write(root.join("crates/contextra-core/src/lib.rs"), "// mod\n").unwrap();
    let _ = std::process::Command::new("git").args(["add", "."]).current_dir(&root).output();
    let _ = std::process::Command::new("git").args(["commit", "-m", "feature change"]).current_dir(&root).output();

    // Step with non-matching pattern should skip
    let config_skip = jules::JulesFacadeConfig {
        phase: jules::JulesFacadePhaseConfig {
            start: None,
            check: Some(vec![
                jules::JulesFacadeStep {
                    cmd: vec!["cargo".to_string(), "--version".to_string()],
                    required: true,
                    parallel_group: None,
                    when_changed: Some(vec!["crates/contextra-types/**/*.rs".to_string()]),
                    serial: false,
                },
                jules::JulesFacadeStep {
                    cmd: vec!["cargo".to_string(), "--version".to_string()],
                    required: true,
                    parallel_group: None,
                    when_changed: Some(vec!["crates/contextra-core/**/*.rs".to_string()]),
                    serial: false,
                },
            ]),
            verify: None,
            submit: None,
            stop: None,
        },
    };

    let (code2, res2) = jules::jules_facade_execute_phase(&root, "check", &[], Some(&config_skip));
    assert_eq!(code2, 0);
    let steps2 = res2.get("steps").unwrap().as_array().unwrap();
    assert_eq!(steps2[0]["status"], "skipped");
    assert_eq!(steps2[0]["message"], "SKIPPED(kein passender Pfad im Diff)");
    assert_eq!(steps2[1]["status"], "pass");
}

#[test]
fn test_pkgs_expansion_workspace_and_timings() {
    let (_temp, root) = setup_parallel_test_repo();

    let config = jules::JulesFacadeConfig {
        phase: jules::JulesFacadePhaseConfig {
            start: None,
            check: Some(vec![
                jules::JulesFacadeStep {
                    cmd: vec!["cargo".to_string(), "check".to_string(), "-h".to_string(), "{pkgs}".to_string()],
                    required: true,
                    parallel_group: None,
                    when_changed: None,
                    serial: false,
                },
            ]),
            verify: None,
            submit: None,
            stop: None,
        },
    };

    let extra_args = vec!["--full".to_string(), "--timings".to_string()];
    let (code, res) = jules::jules_facade_execute_phase(&root, "check", &extra_args, Some(&config));
    assert_eq!(code, 0);

    assert!(res.get("duration_ms").is_some());
    assert!(res.get("duration_str").is_some());

    let steps = res.get("steps").unwrap().as_array().unwrap();
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0]["status"], "pass");
    assert!(steps[0].get("duration_ms").is_some());
    assert!(steps[0].get("duration_str").is_some());
}

#[test]
fn test_backwards_compatibility_harness_phases_toml() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let config_path = root.join(".jules/harness-phases.toml");
    let content = fs::read_to_string(&config_path).expect("harness-phases.toml readable");
    let config: jules::JulesFacadeConfig = toml::from_str(&content).expect("harness-phases.toml parses");

    // Verify all steps deserialize cleanly with optional fields defaulted
    let check_steps = config.phase.check.expect("check phase steps exist");
    assert!(!check_steps.is_empty());
    for step in check_steps {
        assert!(!step.cmd.is_empty());
        assert!(!step.serial);
    }
}
