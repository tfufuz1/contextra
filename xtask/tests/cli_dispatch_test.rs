use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use xtask::cli::COMMAND_DISPATCH_TABLE;

fn find_root_dir() -> std::path::PathBuf {
    xtask::find_root_dir()
}

#[test]
fn test_all_registry_commands_resolvable() {
    let root = find_root_dir();
    let registry_path = root.join("xtask/registry.toml");
    let content = fs::read_to_string(&registry_path).expect("Failed to read xtask/registry.toml");

    let toml_val: toml::Value = toml::from_str(&content).expect("Failed to parse registry.toml");
    let commands = toml_val
        .get("commands")
        .and_then(|c| c.as_array())
        .expect("Missing [[commands]] in registry.toml");

    let dispatch_cmds: HashSet<&str> = COMMAND_DISPATCH_TABLE
        .iter()
        .map(|(name, _)| *name)
        .collect();

    let legacy_unmatched: HashSet<&str> = [
        "--",
        "all",
        "ann-sift1m",
        "audit",
        "beir",
        "deprecated",
        "experimental",
        "fast",
        "hardcoded-secret",
        "json",
        "onnx-test-model",
        "shell-interpolation",
        "stable",
        "std-fs-in-async",
    ]
    .into_iter()
    .collect();

    for cmd in commands {
        let name = cmd
            .get("name")
            .and_then(|n| n.as_str())
            .expect("Missing command name in registry.toml");

        if legacy_unmatched.contains(name) {
            continue;
        }

        let is_in_dispatch_table = dispatch_cmds.contains(name);
        let is_in_harness = xtask::harness::dispatch_with_builtin(name, &[]).is_some();

        assert!(
            is_in_dispatch_table || is_in_harness,
            "Command '{}' from xtask/registry.toml is not resolvable in dispatch table or harness!",
            name
        );
    }
}

#[test]
fn test_unknown_subcommand_exit_code_and_output() {
    let manifest_path = if Path::new("xtask/Cargo.toml").exists() {
        "xtask/Cargo.toml"
    } else if Path::new("Cargo.toml").exists() {
        "Cargo.toml"
    } else {
        "../Cargo.toml"
    };

    let output = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            manifest_path,
            "--",
            "non-existent-unknown-command-xyz",
        ])
        .output()
        .expect("Failed to execute xtask binary");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown xtask command: non-existent-unknown-command-xyz"),
        "stderr should contain unknown command error message, got:\n{}",
        stderr
    );
    assert!(
        stderr.contains("Available commands:"),
        "stderr should list available commands, got:\n{}",
        stderr
    );
}
